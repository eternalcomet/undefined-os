use axerrno::LinuxResult;
use syscall_trace::syscall_trace;

#[syscall_trace]
pub fn sys_madvise(addr: usize, len: usize, flags: usize) -> LinuxResult<isize> {
    Ok(0)
}
