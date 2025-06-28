use crate::imp::task::sys_exit_impl;
use axerrno::LinuxResult;
use core::ffi::c_uint;
use syscall_trace::syscall_trace;

#[syscall_trace]
pub fn sys_exit(status: c_uint) -> LinuxResult<isize> {
    sys_exit_impl(status, 0, false)
}

#[syscall_trace]
pub fn sys_exit_group(status: c_uint) -> LinuxResult<isize> {
    sys_exit_impl(status, 0, true)
}
