use core::{any::Any, cmp::Ordering};

use super::dynamic::{DynamicFs, DynamicNode};
use alloc::{borrow::Cow, sync::Arc, vec::Vec};
use axsync::RawMutex;
use inherit_methods_macro::inherit_methods;
use undefined_vfs::fs::FilesystemOps;
use undefined_vfs::node::{FileNodeOps, NodeOps};
use undefined_vfs::types::{DeviceId, Metadata, MetadataUpdate, NodePermission, NodeType};
use undefined_vfs::{VfsError, VfsResult};

pub trait SimpleFileOps: Send + Sync {
    fn read_all(&self) -> VfsResult<Cow<[u8]>>;
    fn write_all(&self, data: &[u8]) -> VfsResult<()>;
}

impl<F, R> SimpleFileOps for F
where
    F: Fn() -> R + Send + Sync + 'static,
    R: Into<Vec<u8>>,
{
    fn read_all(&self) -> VfsResult<Cow<[u8]>> {
        Ok(Cow::Owned(self().into()))
    }

    fn write_all(&self, _data: &[u8]) -> VfsResult<()> {
        Err(VfsError::EBADF)
    }
}

pub struct SimpleFile {
    node: DynamicNode,
    ops: Arc<dyn SimpleFileOps>,
}
impl SimpleFile {
    pub fn new(fs: Arc<DynamicFs>, ops: impl SimpleFileOps + 'static) -> Arc<Self> {
        let node = DynamicNode::new(fs, NodeType::RegularFile, NodePermission::default());
        Arc::new(Self {
            node,
            ops: Arc::new(ops),
        })
    }
}

#[inherit_methods(from = "self.node")]
impl NodeOps<RawMutex> for SimpleFile {
    fn inode(&self) -> u64;
    fn metadata(&self) -> VfsResult<Metadata>;
    fn update_metadata(&self, update: MetadataUpdate) -> VfsResult<()>;
    fn filesystem(&self) -> &dyn FilesystemOps<RawMutex>;
    fn size(&self) -> VfsResult<u64> {
        Ok(self.ops.read_all()?.len() as u64)
    }
    fn sync(&self, data_only: bool) -> VfsResult<()>;

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl FileNodeOps<RawMutex> for SimpleFile {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        let data = self.ops.read_all()?;
        if offset >= data.len() as u64 {
            return Ok(0);
        }
        let data = &data[offset as usize..];
        let read = data.len().min(buf.len());
        buf[..read].copy_from_slice(&data[..read]);
        Ok(read)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        let offset = offset as usize;
        let data = self.ops.read_all()?;
        if offset == 0 && buf.len() >= data.len() {
            self.ops.write_all(buf)?;
            return Ok(buf.len());
        }
        let mut data = data.to_vec();
        let end_pos = offset + buf.len();
        if end_pos > data.len() {
            data.resize(end_pos, 0);
        }
        data[offset..end_pos].copy_from_slice(buf);
        self.ops.write_all(&data)?;
        Ok(buf.len())
    }

    fn append(&self, buf: &[u8]) -> VfsResult<(usize, u64)> {
        let mut data = self.ops.read_all()?.to_vec();
        data.extend_from_slice(buf);
        self.ops.write_all(&data)?;
        Ok((buf.len(), data.len() as u64))
    }

    fn resize(&self, len: u64) -> VfsResult<()> {
        let data = self.ops.read_all()?;
        match len.cmp(&(data.len() as u64)) {
            Ordering::Less => self.ops.write_all(&data[..len as usize]),
            Ordering::Greater => {
                let mut data = data.to_vec();
                data.resize(len as usize, 0);
                self.ops.write_all(&data)
            }
            _ => Ok(()),
        }
    }

    fn set_symlink(&self, target: &str) -> VfsResult<()> {
        self.ops.write_all(target.as_bytes())
    }
}

pub struct DeviceMem {
    pub physical_addr: usize,
    pub length: usize,
}

pub trait DeviceOps: Send + Sync {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize>;
    fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize>;
    fn get_device_mem(&self) -> Option<DeviceMem> {
        None
    }
    fn ioctl(&self, op: u32, arg: usize) -> VfsResult<isize> {
        warn!(
            "[ioctl] Unsupported ioctl operation. op: {}, arg: {}",
            op, arg
        );
        // The specified operation does not apply to the kind of object that the file descriptor references.
        Err(VfsError::ENOTTY)
    }
}
impl<F> DeviceOps for F
where
    F: Fn(&mut [u8], u64) -> VfsResult<usize> + Send + Sync + 'static,
{
    fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        self(buf, offset)
    }

    fn write_at(&self, _buf: &[u8], _offset: u64) -> VfsResult<usize> {
        Err(VfsError::EBADF)
    }
}

pub struct Device {
    node: DynamicNode,
    ops: Arc<dyn DeviceOps>,
}
impl Device {
    pub fn new(
        fs: Arc<DynamicFs>,
        node_type: NodeType,
        device_id: DeviceId,
        ops: impl DeviceOps + 'static,
    ) -> Arc<Self> {
        let node = DynamicNode::new(fs, node_type, NodePermission::default());
        node.metadata.lock().raw_device = device_id;
        Arc::new(Self {
            node,
            ops: Arc::new(ops),
        })
    }

    pub fn ops(&self) -> &Arc<dyn DeviceOps> {
        &self.ops
    }
}

#[inherit_methods(from = "self.node")]
impl NodeOps<RawMutex> for Device {
    fn inode(&self) -> u64;
    fn metadata(&self) -> VfsResult<Metadata>;
    fn update_metadata(&self, update: MetadataUpdate) -> VfsResult<()>;
    fn filesystem(&self) -> &dyn FilesystemOps<RawMutex>;
    fn size(&self) -> VfsResult<u64> {
        Ok(0)
    }
    fn sync(&self, data_only: bool) -> VfsResult<()>;

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

impl FileNodeOps<RawMutex> for Device {
    fn read_at(&self, buf: &mut [u8], offset: u64) -> VfsResult<usize> {
        self.ops.read_at(buf, offset)
    }

    fn write_at(&self, buf: &[u8], offset: u64) -> VfsResult<usize> {
        self.ops.write_at(buf, offset)
    }

    fn append(&self, _buf: &[u8]) -> VfsResult<(usize, u64)> {
        Err(VfsError::ENOTTY)
    }

    fn resize(&self, _len: u64) -> VfsResult<()> {
        Err(VfsError::EBADF)
    }

    fn set_symlink(&self, _target: &str) -> VfsResult<()> {
        Err(VfsError::EBADF)
    }
}
