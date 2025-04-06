use core::ffi::{c_int, c_ulong};

use crate::ptr::{PtrWrapper, UserPtr};
use arceos_posix_api as api;
use arceos_posix_api::PollFd;
use axerrno::LinuxResult;

pub fn sys_dup(old_fd: c_int) -> LinuxResult<isize> {
    Ok(api::sys_dup(old_fd) as _)
}

pub fn sys_dup2(old_fd: c_int, new_fd: c_int) -> LinuxResult<isize> {
    Ok(api::sys_dup2(old_fd, new_fd) as _)
}

pub fn sys_dup3(old_fd: c_int, new_fd: c_int) -> LinuxResult<isize> {
    Ok(api::sys_dup2(old_fd, new_fd) as _)
}

pub fn sys_close(fd: c_int) -> LinuxResult<isize> {
    Ok(api::sys_close(fd) as _)
}

pub fn sys_fcntl(fd: c_int, cmd: c_int, arg: usize) -> LinuxResult<isize> {
    Ok(api::sys_fcntl(fd, cmd, arg) as _)
}

pub fn sys_poll(fds: UserPtr<PollFd>, nfds: c_ulong, timeout: c_int) -> LinuxResult<isize> {
    let fds = fds.get_as_array(nfds as _)?;
    let fds: &mut [PollFd] = unsafe { core::slice::from_raw_parts_mut(fds, nfds as _) };
    Ok(api::sys_poll(fds, timeout) as _)
}
