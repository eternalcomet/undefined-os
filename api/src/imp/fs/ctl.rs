use crate::core::file::dir::Directory;
use crate::core::file::fd::{FileDescriptor, FileLike, fd_lookup};
use crate::core::time::TimeSpec;
use crate::ptr::{PtrWrapper, UserInOutPtr, UserInPtr, UserOutPtr, nullable};
use crate::utils::dev::get_device_by_fd;
use crate::utils::path::{ResolveFlags, change_current_dir, get_fs_context, resolve_path_at};
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::resolve_path;
use axhal::time::wall_time;
use core::ffi::{c_char, c_void};
use core::mem::offset_of;
use core::time::Duration;
use linux_raw_sys::general::{
    POSIX_FADV_DONTNEED, POSIX_FADV_NOREUSE, POSIX_FADV_NORMAL, POSIX_FADV_RANDOM,
    POSIX_FADV_SEQUENTIAL, POSIX_FADV_WILLNEED, UTIME_NOW, UTIME_OMIT,
};
use starry_core::task::current_process;
use syscall_trace::syscall_trace;
use undefined_process::Pid;
use undefined_vfs::types::{MetadataUpdate, NodeType};

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
///
/// # Arguments
/// * `fd` - The file descriptor
/// * `op` - The request code. It is of type unsigned long in glibc and BSD,
///   and of type int in musl and other UNIX systems.
/// * `argp` - The argument to the request. It is a pointer to a memory location
#[syscall_trace]
pub fn sys_ioctl(fd: i32, op: usize, arg: UserInOutPtr<u8>) -> LinuxResult<isize> {
    if let Some(device) = get_device_by_fd(fd) {
        // If the file descriptor is a device, we can call its ioctl method
        return device.ops().ioctl(op as u32, arg.address().as_usize());
    }
    const TIOCGPGRP: usize = 0x540F;
    match op {
        TIOCGPGRP => {
            let process_group = current_process().get_group().get_pgid();
            let result = arg.clone().cast::<Pid>();
            let arg = result.get_as_mut_ref()?;
            *arg = process_group;
        }
        _ => {
            warn!("Unimplemented syscall: SYS_IOCTL");
        }
    }
    Ok(0)
}

#[syscall_trace]
pub fn sys_chdir(path: UserInPtr<c_char>) -> LinuxResult<isize> {
    let path = path.get_as_str()?;
    let context = get_fs_context();
    let path = resolve_path(&context, &path, &mut 0, true)?;
    drop(context); // drop the context to avoid holding the lock longer than necessary
    change_current_dir(path)?;
    Ok(0)
}

#[syscall_trace]
pub fn sys_fchdir(fd: FileDescriptor) -> LinuxResult<isize> {
    let file_like = fd_lookup(fd)?;
    let location = file_like.location().ok_or(LinuxError::ENOTDIR)?;
    change_current_dir(location)?;
    Ok(0)
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEnt {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
    d_name: [u8; 0],
}

#[syscall_trace]
pub fn sys_getdents64(fd: i32, buf: UserOutPtr<u8>, buf_size: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_mut_slice(buf_size)?;
    let directory = Directory::from_fd(fd)?;
    let mut directory = directory.inner();
    let mut entry_index = directory.get_position();
    let mut buf_offset = 0;

    directory.location().read_dir(
        entry_index,
        &mut |name: &str, ino: u64, node_type: NodeType, offset: u64| {
            let name = name.as_bytes();
            // 这里使用offset_of!宏而不是sizeof来计算d_name字段的偏移量的原因是后者会考虑对齐问题
            let write_len = offset_of!(DirEnt, d_name) + name.len() + 1;
            let write_len = write_len.next_multiple_of(align_of::<DirEnt>());
            if buf_offset + write_len > buf_size {
                // not enough space in the buffer
                return false;
            }
            unsafe {
                let entry_ptr = buf.as_mut_ptr().add(buf_offset);
                let buf_entry_ref = &mut *(entry_ptr.cast::<DirEnt>());
                buf_entry_ref.d_ino = ino;
                buf_entry_ref.d_off = offset as _;
                buf_entry_ref.d_reclen = write_len as _;
                buf_entry_ref.d_type = node_type as _;
                let name_ptr = entry_ptr.add(offset_of!(DirEnt, d_name));
                name_ptr.copy_from_nonoverlapping(name.as_ptr(), name.len());
                *name_ptr.add(name.len()) = 0; // null-terminate the name
            }

            buf_offset += write_len;
            entry_index = offset;
            true
        },
    )?;
    directory.set_position(entry_index);
    Ok(buf_offset as _)
}

#[syscall_trace]
pub fn sys_utimensat(
    dir_fd: i32,
    path: UserInPtr<c_char>,
    times: UserInPtr<TimeSpec>,
    flags: u32,
) -> LinuxResult<isize> {
    fn into_duration(time: TimeSpec) -> Option<Duration> {
        match time.nanoseconds as u32 {
            UTIME_OMIT => None,
            UTIME_NOW => Some(wall_time()),
            _ => Some(Duration::from(time)),
        }
    }
    let path = nullable!(path.get_as_str())?;
    let resolve_flags = ResolveFlags::from_bits_truncate(flags);
    let resolve = resolve_path_at(dir_fd, path, resolve_flags)?;
    let location = resolve.location().ok_or(LinuxError::EINVAL)?;
    let (atime, mtime) = if times.is_null() {
        (Some(wall_time()), Some(wall_time()))
    } else {
        let times = times.get_as_slice(2)?;
        (into_duration(times[0]), into_duration(times[1]))
    };
    location.update_metadata(MetadataUpdate {
        atime,
        mtime,
        ..Default::default()
    })?;
    Ok(0)
}

#[syscall_trace]
pub fn sys_fadvise(
    fd: FileDescriptor,
    offset: isize,
    size: isize,
    advice: u32,
) -> LinuxResult<isize> {
    let _file_like = fd_lookup(fd)?;
    match advice {
        POSIX_FADV_NORMAL
        | POSIX_FADV_SEQUENTIAL
        | POSIX_FADV_RANDOM
        | POSIX_FADV_NOREUSE
        | POSIX_FADV_WILLNEED
        | POSIX_FADV_DONTNEED => {}
        _ => return Err(LinuxError::EINVAL),
    }
    Ok(0)
}
