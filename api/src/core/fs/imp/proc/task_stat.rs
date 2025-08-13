use alloc::{borrow::ToOwned, fmt, string::String};
use axerrno::{LinuxError, LinuxResult};
use linux_raw_sys::general::SIGCHLD;
use starry_core::process::get_process_data;
use undefined_process::Pid;
use undefined_process::process::get_process;

/// Represents the `/proc/[pid]/stat` file.
///
/// See ['https://man7.org/linux/man-pages/man5/proc_pid_stat.5.html'] for details.
#[allow(missing_docs)]
#[derive(Default)]
pub struct TaskStat {
    pub pid: u32,
    pub comm: String,
    pub state: char,
    pub ppid: u32,
    pub pgrp: u32,
    pub session: u32,
    pub tty_nr: u32,
    pub tpgid: u32,
    pub flags: u32,
    pub minflt: u64,
    pub cminflt: u64,
    pub majflt: u64,
    pub cmajflt: u64,
    pub utime: u64,
    pub stime: u64,
    pub cutime: u64,
    pub cstime: u64,
    pub priority: u32,
    pub nice: u32,
    pub num_threads: u32,
    pub itrealvalue: u32,
    pub starttime: u64,
    pub vsize: u64,
    pub rss: i64,
    pub rsslim: u64,
    pub start_code: u64,
    pub end_code: u64,
    pub start_stack: u64,
    pub kstk_esp: u64,
    pub kstk_eip: u64,
    pub signal: u32,
    pub blocked: u32,
    pub sigignore: u32,
    pub sigcatch: u32,
    pub wchan: u64,
    pub nswap: u64,
    pub cnswap: u64,
    pub exit_signal: u8,
    pub processor: u32,
    pub rt_priority: u32,
    pub policy: u32,
    pub delayacct_blkio_ticks: u64,
    pub guest_time: u64,
    pub cguest_time: u64,
    pub start_data: u64,
    pub end_data: u64,
    pub start_brk: u64,
    pub arg_start: u64,
    pub arg_end: u64,
    pub env_start: u64,
    pub env_end: u64,
    pub exit_code: i32,
}

impl TaskStat {
    pub fn from_process(pid: Pid) -> LinuxResult<Self> {
        let process = get_process(pid).ok_or(LinuxError::ENOENT)?;
        let process_data = get_process_data(pid).ok_or(LinuxError::ENOENT)?;

        let comm = process_data
            .command_line
            .lock()
            .first()
            .cloned()
            .unwrap_or_default();
        // TODO: get state from task scheduler
        let state = if process.is_zombie() { 'Z' } else { 'R' };
        let ppid = process.get_parent().map_or(0, |p| p.get_pid());
        let pgrp = process.get_group().get_pgid();
        let session = process.get_session().get_sid();
        let num_threads = process.get_threads().len() as _;
        let (exit_signal, exit_code) = if process.is_zombie() {
            (
                (process.get_exit_code() & 0xff) as u8,
                process.get_exit_code() >> 8,
            )
        } else {
            (SIGCHLD as u8, 0)
        };
        Ok(Self {
            pid,
            comm: comm.to_owned(),
            state,
            ppid,
            pgrp,
            session,
            num_threads,
            exit_signal,
            exit_code,
            ..Default::default()
        })
    }
}

impl fmt::Display for TaskStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            pid,
            comm,
            state,
            ppid,
            pgrp,
            session,
            tty_nr,
            tpgid,
            flags,
            minflt,
            cminflt,
            majflt,
            cmajflt,
            utime,
            stime,
            cutime,
            cstime,
            priority,
            nice,
            num_threads,
            itrealvalue,
            starttime,
            vsize,
            rss,
            rsslim,
            start_code,
            end_code,
            start_stack,
            kstk_esp,
            kstk_eip,
            signal,
            blocked,
            sigignore,
            sigcatch,
            wchan,
            nswap,
            cnswap,
            exit_signal,
            processor,
            rt_priority,
            policy,
            delayacct_blkio_ticks,
            guest_time,
            cguest_time,
            start_data,
            end_data,
            start_brk,
            arg_start,
            arg_end,
            env_start,
            env_end,
            exit_code,
        } = self;
        writeln!(
            f,
            "{pid} ({comm}) {state} {ppid} {pgrp} {session} {tty_nr} {tpgid} {flags} {minflt} \
             {cminflt} {majflt} {cmajflt} {utime} {stime} {cutime} {cstime} {priority} {nice} \
             {num_threads} {itrealvalue} {starttime} {vsize} {rss} {rsslim} {start_code} \
             {end_code} {start_stack} {kstk_esp} {kstk_eip} {signal} {blocked} {sigignore} \
             {sigcatch} {wchan} {nswap} {cnswap} {exit_signal} {processor} {rt_priority} {policy} \
             {delayacct_blkio_ticks} {guest_time} {cguest_time} {start_data} {end_data} \
             {start_brk} {arg_start} {arg_end} {env_start} {env_end} {exit_code}",
        )
    }
}
