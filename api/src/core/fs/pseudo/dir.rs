use crate::core::fs::pseudo::dynamic::DynNodeOps;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use undefined_vfs::VfsResult;

pub trait PseudoDirOps: Send + Sync + 'static {
    fn list_children<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, str>> + 'a>;
    fn get_child(&self, name: &str) -> VfsResult<DynNodeOps>;
}
