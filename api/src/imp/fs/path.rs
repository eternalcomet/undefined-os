use crate::core::fs::fd::{FileDescriptor, file_like_as};
use crate::core::fs::file::File;
use crate::utils::path::{Resolve, ResolveFlags, resolve_path_at, resolve_path_at_existed};
use axerrno::{LinuxError, LinuxResult};
use bitflags::bitflags;
use linux_raw_sys::general::{
    R_OK, RENAME_EXCHANGE, RENAME_NOREPLACE, RENAME_WHITEOUT, W_OK, X_OK,
};
use undefined_vfs::types::{NodePermission, NodeType};

bitflags! {
    #[derive(Debug)]
    pub struct RenameFlags: u32 {
        const NOREPLACE = RENAME_NOREPLACE;
        const EXCHANGE = RENAME_EXCHANGE;
        const WHITEOUT = RENAME_WHITEOUT;
    }
}

pub fn sys_rename_impl(
    old_dir_fd: FileDescriptor,
    old_path: &str,
    new_dir_fd: FileDescriptor,
    new_path: &str,
    flags: RenameFlags,
) -> LinuxResult<isize> {
    let old_path = resolve_path_at(old_dir_fd, old_path, ResolveFlags::NO_FOLLOW)?;
    let old_path = old_path.location().ok_or(LinuxError::ENOTDIR)?;
    let (new_path, new_name) = resolve_path_at_existed(new_dir_fd, new_path)?;

    let parent = old_path.parent().ok_or(LinuxError::EINVAL)?;
    if new_name.is_empty() {
        if flags.contains(RenameFlags::NOREPLACE) {
            return Err(LinuxError::EEXIST);
        } else {
            // new_location.unlink()
            let parent = new_path.parent().ok_or(LinuxError::EINVAL)?;
            parent.rename(old_path.name(), &parent, new_path.name())?;
        }
    } else {
    }
    parent.rename(old_path.name(), &new_path, new_name.as_str())?;
    Ok(0)
}

pub fn sys_mkdir_impl(dir_fd: FileDescriptor, path: &str, mode: u16) -> LinuxResult<isize> {
    let mode = NodePermission::from_bits(mode).ok_or(LinuxError::EINVAL)?;
    let (location, name) = resolve_path_at_existed(dir_fd, path)?;
    location.create(name.as_ref(), NodeType::Directory, mode)?;
    Ok(0)
}

bitflags! {
    #[derive(Debug)]
    pub struct UnlinkFlags: u8 {
        const NO_REMOVE_DIR = 0x1;
        const NO_REMOVE_FILE = 0x2;
    }
}

pub fn sys_unlink_impl(
    dir_fd: FileDescriptor,
    path: &str,
    flags: UnlinkFlags,
) -> LinuxResult<isize> {
    // If the name referred to a symbolic link, the link is removed.
    let path = resolve_path_at(dir_fd, path, ResolveFlags::NO_FOLLOW)?;
    // TODO: we do not support removing a socket, FIFO, or device
    let path = path.location().ok_or(LinuxError::EPERM)?;
    // let meta = path.metadata();
    if path.is_dir() {
        if flags.contains(UnlinkFlags::NO_REMOVE_DIR) {
            return Err(LinuxError::EISDIR);
        }
        path.parent()
            .ok_or(LinuxError::EPERM)?
            .unlink(path.name(), true)?;
    } else if path.is_file() {
        if flags.contains(UnlinkFlags::NO_REMOVE_FILE) {
            return Err(LinuxError::ENOTDIR);
        }
        path.parent()
            .ok_or(LinuxError::EPERM)?
            .unlink(path.name(), false)?;
    } else {
        // other types of files, like symlink, socket, etc.
        return Err(LinuxError::EPERM);
    }
    Ok(0)
}

pub fn sys_link_impl(
    old_dir_fd: FileDescriptor,
    old_path: &str,
    new_dir_fd: FileDescriptor,
    new_path: &str,
    flags: u32,
) -> LinuxResult<isize> {
    let flags = ResolveFlags::from_bits_truncate(flags);
    let old_path = resolve_path_at(old_dir_fd, old_path, flags)?;
    let (new_path, new_name) = resolve_path_at_existed(new_dir_fd, new_path)?;

    let _location = match old_path {
        Resolve::FileLike(file_like) => {
            let file = file_like_as::<File>(file_like).ok_or(LinuxError::EPERM)?;
            new_path.link(new_name.as_ref(), file.inner().location())?;
        }
        Resolve::Location(location) => {
            new_path.link(new_name.as_ref(), &location)?;
        }
    };
    Ok(0)
}

bitflags! {
    #[derive(Debug)]
    pub struct AccessFlags: u32 {
        const R_OK = R_OK;
        const W_OK = W_OK;
        const X_OK = X_OK;
    }
}

// TODO: sys_access_impl
