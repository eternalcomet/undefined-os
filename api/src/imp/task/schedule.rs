use crate::core::time::TimeSpec;
use crate::ptr::{UserInPtr, UserOutPtr};
use crate::utils::task::{task_sleep_interruptable, task_yield};
use axerrno::{LinuxError, LinuxResult};
use syscall_trace::syscall_trace;

#[syscall_trace]
pub fn sys_sched_yield() -> LinuxResult<isize> {
    task_yield();
    Ok(0)
}

#[syscall_trace]
pub fn sys_nanosleep(
    requested: UserInPtr<TimeSpec>,
    remain: UserOutPtr<TimeSpec>,
) -> LinuxResult<isize> {
    let duration = requested.get_as_ref()?.to_duration()?;
    let now = axhal::time::monotonic_time();
    if let Err(LinuxError::EINTR) = task_sleep_interruptable(duration) {
        if let Ok(remain) = remain.get_as_mut_ref() {
            let after = axhal::time::monotonic_time();
            let elapsed = after - now;
            *remain = elapsed.into();
        }
        return Err(LinuxError::EINTR);
    }
    Ok(0)
}
