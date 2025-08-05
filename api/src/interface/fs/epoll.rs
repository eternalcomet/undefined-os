use crate::core::file::epoll::EpollInstance;
use crate::core::file::fd::{FdFlags, FileLike, fd_add};
use crate::ptr::{UserInOutPtr, UserInPtr};
use crate::utils::task::task_yield_interruptable;
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use axhal::time::wall_time;
use core::ffi::c_int;
use core::time::Duration;
use linux_raw_sys::general::epoll_event;
use syscall_trace::syscall_trace;

/// Creates a new epoll instance.
///
/// It returns a file descriptor referring to the new epoll instance.
#[syscall_trace]
pub fn sys_epoll_create(size: c_int) -> LinuxResult<isize> {
    if size < 0 {
        return Err(LinuxError::EINVAL);
    }
    let epoll_instance = EpollInstance::new(0);
    Ok(fd_add(Arc::new(epoll_instance), FdFlags::empty())? as _)
}

/// Control interface for an epoll file descriptor
#[syscall_trace]
pub fn sys_epoll_ctl(
    epfd: c_int,
    op: c_int,
    fd: c_int,
    event: UserInPtr<epoll_event>,
) -> LinuxResult<isize> {
    let event = event.get_as_ref()?;
    let instance = EpollInstance::from_fd(epfd)?;
    Ok(instance.control(op as _, fd as _, event)? as _)
}

/// Waits for events on the epoll instance referred to by the file descriptor epfd.
#[syscall_trace]
pub fn sys_epoll_wait(
    epfd: c_int,
    events: UserInOutPtr<epoll_event>,
    maxevents: c_int,
    timeout: c_int,
) -> LinuxResult<isize> {
    if maxevents <= 0 {
        return Err(LinuxError::EINVAL);
    }
    let events = events.get_as_mut_slice(maxevents as _)?;
    let deadline =
        (!timeout.is_negative()).then(|| wall_time() + Duration::from_millis(timeout as u64));
    let epoll_instance = EpollInstance::from_fd(epfd)?;
    loop {
        #[cfg(feature = "net")]
        axnet::poll_interfaces();
        let events_num = epoll_instance.poll_all(events)?;
        if events_num > 0 {
            return Ok(events_num as _);
        }

        if deadline.is_some_and(|ddl| wall_time() >= ddl) {
            debug!("[epoll_wait] timeout!");
            return Ok(0);
        }
        task_yield_interruptable()?;
    }
}
