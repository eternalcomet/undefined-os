use axfs_ng::api;
use axsync::RawMutex;
use undefined_vfs::mount::Location;

pub mod dir;
pub mod epoll;
pub mod fd;
pub mod file;
pub mod pathfd;
pub mod pipe;
pub mod stdio;
pub mod stub;

pub type ApiFile = api::File<RawMutex>;
pub type ApiDir = api::Directory<RawMutex>;
pub type FsLocation = Location<RawMutex>;
