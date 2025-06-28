use crate::imp::fs::path::*;
use crate::ptr::{UserInPtr, UserOutPtr, nullable};
use crate::utils::path::{ResolveFlags, get_fs_context, resolve_path_at};
use axerrno::{LinuxError, LinuxResult};
use core::cmp::min;
use core::ffi::{c_char, c_int, c_uint};
use linux_raw_sys::general::{AT_FDCWD, AT_REMOVEDIR};
use syscall_trace::syscall_trace;

#[syscall_trace]
pub fn sys_rename(old_path: UserInPtr<c_char>, new_path: UserInPtr<c_char>) -> LinuxResult<isize> {
    sys_rename_impl(
        AT_FDCWD,
        nullable!(old_path.get_as_str())?,
        AT_FDCWD,
        new_path.get_as_str()?,
        RenameFlags::empty(),
    )
}

#[syscall_trace]
pub fn sys_renameat(
    old_dir_fd: c_int,
    old_path: UserInPtr<c_char>,
    new_dir_fd: c_int,
    new_path: UserInPtr<c_char>,
) -> LinuxResult<isize> {
    sys_rename_impl(
        old_dir_fd,
        nullable!(old_path.get_as_str())?,
        new_dir_fd,
        new_path.get_as_str()?,
        RenameFlags::empty(),
    )
}

#[syscall_trace]
pub fn sys_renameat2(
    old_dir_fd: c_int,
    old_path: UserInPtr<c_char>,
    new_dir_fd: c_int,
    new_path: UserInPtr<c_char>,
    flags: c_uint,
) -> LinuxResult<isize> {
    let flags = RenameFlags::from_bits(flags).ok_or(LinuxError::EINVAL)?;
    sys_rename_impl(
        old_dir_fd,
        nullable!(old_path.get_as_str())?,
        new_dir_fd,
        new_path.get_as_str()?,
        flags,
    )
}

#[syscall_trace]
pub fn sys_mkdir(path_name: UserInPtr<c_char>, mode: c_uint) -> LinuxResult<isize> {
    let path_name = path_name.get_as_str()?;
    let mode = mode as u16;
    sys_mkdir_impl(AT_FDCWD, path_name, mode)
}

#[syscall_trace]
pub fn sys_mkdirat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    mode: c_uint,
) -> LinuxResult<isize> {
    let path_name = path_name.get_as_str()?;
    let mode = mode as u16;
    sys_mkdir_impl(dir_fd, path_name, mode)
}

#[syscall_trace]
pub fn sys_unlink(path_name: UserInPtr<c_char>) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    sys_unlink_impl(AT_FDCWD, path_name, UnlinkFlags::NO_REMOVE_DIR)
}

#[syscall_trace]
pub fn sys_unlinkat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    flags: c_uint,
) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let flags = if flags & AT_REMOVEDIR != 0 {
        UnlinkFlags::empty()
    } else {
        UnlinkFlags::NO_REMOVE_DIR
    };
    sys_unlink_impl(dir_fd, path_name, flags)
}

#[syscall_trace]
pub fn sys_link(old_path: UserInPtr<c_char>, new_path: UserInPtr<c_char>) -> LinuxResult<isize> {
    sys_link_impl(
        AT_FDCWD,
        nullable!(old_path.get_as_str())?,
        AT_FDCWD,
        new_path.get_as_str()?,
        0,
    )
}

#[syscall_trace]
pub fn sys_linkat(
    old_dir_fd: c_int,
    old_path: UserInPtr<c_char>,
    new_dir_fd: c_int,
    new_path: UserInPtr<c_char>,
    flags: c_uint,
) -> LinuxResult<isize> {
    sys_link_impl(
        old_dir_fd,
        nullable!(old_path.get_as_str())?,
        new_dir_fd,
        new_path.get_as_str()?,
        flags,
    )
}

#[syscall_trace]
pub fn sys_symlink(target: UserInPtr<c_char>, link_path: UserInPtr<c_char>) -> LinuxResult<isize> {
    let target = target.get_as_str()?;
    let link_path = link_path.get_as_str()?;
    sys_symlink_impl(target, AT_FDCWD, link_path)
}

#[syscall_trace]
pub fn sys_symlinkat(
    target: UserInPtr<c_char>,
    new_dir_fd: c_int,
    link_path: UserInPtr<c_char>,
) -> LinuxResult<isize> {
    let target = target.get_as_str()?;
    let link_path = link_path.get_as_str()?;
    sys_symlink_impl(target, new_dir_fd, link_path)
}

#[syscall_trace]
pub fn sys_rmdir(path_name: UserInPtr<c_char>) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    sys_unlink_impl(AT_FDCWD, path_name, UnlinkFlags::NO_REMOVE_FILE)
}

#[syscall_trace]
pub fn sys_readlinkat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    buf: UserOutPtr<u8>,
    buf_len: usize,
) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let buf = buf.get_as_mut_slice(buf_len)?;
    let resolve = resolve_path_at(dir_fd, path_name, ResolveFlags::NO_FOLLOW)?;
    let location = resolve.location().ok_or(LinuxError::EINVAL)?;
    let link = location.read_link()?;
    let link_buf = link.as_bytes();
    let len = min(buf_len, link_buf.len());
    buf[..len].copy_from_slice(&link_buf[..len]);
    Ok(len as _)
}

#[syscall_trace]
pub fn sys_getcwd(buf: UserOutPtr<u8>, size: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_mut_slice(size)?;
    let cwd = get_fs_context().current_dir.absolute_path()?;
    let cwd = cwd.as_bytes();
    if cwd.len() < size {
        buf[..cwd.len()].copy_from_slice(cwd);
        buf[cwd.len()] = 0;
        Ok(cwd.len() as isize + 1)
    } else {
        Err(LinuxError::ERANGE)
    }
}

#[syscall_trace]
pub fn sys_chmod(path_name: UserInPtr<c_char>, mode: c_uint) -> LinuxResult<isize> {
    let path_name = path_name.get_as_str()?;
    let mode = mode as u16;
    sys_chmod_impl(AT_FDCWD, Some(path_name), mode, 0)
}

#[syscall_trace]
pub fn sys_fchmod(fd: c_int, mode: c_uint) -> LinuxResult<isize> {
    let mode = mode as u16;
    sys_chmod_impl(fd, None, mode, 0)
}

#[syscall_trace]
pub fn sys_fchmodat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    mode: c_uint,
    flags: c_int,
) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let mode = mode as u16;
    sys_chmod_impl(dir_fd, path_name, mode, flags as _)
}

#[syscall_trace]
pub fn sys_chown(path_name: UserInPtr<c_char>, owner: c_int, group: c_int) -> LinuxResult<isize> {
    let path_name = path_name.get_as_str()?;
    let owner = if owner == -1 { None } else { Some(owner as _) };
    let group = if group == -1 { None } else { Some(group as _) };
    sys_chown_impl(
        AT_FDCWD,
        Some(path_name),
        owner,
        group,
        ResolveFlags::empty(),
    )
}

#[syscall_trace]
pub fn sys_fchown(fd: c_int, owner: c_int, group: c_int) -> LinuxResult<isize> {
    let owner = if owner == -1 { None } else { Some(owner as _) };
    let group = if group == -1 { None } else { Some(group as _) };
    sys_chown_impl(fd, None, owner, group, ResolveFlags::empty())
}

#[syscall_trace]
pub fn sys_lchown(path_name: UserInPtr<c_char>, owner: c_int, group: c_int) -> LinuxResult<isize> {
    let path_name = path_name.get_as_str()?;
    let owner = if owner == -1 { None } else { Some(owner as _) };
    let group = if group == -1 { None } else { Some(group as _) };
    sys_chown_impl(
        AT_FDCWD,
        Some(path_name),
        owner,
        group,
        ResolveFlags::NO_FOLLOW,
    )
}

#[syscall_trace]
pub fn sys_fchownat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    owner: c_int,
    group: c_int,
    flags: c_int,
) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let owner = if owner == -1 { None } else { Some(owner as _) };
    let group = if group == -1 { None } else { Some(group as _) };
    let resolve_flags = ResolveFlags::from_bits_truncate(flags as _);
    sys_chown_impl(dir_fd, path_name, owner, group, resolve_flags)
}

#[syscall_trace]
pub fn sys_access(path_name: UserInPtr<c_char>, mode: c_uint) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let mode = mode as u16;
    sys_access_impl(AT_FDCWD, path_name, mode, 0)
}

#[syscall_trace]
pub fn sys_faccessat(
    dir_fd: c_int,
    path_name: UserInPtr<c_char>,
    mode: c_uint,
    flags: c_int,
) -> LinuxResult<isize> {
    let path_name = nullable!(path_name.get_as_str())?;
    let mode = mode as u16;
    sys_access_impl(dir_fd, path_name, mode, flags as _)
}
