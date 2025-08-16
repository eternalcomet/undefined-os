use crate::core::time::TimeSpec;
use crate::ptr::{PtrWrapper, UserOutPtr, UserPtr, nullable};
use arceos_posix_api::{self as api, ctypes::timeval};
use axerrno::{LinuxError, LinuxResult};
use axhal::time::{NANOS_PER_SEC, monotonic_time_nanos, nanos_to_ticks};
use linux_raw_sys::general::timezone;
use starry_core::{ctypes::Tms, task::time_stat_output};
use syscall_trace::syscall_trace;

pub fn sys_clock_gettime(clock_id: i32, tp: UserPtr<api::ctypes::timespec>) -> LinuxResult<isize> {
    unsafe { Ok(api::sys_clock_gettime(clock_id, tp.get()?) as _) }
}

#[syscall_trace]
pub fn sys_get_time_of_day(
    ts: UserOutPtr<timeval>,
    tz: UserOutPtr<timezone>,
) -> LinuxResult<isize> {
    nullable!(tz.get())?;
    unsafe { Ok(api::sys_get_time_of_day(ts.get()?) as _) }
}

pub fn sys_times(tms: UserPtr<Tms>) -> LinuxResult<isize> {
    let (_, utime_us, _, stime_us) = time_stat_output();
    unsafe {
        *tms.get()? = Tms {
            tms_utime: utime_us,
            tms_stime: stime_us,
            tms_cutime: utime_us,
            tms_cstime: stime_us,
        }
    }
    Ok(nanos_to_ticks(monotonic_time_nanos()) as _)
}

#[syscall_trace]
pub fn sys_clock_getres(clock_id: i32, resolution: UserOutPtr<TimeSpec>) -> LinuxResult<isize> {
    if clock_id < 0 {
        return Err(LinuxError::EINVAL);
    }
    if let Some(resolution) = nullable!(resolution.get_as_mut_ref())? {
        *resolution = TimeSpec {
            seconds: 0,
            nanoseconds: NANOS_PER_SEC as i64 / axconfig::devices::TIMER_FREQUENCY as i64,
        };
    }
    Ok(0)
}
