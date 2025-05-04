use alloc::string::String;
use alloc::vec::Vec;
use axerrno::{AxError, LinuxResult};
use axhal::arch::UspaceContext;
use axtask::{TaskExtRef, current};
use starry_core::mm;
use starry_core::task::{TaskExt, current_process, current_process_data};

pub fn sys_execve_impl(path: String, args: Vec<String>, envs: Vec<String>) -> LinuxResult<isize> {
    if current_process().get_threads().len() > 1 {
        // TODO: kill other threads except leader thread
        // because we need to unmap the address space
        error!("[sys_execve] execve is not supported in multi-threaded process");
    }

    debug!("[execve] args = {:?}, envs = {:?}", &args, &envs);

    // clear address space
    let addr_space = &current_process_data().addr_space;
    let mut addr_space = addr_space.lock();
    addr_space.unmap_user_areas()?;
    // TODO: signals
    axhal::arch::flush_tlb(None);

    // load executable binary
    let (entry_point, user_stack_base) =
        mm::load_user_app(&mut addr_space, &args, &envs).map_err(|_| {
            error!("Failed to load app {}", path);
            AxError::NotFound
        })?;
    drop(addr_space);

    // set name and path
    current().set_name(&path);
    *current_process_data().command_line.lock() = args;

    // new user context
    let uctx = UspaceContext::new(entry_point.as_usize(), user_stack_base, 0);
    unsafe {
        uctx.enter_uspace(current().kernel_stack_top().expect("No kernel stack top"));
    }
}
