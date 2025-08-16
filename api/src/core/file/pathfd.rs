use crate::core::file::fd::{FileDescriptor, FileLike, fd_lookup};
use crate::core::file::{ApiFile, FsLocation};
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::FileFlags;
use axio::PollState;
use axsync::{Mutex, MutexGuard};
use core::any::Any;
use undefined_vfs::types::Metadata;

/// File opened with flag `O_PATH`.
/// The file itself is not opened, and other file operations
/// (e.g., read(2), write(2), fchmod(2), fchown(2), fgetxattr(2), ioctl(2), mmap(2))
/// fail with the error EBADF.
pub struct PathFile {
    inner: Mutex<ApiFile>,
}

impl PathFile {
    pub fn new(inner: ApiFile) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub fn inner(&self) -> MutexGuard<ApiFile> {
        self.inner.lock()
    }

    pub fn from_location(location: FsLocation, flags: FileFlags) -> Self {
        let file = ApiFile::new(location, flags);
        Self::new(file)
    }

    pub fn from_fd(fd: FileDescriptor) -> LinuxResult<Arc<Self>> {
        let file_like = fd_lookup(fd)?;
        let err = file_like.type_mismatch_error();
        file_like.into_any().downcast::<Self>().map_err(|_| err)
    }
}

impl FileLike for PathFile {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EBADF)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EBADF)
    }

    fn status(&self) -> LinuxResult<Metadata> {
        self.inner().metadata()
    }

    fn poll(&self) -> LinuxResult<PollState> {
        // Regular files are always readable and writable, regardless of file flags
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn get_flags(&self) -> FileFlags {
        self.inner.lock().get_flags()
    }

    fn set_flags(&self, _flags: FileFlags) {
        warn!("set file flags is not implemented for regular files");
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }

    fn location(&self) -> Option<FsLocation> {
        Some(self.inner().location().clone())
    }
}
