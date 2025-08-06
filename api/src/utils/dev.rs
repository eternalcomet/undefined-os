use crate::core::file::fd::{FileDescriptor, FileLike};
use crate::core::file::file::File;
use crate::core::fs::dynamic::file::Device;
use alloc::sync::Arc;

pub fn get_device_by_fd(fd: FileDescriptor) -> Option<Arc<Device>> {
    let file = File::from_fd(fd).ok()?;
    let location = file.location()?;
    location.entry().downcast::<Device>().ok()
}
