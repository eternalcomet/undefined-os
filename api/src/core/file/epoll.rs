//! `epoll` implementation.
//!
//! TODO: do not support `EPOLLET` flag

use alloc::collections::BTreeMap;
use alloc::collections::btree_map::Entry;
use alloc::sync::Arc;
use core::{ffi::c_int, time::Duration};

use crate::core::file::fd::{FdFlags, FileLike, fd_add, fd_lookup};
use crate::utils::task::task_yield_interruptable;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::FileFlags;
use axhal::time::wall_time;
use axsync::Mutex;
use linux_raw_sys::general::{
    EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, EPOLLERR, EPOLLIN, EPOLLOUT, epoll_event,
};
use undefined_vfs::types::{Metadata, NodePermission};

pub struct EpollInstance {
    events: Mutex<BTreeMap<usize, epoll_event>>,
}

impl EpollInstance {
    // TODO: parse flags
    pub fn new(_flags: usize) -> Self {
        Self {
            events: Mutex::new(BTreeMap::new()),
        }
    }

    pub(crate) fn control(&self, op: usize, fd: usize, event: &epoll_event) -> LinuxResult<usize> {
        fd_lookup(fd as c_int)?;

        match op as u32 {
            EPOLL_CTL_ADD => {
                if let Entry::Vacant(e) = self.events.lock().entry(fd) {
                    e.insert(*event);
                } else {
                    return Err(LinuxError::EEXIST);
                }
            }
            EPOLL_CTL_MOD => {
                let mut events = self.events.lock();
                if let Entry::Occupied(mut ocp) = events.entry(fd) {
                    ocp.insert(*event);
                } else {
                    return Err(LinuxError::ENOENT);
                }
            }
            EPOLL_CTL_DEL => {
                let mut events = self.events.lock();
                if let Entry::Occupied(ocp) = events.entry(fd) {
                    ocp.remove_entry();
                } else {
                    return Err(LinuxError::ENOENT);
                }
            }
            _ => {
                return Err(LinuxError::EINVAL);
            }
        }
        Ok(0)
    }

    pub(crate) fn poll_all(&self, events: &mut [epoll_event]) -> LinuxResult<usize> {
        let ready_list = self.events.lock();
        let mut events_num = 0;

        for (infd, ev) in ready_list.iter() {
            match fd_lookup(*infd as c_int)?.poll() {
                Err(_) => {
                    if (ev.events & EPOLLERR) != 0 {
                        events[events_num].events = EPOLLERR;
                        events[events_num].data = ev.data;
                        events_num += 1;
                    }
                }
                Ok(state) => {
                    if state.readable && (ev.events & EPOLLIN != 0) {
                        events[events_num].events = EPOLLIN;
                        events[events_num].data = ev.data;
                        events_num += 1;
                    }

                    if state.writable && (ev.events & EPOLLOUT != 0) {
                        events[events_num].events = EPOLLOUT;
                        events[events_num].data = ev.data;
                        events_num += 1;
                    }
                }
            }
        }
        Ok(events_num)
    }
}

impl FileLike for EpollInstance {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::ENOSYS)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::ENOSYS)
    }

    fn status(&self) -> LinuxResult<Metadata> {
        Ok(Metadata {
            inode: 999,
            n_link: 1,
            mode: NodePermission::OWNER_READ | NodePermission::OWNER_WRITE,
            ..Default::default()
        })
    }

    fn poll(&self) -> LinuxResult<axio::PollState> {
        Err(LinuxError::ENOSYS)
    }

    fn get_flags(&self) -> FileFlags {
        warn!("get_flags is not supported for epoll instance");
        FileFlags::empty()
    }

    fn set_flags(&self, flags: FileFlags) {
        warn!("set_flags is not supported for epoll instance: {flags:?}");
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }
}
