use crate::path::{resolve_path, resolve_path_with_parent};
use crate::ptr::{PtrWrapper, UserConstPtr, UserPtr};
use crate::status::{FileStatus, TimeSpec, sys_stat_impl};
use arceos_posix_api::AT_FDCWD;
use axerrno::LinuxError;
use axerrno::LinuxResult;
use core::ffi::{c_char, c_int};

/// File status: struct stat
#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Stat {
    /// ID of device containing file
    pub st_dev: usize,
    /// inode number
    pub st_ino: usize,
    /// number of hard links
    /// note that the field sequence is different from other archs
    pub st_nlink: usize,
    /// file type and mode
    pub st_mode: u32,
    /// user ID of owner
    pub st_uid: u32,
    /// group ID of owner
    pub st_gid: u32,
    /// paddings for arch x86_64
    pub _pad0: i32,
    /// device ID (if special file)
    pub st_rdev: usize,
    /// total size, in bytes
    pub st_size: isize,
    /// Block size for filesystem I/O
    pub st_blksize: isize,
    /// number of blocks allocated
    pub st_blocks: isize,
    /// time of last access
    pub st_atime: TimeSpec,
    /// time of last modification
    pub st_mtime: TimeSpec,
    /// time of last status change
    pub st_ctime: TimeSpec,
    /// glibc reserved for arch x86_64
    pub _glibc_reserved: [isize; 3],
}

#[cfg(target_arch = "x86_64")]
impl From<FileStatus> for Stat {
    fn from(fs: FileStatus) -> Self {
        Stat {
            st_dev: fs.dev,
            st_ino: fs.inode,
            st_nlink: fs.n_link,
            st_mode: fs.mode,
            st_uid: fs.uid,
            st_gid: fs.gid,
            st_rdev: fs.rdev,
            st_size: fs.size,
            st_blksize: fs.block_size,
            st_blocks: fs.n_blocks,
            st_atime: fs.access_time,
            st_mtime: fs.modify_time,
            st_ctime: fs.change_time,
            ..Default::default()
        }
    }
}

#[cfg(target_arch = "x86_64")]
const _: () = assert!(size_of::<Stat>() == 144, "size of Stat is not 144");

#[cfg(not(target_arch = "x86_64"))]
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct Stat {
    /// ID of device containing file
    pub st_dev: usize,
    /// inode number
    pub st_ino: usize,
    /// file type and mode
    pub st_mode: u32,
    /// number of hard links
    pub st_nlink: u32,
    /// user ID of owner
    pub st_uid: u32,
    /// group ID of owner
    pub st_gid: u32,
    /// device ID (if special file)
    pub st_rdev: usize,
    /// paddings for arch non x86_64
    pub _pad0: isize,
    /// total size, in bytes
    pub st_size: isize,
    /// Block size for filesystem I/O
    pub st_blksize: i32,
    /// paddings for arch non x86_64
    pub _pad1: i32,
    /// number of blocks allocated
    pub st_blocks: isize,
    /// time of last access
    pub st_atime: TimeSpec,
    /// time of last modification
    pub st_mtime: TimeSpec,
    /// time of last status change
    pub st_ctime: TimeSpec,
    /// reserved for arch non x86_64
    pub _unused: [i32; 2],
}

#[cfg(not(target_arch = "x86_64"))]
impl From<FileStatus> for Stat {
    fn from(fs: FileStatus) -> Self {
        Stat {
            st_dev: fs.dev,
            st_ino: fs.inode,
            st_mode: fs.mode,
            st_nlink: fs.n_link as u32,
            st_uid: fs.uid,
            st_gid: fs.gid,
            st_rdev: fs.rdev,
            st_size: fs.size,
            st_blksize: fs.block_size as i32,
            st_blocks: fs.n_blocks,
            st_atime: fs.access_time,
            st_mtime: fs.modify_time,
            st_ctime: fs.change_time,
            ..Default::default()
        }
    }
}

#[cfg(not(target_arch = "x86_64"))]
const _: () = assert!(size_of::<Stat>() == 128, "size of Stat is not 128");

pub fn sys_stat(path: UserConstPtr<c_char>, stat_buf: UserPtr<Stat>) -> LinuxResult<isize> {
    // get params
    let path = path.get_as_str()?;
    let stat_buf = stat_buf.get()?;

    // perform syscall
    let result = (|| -> LinuxResult<_> { sys_stat_impl(-1, path, false) })();

    // check result
    match result {
        Ok(fs) => {
            let stat: Stat = fs.into();
            debug!(
                "[syscall] stat(pathname={:?}, statbuf={:?}) = {}",
                path, stat, 0
            );
            // copy to user space
            unsafe { stat_buf.write(fs.into()) }
            Ok(0)
        }
        Err(err) => {
            debug!(
                "[syscall] stat(pathname={:?}, statbuf={:p}) = {:?}",
                path, stat_buf, err
            );
            Err(err)
        }
    }
}

pub fn sys_lstat(path: UserConstPtr<c_char>, stat_buf: UserPtr<Stat>) -> LinuxResult<isize> {
    // get params
    let path = path.get_as_str()?;
    let stat_buf = stat_buf.get()?;

    // perform syscall
    let result = (|| -> LinuxResult<_> { sys_stat_impl(-1, path, true) })();

    // check result
    match result {
        Ok(fs) => {
            let stat: Stat = fs.into();
            debug!(
                "[syscall] lstat(pathname={:?}, statbuf={:?}) = {}",
                path, stat, 0
            );
            // copy to user space
            unsafe { stat_buf.write(fs.into()) }
            Ok(0)
        }
        Err(err) => {
            debug!(
                "[syscall] lstat(pathname={:?}, statbuf={:p}) = {:?}",
                path, stat_buf, err
            );
            Err(err)
        }
    }
}

pub fn sys_fstat(fd: c_int, stat_buf: UserPtr<Stat>) -> LinuxResult<isize> {
    // get params
    let stat_buf = stat_buf.get()?;

    // perform syscall
    let result = (|| -> LinuxResult<_> {
        if fd < 0 && fd != AT_FDCWD as i32 {
            Err(LinuxError::EBADFD)
        } else {
            sys_stat_impl(fd, "", false)
        }
    })();

    // check result
    match result {
        Ok(fs) => {
            let stat: Stat = fs.into();
            debug!("[syscall] fstat(fd={}, statbuf={:?}) = {}", fd, stat, 0);
            // copy to user space
            unsafe { stat_buf.write(fs.into()) }
            Ok(0)
        }
        Err(err) => {
            debug!(
                "[syscall] fstat(fd={}, statbuf={:p}) = {:?}",
                fd, stat_buf, err
            );
            Err(err)
        }
    }
}

pub fn sys_fstatat(
    dir_fd: c_int,
    path: UserConstPtr<c_char>,
    stat_buf: UserPtr<Stat>,
    flags: c_int,
) -> LinuxResult<isize> {
    // constants
    const AT_EMPTY_PATH: c_int = 0x1000;
    const AT_SYMLINK_NOFOLLOW: c_int = 0x100;

    // get params
    let path = path.get_as_str().unwrap_or("");
    let stat_buf = stat_buf.get()?;

    // perform syscall
    let result = (|| -> LinuxResult<_> {
        if dir_fd < 0 && dir_fd != AT_FDCWD as i32 {
            return Err(LinuxError::EBADFD);
        }
        if path.is_empty() && (flags & AT_EMPTY_PATH == 0) {
            return Err(LinuxError::ENOENT);
        }
        let follow_symlinks = flags & AT_SYMLINK_NOFOLLOW == 0;
        sys_stat_impl(dir_fd, path, follow_symlinks)
    })();

    // check result
    match result {
        Ok(fs) => {
            let stat: Stat = fs.into();
            debug!(
                "[syscall] fstatat(dirfd={}, pathname={:?}, statbuf={:?}, flags={}) = {}",
                dir_fd, path, stat, flags, 0
            );
            // copy to user space
            unsafe { stat_buf.write(fs.into()) }
            Ok(0)
        }
        Err(err) => {
            debug!(
                "[syscall] fstatat(dirfd={}, pathname={:?}, statbuf={:p}, flags={}) = {:?}",
                dir_fd, path, stat_buf, flags, err
            );
            Err(err)
        }
    }
}
