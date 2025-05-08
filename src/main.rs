#![no_std]
#![no_main]
#![doc = include_str!("../README.md")]

extern crate alloc;
#[macro_use]
extern crate axlog;

mod mm;
mod syscall;

use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use starry_core::entry::run_user_app;

#[unsafe(no_mangle)]
fn main() {
    let testcases = option_env!("AX_TESTCASES_LIST")
        .unwrap_or_else(|| "Please specify the testcases list by making user_apps")
        .split(',')
        .filter(|&x| !x.is_empty());

    for testcase in testcases {
        let testcase = testcase.trim();
        if testcase.is_empty() {
            continue;
        }
        // sh mode
        let args = vec!["/musl/busybox", "sh", "-c", testcase];
        // direct mode
        // let args = testcase.split(" ");
        let args: Vec<String> = args.into_iter().map(String::from).collect();

        info!("[task manager] Running user task: {}", testcase);

        let envs = vec![
            "PATH=/bin".to_string(),
            "LD_LIBRARY_PATH=/glibc/lib/:/musl/lib/".to_string(),
            // "LD_DEBUG=all".to_string(),
        ];

        let exit_code = run_user_app(&args, &envs);
        info!(
            "[task manager] User task {} exited with code: {:?}",
            testcase, exit_code
        );
    }
}
