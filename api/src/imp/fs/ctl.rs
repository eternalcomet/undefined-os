use core::ffi::{c_char, c_void};

use axerrno::{LinuxError, LinuxResult};
use axfs::fops::DirEntry;
use macro_rules_attribute::apply;

use crate::{
    ptr::{PtrWrapper, UserConstPtr, UserPtr},
    syscall_instrument,
};

/// The ioctl() system call manipulates the underlying device parameters
/// of special files.
///
/// # Arguments
/// * `fd` - The file descriptor
/// * `op` - The request code. It is of type unsigned long in glibc and BSD,
///   and of type int in musl and other UNIX systems.
/// * `argp` - The argument to the request. It is a pointer to a memory location
#[apply(syscall_instrument)]
pub fn sys_ioctl(_fd: i32, _op: usize, _argp: UserPtr<c_void>) -> LinuxResult<isize> {
    warn!("Unimplemented syscall: SYS_IOCTL");
    Ok(0)
}

pub fn sys_chdir(path: UserConstPtr<c_char>) -> LinuxResult<isize> {
    let path = path.get_as_str()?;
    axfs::api::set_current_dir(path).map(|_| 0).map_err(|err| {
        warn!("Failed to change directory: {err:?}");
        err.into()
    })
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct DirEnt {
    d_ino: u64,
    d_off: i64,
    d_reclen: u16,
    d_type: u8,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum FileType {
    Unknown = 0,
    Fifo = 1,
    Chr = 2,
    Dir = 4,
    Blk = 6,
    Reg = 8,
    Lnk = 10,
    Socket = 12,
    Wht = 14,
}

impl From<axfs::api::FileType> for FileType {
    fn from(ft: axfs::api::FileType) -> Self {
        match ft {
            ft if ft.is_dir() => FileType::Dir,
            ft if ft.is_file() => FileType::Reg,
            _ => FileType::Unknown,
        }
    }
}

impl DirEnt {
    const FIXED_SIZE: usize =
        size_of::<u64>() + size_of::<i64>() + size_of::<u16>() + size_of::<u8>();

    fn new(ino: u64, off: i64, reclen: usize, file_type: FileType) -> Self {
        Self {
            d_ino: ino,
            d_off: off,
            d_reclen: reclen as u16,
            d_type: file_type as u8,
        }
    }
}

pub fn sys_getdents64(fd: i32, buf: UserPtr<c_void>, len: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_bytes(len)?;

    if len < DirEnt::FIXED_SIZE {
        warn!("Buffer size too small: {len}");
        return Err(LinuxError::EINVAL);
    }

    let directory = arceos_posix_api::Directory::from_fd(fd)?;
    let directory = directory.inner();
    let user_buffer = buf as *mut u8;
    let mut current_offset: usize = 0;
    loop {
        // read directory entries into buffer
        if current_offset + DirEnt::FIXED_SIZE + 2 > len {
            // there is no enough space for another entry
            break;
        }
        // we don't know how many entries can be contained by the buf provided by user
        // so we make the buffer small(1)
        let mut entry_buffer = [DirEntry::default()];
        let count = directory.lock().read_dir(&mut entry_buffer)?;
        if count == 0 {
            // no more entries
            break;
        }
        let entry = &entry_buffer[0];
        let name = entry.name_as_bytes();
        let entry_type = FileType::from(entry.entry_type());
        let entry_length = DirEnt::FIXED_SIZE + name.len() + 1;
        if current_offset + entry_length > len {
            // check again
            // there is no enough space for another entry
            break;
        }

        let user_dir_entry = DirEnt::new(
            1,
            (current_offset + entry_length) as _,
            entry_length,
            entry_type,
        );
        unsafe {
            // let pointer be *mut u8 so that the offset can be calculated
            let entry_ptr = user_buffer.add(current_offset);
            (entry_ptr as *mut DirEnt).write(user_dir_entry);
            let name_ptr = entry_ptr.add(DirEnt::FIXED_SIZE);
            core::ptr::copy_nonoverlapping(name.as_ptr(), name_ptr, name.len());
            *name_ptr.add(name.len()) = 0; // null-terminate the name
        }

        current_offset += entry_length;
    }
    Ok(current_offset as _)
}

/// create a link from new_path to old_path
/// old_path: old file path
/// new_path: new file path
/// flags: link flags
/// return value: return 0 when success, else return -1.
pub fn sys_linkat(
    old_dirfd: i32,
    old_path: UserConstPtr<c_char>,
    new_dirfd: i32,
    new_path: UserConstPtr<c_char>,
    flags: i32,
) -> LinuxResult<isize> {
    let old_path = old_path.get_as_null_terminated()?;
    let new_path = new_path.get_as_null_terminated()?;

    if flags != 0 {
        warn!("Unsupported flags: {flags}");
    }

    // handle old path
    arceos_posix_api::handle_file_path(old_dirfd as isize, Some(old_path.as_ptr() as _), false)
        .inspect_err(|err| warn!("Failed to convert new path: {err:?}"))
        .and_then(|old_path| {
            //handle new path
            arceos_posix_api::handle_file_path(
                new_dirfd as isize,
                Some(new_path.as_ptr() as _),
                false,
            )
            .inspect_err(|err| warn!("Failed to convert new path: {err:?}"))
            .map(|new_path| (old_path, new_path))
        })
        .and_then(|(old_path, new_path)| {
            arceos_posix_api::HARDLINK_MANAGER
                .create_link(&new_path, &old_path)
                .inspect_err(|err| warn!("Failed to create link: {err:?}"))
                .map_err(Into::into)
        })
        .map(|_| 0)
        .map_err(|err| err.into())
}

pub fn sys_getcwd(buf: UserPtr<c_char>, size: usize) -> LinuxResult<isize> {
    Ok(arceos_posix_api::sys_getcwd(buf.get_as_null_terminated()?.as_ptr() as _, size) as _)
}
