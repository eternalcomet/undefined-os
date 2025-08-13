use core::{any::Any, iter, time::Duration};

use crate::core::fs::pseudo::dir::PseudoDirOps;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned, collections::btree_map::BTreeMap, string::String, sync::Arc};
use axsync::{Mutex, RawMutex};
use inherit_methods_macro::inherit_methods;
use slab::Slab;
use undefined_vfs::fs::{Filesystem, FilesystemOps, StatFs};
use undefined_vfs::node::{
    DirEntry, DirEntrySink, DirNode, DirNodeOps, FileNode, FileNodeOps, NodeOps, Reference,
    WeakDirEntry,
};
use undefined_vfs::path::{DOT, DOTDOT, MAX_NAME_LEN};
use undefined_vfs::types::{DeviceId, Metadata, MetadataUpdate, NodePermission, NodeType};
use undefined_vfs::{VfsError, VfsResult};

pub type DirMaker =
    Arc<dyn Fn(WeakDirEntry<RawMutex>) -> Arc<dyn DirNodeOps<RawMutex>> + Send + Sync>;

pub fn dummy_stat_fs(fs_type: u32) -> StatFs {
    StatFs {
        fs_type,
        block_size: 512,
        blocks: 100,
        blocks_free: 100,
        blocks_available: 100,

        file_count: 0,
        free_file_count: 0,

        name_length: MAX_NAME_LEN as _,
        fragment_size: 0,
        mount_flags: 0,
    }
}

pub struct DynamicFs {
    name: String,
    fs_type: u32,
    inodes: Mutex<Slab<()>>,
    root: Mutex<Option<DirEntry<RawMutex>>>,
}
impl DynamicFs {
    pub fn new_with(
        name: String,
        fs_type: u32,
        root: impl FnOnce(Arc<DynamicFs>) -> DirMaker,
    ) -> Filesystem<RawMutex> {
        let fs = Arc::new(Self {
            name,
            fs_type,
            inodes: Mutex::default(),
            root: Mutex::default(),
        });
        let root = root(fs.clone());
        fs.set_root(DirEntry::new_dir(
            |this| DirNode::new(root(this)),
            Reference::root(),
        ));
        Filesystem::new(fs)
    }

    pub fn set_root(&self, root: DirEntry<RawMutex>) {
        *self.root.lock() = Some(root);
    }

    pub fn alloc_inode(&self) -> u64 {
        self.inodes.lock().insert(()) as u64 + 1
    }
    pub fn release_inode(&self, ino: u64) {
        self.inodes.lock().remove(ino as usize - 1);
    }
}
impl FilesystemOps<RawMutex> for DynamicFs {
    fn name(&self) -> &str {
        &self.name
    }

    fn root_dir(&self) -> DirEntry<RawMutex> {
        self.root.lock().clone().unwrap()
    }

    fn stat(&self) -> VfsResult<StatFs> {
        Ok(dummy_stat_fs(self.fs_type))
    }
}

#[derive(Clone)]
pub enum DynNodeOps {
    Dir(DirMaker),
    File(Arc<dyn FileNodeOps<RawMutex>>),
}
impl From<DirMaker> for DynNodeOps {
    fn from(maker: DirMaker) -> Self {
        Self::Dir(maker)
    }
}
impl<T: FileNodeOps<RawMutex> + 'static> From<Arc<T>> for DynNodeOps {
    fn from(ops: Arc<T>) -> Self {
        Self::File(ops)
    }
}

pub struct DynamicNode {
    fs: Arc<DynamicFs>,
    ino: u64,
    pub(crate) metadata: Mutex<Metadata>,
}
impl DynamicNode {
    pub fn new(fs: Arc<DynamicFs>, node_type: NodeType, mode: NodePermission) -> Self {
        let ino = fs.alloc_inode();
        let metadata = Metadata {
            device: 0,
            inode: ino,
            n_link: 1,
            mode,
            node_type,
            uid: 0,
            gid: 0,
            size: 0,
            block_size: 0,
            n_blocks: 0,
            raw_device: DeviceId::default(),
            access_time: Duration::default(),
            modify_time: Duration::default(),
            change_time: Duration::default(),
        };
        Self {
            fs,
            ino,
            metadata: Mutex::new(metadata),
        }
    }
}

pub struct DynamicDir {
    node: DynamicNode,
    this: WeakDirEntry<RawMutex>,
    children: Arc<BTreeMap<String, DynNodeOps>>,
    pseudo_ops: Option<Arc<dyn PseudoDirOps>>,
}
impl DynamicDir {
    fn new(
        node: DynamicNode,
        children: Arc<BTreeMap<String, DynNodeOps>>,
        this: WeakDirEntry<RawMutex>,
        pseudo_ops: Option<Arc<dyn PseudoDirOps>>,
    ) -> Arc<DynamicDir> {
        Arc::new(Self {
            node,
            this,
            children,
            pseudo_ops,
        })
    }

    pub fn builder(fs: Arc<DynamicFs>) -> DynamicDirBuilder {
        DynamicDirBuilder::new(fs)
    }
}
impl Drop for DynamicNode {
    fn drop(&mut self) {
        self.fs.release_inode(self.ino);
    }
}

pub struct DynamicDirBuilder {
    fs: Arc<DynamicFs>,
    children: BTreeMap<String, DynNodeOps>,
    pseudo_ops: Option<Arc<dyn PseudoDirOps>>,
}
impl DynamicDirBuilder {
    pub fn new(fs: Arc<DynamicFs>) -> Self {
        Self {
            fs,
            children: BTreeMap::new(),
            pseudo_ops: None,
        }
    }

    pub fn add(&mut self, name: impl Into<String>, ops: impl Into<DynNodeOps>) {
        self.children.insert(name.into(), ops.into());
    }

    pub fn set_pseudo_ops(&mut self, ops: impl PseudoDirOps + 'static) {
        self.pseudo_ops = Some(Arc::new(ops));
    }

    pub fn build(self) -> DirMaker {
        let children = Arc::new(self.children);
        Arc::new(move |this| {
            DynamicDir::new(
                DynamicNode::new(
                    self.fs.clone(),
                    NodeType::Directory,
                    NodePermission::from_bits_truncate(0o755),
                ),
                children.clone(),
                this,
                self.pseudo_ops.clone(),
            )
        })
    }
}

impl NodeOps<RawMutex> for DynamicNode {
    fn inode(&self) -> u64 {
        self.ino
    }

    fn metadata(&self) -> VfsResult<Metadata> {
        let mut metadata = self.metadata.lock().clone();
        metadata.size = self.size()?;
        Ok(metadata)
    }

    fn update_metadata(&self, update: MetadataUpdate) -> VfsResult<()> {
        let mut metadata = self.metadata.lock();
        if let Some(mode) = update.mode {
            metadata.mode = mode;
        }
        if let Some((uid, gid)) = update.owner {
            metadata.uid = uid;
            metadata.gid = gid;
        }
        if let Some(atime) = update.atime {
            metadata.access_time = atime;
        }
        if let Some(mtime) = update.mtime {
            metadata.modify_time = mtime;
        }
        Ok(())
    }

    fn filesystem(&self) -> &dyn FilesystemOps<RawMutex> {
        self.fs.as_ref()
    }

    fn size(&self) -> VfsResult<u64> {
        Ok(0)
    }

    fn sync(&self, _data_only: bool) -> VfsResult<()> {
        Ok(())
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[inherit_methods(from = "self.node")]
impl NodeOps<RawMutex> for DynamicDir {
    fn inode(&self) -> u64;
    fn metadata(&self) -> VfsResult<Metadata>;
    fn update_metadata(&self, update: MetadataUpdate) -> VfsResult<()>;
    fn filesystem(&self) -> &dyn FilesystemOps<RawMutex>;
    fn sync(&self, data_only: bool) -> VfsResult<()>;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl DirNodeOps<RawMutex> for DynamicDir {
    fn read_dir(&self, offset: u64, sink: &mut dyn DirEntrySink) -> VfsResult<usize> {
        let pseudo_children = if let Some(pseudo_ops) = &self.pseudo_ops {
            pseudo_ops.list_children()
        } else {
            Box::new(iter::empty())
        };
        let special_children = [DOT, DOTDOT].iter().map(|s| Cow::Borrowed(*s));
        let ordinary_children = self.children.keys().map(|k| Cow::Borrowed(k.as_str()));
        let all_children = special_children
            .chain(ordinary_children)
            .chain(pseudo_children);

        let this_entry = self.this.upgrade().unwrap();
        let this_dir = this_entry.as_dir()?;

        let mut count = 0;
        for (i, name) in all_children.enumerate().skip(offset as usize) {
            let name = name.as_ref();
            let metadata = match name {
                DOT => this_entry.metadata(),
                DOTDOT => this_entry
                    .parent()
                    .map_or_else(|| this_entry.metadata(), |parent| parent.metadata()),
                _ => {
                    let entry = this_dir.lookup(name)?;
                    entry.metadata()
                } // DOTDOT => self.
            }?;
            if !sink.accept(name, metadata.inode, metadata.node_type, i as u64 + 1) {
                break;
            }
            count += 1;
        }

        Ok(count)
    }

    fn lookup(&self, name: &str) -> VfsResult<DirEntry<RawMutex>> {
        let ops = if let Some(ops) = self.children.get(name) {
            Cow::Borrowed(ops)
        } else if let Some(pseudo_ops) = &self.pseudo_ops {
            Cow::Owned(pseudo_ops.get_child(name)?)
        } else {
            return Err(VfsError::ENOENT);
        };
        let reference = Reference::new(self.this.upgrade(), name.to_owned());
        let ops = ops.as_ref();
        Ok(match ops {
            DynNodeOps::Dir(maker) => {
                DirEntry::new_dir(|this| DirNode::new(maker(this)), reference)
            }
            DynNodeOps::File(ops) => {
                let node_type = ops.metadata()?.node_type;
                DirEntry::new_file(FileNode::new(ops.clone()), node_type, reference)
            }
        })
    }

    fn create(
        &self,
        _name: &str,
        _node_type: NodeType,
        _permission: NodePermission,
    ) -> VfsResult<DirEntry<RawMutex>> {
        Err(VfsError::EPERM)
    }

    fn link(&self, _name: &str, _node: &DirEntry<RawMutex>) -> VfsResult<DirEntry<RawMutex>> {
        Err(VfsError::EPERM)
    }

    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::EPERM)
    }

    fn rename(
        &self,
        _src_name: &str,
        _dst_dir: &DirNode<RawMutex>,
        _dst_name: &str,
    ) -> VfsResult<()> {
        Err(VfsError::EPERM)
    }
}
