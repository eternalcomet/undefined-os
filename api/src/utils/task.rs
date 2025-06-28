use crate::imp::task::signal::check_signals;
use axerrno::{LinuxError, LinuxResult};
use axhal::arch::TrapFrame;
use core::time::Duration;
use percpu::def_percpu;

pub fn task_yield() {
    axtask::yield_now();
}

/// A thread-local storage for the trap frame.
#[def_percpu]
static mut TRAP_FRAME: *mut TrapFrame = core::ptr::null_mut();

pub fn get_trap_frame() -> Option<&'static mut TrapFrame> {
    let ptr = unsafe { TRAP_FRAME.current_ptr() };
    let tf = unsafe { *ptr };
    if tf.is_null() {
        None
    } else {
        Some(unsafe { &mut *tf })
    }
}

pub fn set_trap_frame(tf: *mut TrapFrame) {
    TRAP_FRAME.with_current(|current| {
        *current = tf;
    });
}

/// Provides a signal-interruptible yield function for tasks.
pub fn task_yield_interruptable() -> LinuxResult {
    // debug!("yield_interruptable");
    axtask::yield_now();
    // debug!("yield resumed");
    // TODO: check signals, if any are pending, return LinuxError::EINTR
    let tf = get_trap_frame();
    if let Some(tf) = tf {
        // If we have a trap frame, we can check for signals
        // This is where we would handle the signal logic
        if check_signals(tf, None) {
            error!("syscall interrupted by signal");
            return Err(LinuxError::EINTR);
        }
    }
    Ok(())
}
pub fn task_sleep(duration: Duration) {
    axtask::sleep(duration);
}

pub fn task_sleep_interruptable(duration: Duration) -> LinuxResult {
    axtask::sleep(duration);
    // TODO: check signals, if any are pending, return LinuxError::EINTR
    Ok(())
}
