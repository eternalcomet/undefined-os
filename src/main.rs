#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[macro_use]
extern crate axlog;

pub mod entry;
mod mm;
mod syscall;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use axdisplay::get_main_display;
use axfs_ng::api::FS_CONTEXT;

use axdriver_display::DisplayDriverOps;
use axhal::mem::VirtAddr;
use axmm::kernel_aspace;
use entry::run_user_app;
use undefined_os_api::core::file::fd::{FD_TABLE, FdTable};
use undefined_os_api::core::fs::mount::mount_all;

#[unsafe(no_mangle)]
fn main() {
    let root_dir = axfs_ng::api::FS_CONTEXT.lock().root_dir.clone();
    FS_CONTEXT.lock().change_root(root_dir).unwrap();
    FD_TABLE.init_new(FdTable::new());
    mount_all().expect("Mounting all filesystems failed");

    let command = include_str!(env!("AX_TESTCASES_FILE"));
    let args = vec!["/usr/bin/bash", "-c", command];
    let args: Vec<String> = args.into_iter().map(String::from).collect();

    let envs = vec![
        "PATH=/bin".to_string(),
        // "LD_LIBRARY_PATH=/lib:/lib64".to_string(),
        // "LD_DEBUG=all".to_string(),
    ];

    let exit_code = run_user_app(&args, &envs);
    info!("[task manager] Shell exited with code: {:?}", exit_code);
}
