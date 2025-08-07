use crate::core::file::fd::{FileDescriptor, file_like_as};
use crate::core::file::file::File;
use crate::interface::user::identity::{sys_getgid, sys_getuid};
use crate::utils::path::{
    Resolve, ResolveFlags, get_fs_context, resolve_path_at, resolve_path_at_existed,
};
use axerrno::{LinuxError, LinuxResult};
use bitflags::bitflags;
use linux_raw_sys::general::{
    AT_SYMLINK_FOLLOW, AT_SYMLINK_NOFOLLOW, R_OK, RENAME_EXCHANGE, RENAME_NOREPLACE,
    RENAME_WHITEOUT, W_OK, X_OK,
};
use undefined_vfs::types::{MetadataUpdate, NodePermission, NodeType};

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
    old_path: Option<&str>,
    new_dir_fd: FileDescriptor,
    new_path: &str,
    flags: RenameFlags,
) -> LinuxResult<isize> {
    let old_path = resolve_path_at(old_dir_fd, old_path, ResolveFlags::NO_FOLLOW)?;
    let old_path = old_path.location().ok_or(LinuxError::ENOTDIR)?;
    let (new_path, new_name) = resolve_path_at_existed(new_dir_fd, new_path, true)?;

    let parent = old_path.parent().ok_or(LinuxError::EINVAL)?;
    if new_name.is_empty() {
        // new path already exists
        if flags.contains(RenameFlags::NOREPLACE) {
            return Err(LinuxError::EEXIST);
        } else {
            // needn't unlink the old path
            let parent = new_path.parent().ok_or(LinuxError::EINVAL)?;
            parent.rename(old_path.name(), &parent, new_path.name())?;
        }
    } else {
        parent.rename(old_path.name(), &new_path, new_name.as_str())?;
    }
    Ok(0)
}

pub fn sys_mkdir_impl(dir_fd: FileDescriptor, path: &str, mode: u16) -> LinuxResult<isize> {
    let mode = NodePermission::from_bits(mode).ok_or(LinuxError::EINVAL)?;
    let (location, name) = resolve_path_at_existed(dir_fd, path, true)?;
    if name.is_empty() {
        return Err(LinuxError::EEXIST);
    }
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
    path: Option<&str>,
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
    old_path: Option<&str>,
    new_dir_fd: FileDescriptor,
    new_path: &str,
    mut flags: u32,
) -> LinuxResult<isize> {
    // By default, link(), does not dereference old_path if it is a symbolic link,
    // unless flag AT_SYMLINK_FOLLOW is specified.
    if (flags & AT_SYMLINK_FOLLOW) == 0 {
        flags |= AT_SYMLINK_NOFOLLOW;
    }
    let flags = ResolveFlags::from_bits_truncate(flags);
    let old_path = resolve_path_at(old_dir_fd, old_path, flags)?;
    let (new_path, new_name) = resolve_path_at_existed(new_dir_fd, new_path, true)?;
    let new_name = new_name.as_str();

    if new_name.is_empty() {
        return Err(LinuxError::EEXIST);
    }

    match old_path {
        Resolve::FileLike(file_like) => {
            let file = file_like_as::<File>(file_like).ok_or(LinuxError::EPERM)?;
            new_path.link(new_name, file.inner().location())?;
        }
        Resolve::Location(location) => {
            new_path.link(new_name, &location)?;
        }
    };
    Ok(0)
}

pub fn sys_symlink_impl(
    target: &str,
    dir_fd: FileDescriptor,
    link_path: &str,
) -> LinuxResult<isize> {
    let (location, name) = resolve_path_at_existed(dir_fd, link_path, true)?;
    let name = name.as_str();
    if name.is_empty() {
        return Err(LinuxError::EEXIST);
    }
    let permission = get_fs_context().get_permissions(0o666);
    let symlink = location.create(name, NodeType::Symlink, permission)?;
    symlink.entry().as_file()?.set_symlink(target)?;
    Ok(0)
}

pub fn sys_chmod_impl(
    dir_fd: FileDescriptor,
    path: Option<&str>,
    mode: u16,
    flags: u32,
) -> LinuxResult<isize> {
    let flags = ResolveFlags::from_bits_truncate(flags);
    let path = resolve_path_at(dir_fd, path, flags)?;
    let location = path.location().ok_or(LinuxError::ENOTDIR)?;
    let permission = NodePermission::from_bits(mode).ok_or(LinuxError::EINVAL)?;
    location.update_metadata(MetadataUpdate {
        mode: Some(permission),
        ..Default::default()
    })?;
    Ok(0)
}

pub fn sys_chown_impl(
    dir_fd: FileDescriptor,
    path: Option<&str>,
    owner: Option<u32>,
    group: Option<u32>,
    flags: ResolveFlags,
) -> LinuxResult<isize> {
    let path = resolve_path_at(dir_fd, path, flags)?;
    let location = path.location().ok_or(LinuxError::ENOTDIR)?;
    let metadata = location.metadata()?;
    let uid = owner.unwrap_or(metadata.uid);
    let gid = group.unwrap_or(metadata.gid);
    // TODO: check if the caller has permission to change the owner/group
    // Only a privileged process (Linux: one with the CAP_CHOWN capability) may change the owner of a file.
    // The owner of a file may change the group of the file to any group of which that owner is a member.
    // A privileged process (Linux: with CAP_CHOWN) may change the group arbitrarily.
    location.update_metadata(MetadataUpdate {
        owner: Some((uid, gid)),
        ..Default::default()
    })?;
    Ok(0)
}

pub fn sys_access_impl(
    dir_fd: FileDescriptor,
    path: Option<&str>,
    mode: u16,
    flags: u32,
) -> LinuxResult<isize> {
    // The check is done using the calling process's real UID and GID,
    // rather than the effective IDs as is done when actually attempting
    // an operation (e.g., open) on the file.
    let uid = sys_getuid()? as u32;
    let gid = sys_getgid()? as u32;
    let flags = ResolveFlags::from_bits_truncate(flags);
    let path = resolve_path_at(dir_fd, path, flags)?;
    let metadata = path.metadata()?;
    let mut permission = metadata.mode.bits();
    // permissions for "other" users
    let mut mode_granted = permission & 0o7;
    // permissions for "group" users
    permission >>= 3;
    if gid == metadata.gid {
        mode_granted |= permission & 0o7;
    }
    // permissions for "owner" user
    permission >>= 3;
    if uid == metadata.uid {
        mode_granted |= permission & 0o7;
    }
    // check if the requested mode is granted
    let mode_requested = mode & 0o7;
    if mode_requested & mode_granted != mode_requested {
        Err(LinuxError::EACCES)
    } else {
        Ok(0)
    }
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
