use crate::core::file::fd::FileLike;
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::FileFlags;
use axio::PollState;
use core::any::Any;
use undefined_vfs::types::{Metadata, NodePermission};

pub struct StubFd {}

impl StubFd {
    pub fn new() -> Self {
        Self {}
    }
}

impl FileLike for StubFd {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EINVAL)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EINVAL)
    }

    fn status(&self) -> LinuxResult<Metadata> {
        Ok(Metadata {
            inode: 6,
            n_link: 1,
            mode: NodePermission::OWNER_READ | NodePermission::OWNER_WRITE,
            ..Default::default()
        })
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn get_flags(&self) -> FileFlags {
        FileFlags::empty()
    }

    fn set_flags(&self, _flags: FileFlags) {}

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}
