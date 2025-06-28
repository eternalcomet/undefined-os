use arceos_posix_api::config::plat::PHYS_VIRT_OFFSET;
use axerrno::LinuxError;
use axhal::mem::VirtAddr;
use axhal::paging::MappingFlags;
use axhal::trap::{PAGE_FAULT, register_trap_handler};
use axsignal::{SignalInfo, Signo};
use linux_raw_sys::general::SI_KERNEL;
use starry_api::imp::task::signal::send_signal_process;
use starry_api::imp::task::sys_exit_impl;
use starry_core::mm::is_accessing_user_memory;
use starry_core::task::{current_process, current_process_data};

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(vaddr: VirtAddr, access_flags: MappingFlags, is_user: bool) -> bool {
    if vaddr.as_usize() > PHYS_VIRT_OFFSET {
        error!(
            "Kernel page fault at {:#x}, access_flags: {:#x?}",
            vaddr, access_flags
        );
        return false;
    }
    if !is_user && !is_accessing_user_memory() {
        warn!(
            "Maybe we are accessing user memory in kernel, and triggered a page fault at {:#x}, access_flags: {:#x?}",
            vaddr, access_flags
        );
    }

    if !current_process_data()
        .addr_space
        .lock()
        .handle_page_fault(vaddr, access_flags)
    {
        warn!(
            "{}: segmentation fault at {:#x}, send SIGSEGV.",
            axtask::current().id_name(),
            vaddr
        );
        if send_signal_process(
            current_process().get_pid(),
            SignalInfo::new(Signo::SIGSEGV, SI_KERNEL as _),
        )
        .is_err()
        {
            error!("send SIGSEGV failed");
            sys_exit_impl(LinuxError::EFAULT as _, 0,false);
        }
    }
    true
}
