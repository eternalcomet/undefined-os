use crate::core::file::fd::{FileDescriptor, fd_lookup, file_like_as};
use crate::core::file::file::File;
use crate::core::file::pipe::Pipe;
use crate::imp::fs::{
    sys_copy_file_range_impl, sys_pread_impl, sys_pwrite_impl, sys_read_impl, sys_truncate_impl,
    sys_write_impl,
};
use crate::ptr::{UserInOutPtr, UserInPtr, UserOutPtr, nullable};
use crate::utils::path::resolve_path_at_cwd;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::FileFlags;
use axio::SeekFrom;
use core::ffi::{c_char, c_int, c_long, c_uint};
use syscall_trace::syscall_trace;

#[syscall_trace]
pub fn sys_truncate(path: UserInPtr<c_char>, length: c_long) -> LinuxResult<isize> {
    // get params
    let path = nullable!(path.get_as_str())?;

    // open file
    let location = resolve_path_at_cwd(path)?;
    let file = File::from_location(location, FileFlags::WRITE);
    sys_truncate_impl(&file, length as _)
}

#[syscall_trace]
pub fn sys_ftruncate(fd: c_int, length: c_long) -> LinuxResult<isize> {
    let file = File::from_fd(fd)?;
    sys_truncate_impl(&file, length as _)
}

#[syscall_trace]
pub fn sys_lseek(fd: c_int, offset: isize, whence: c_int) -> LinuxResult<isize> {
    let pos = match whence {
        0 => SeekFrom::Start(offset as _),
        1 => SeekFrom::Current(offset as _),
        2 => SeekFrom::End(offset as _),
        _ => return Err(LinuxError::EINVAL),
    };
    let file_like = fd_lookup(fd)?.into_any();
    if let Some(file) = file_like.downcast_ref::<File>() {
        let offset = file.inner().seek(pos)?;
        Ok(offset as _)
    } else {
        // For pipes, sockets, FIFOs, they are not seekable, so we return an error.
        Err(LinuxError::ESPIPE)
    }
}

#[syscall_trace]
pub fn sys_read(fd: i32, buf: UserOutPtr<u8>, count: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_mut_slice(count)?;
    let file_like = fd_lookup(fd as _)?;
    sys_read_impl(&*file_like, buf)
}

#[syscall_trace]
pub fn sys_write(fd: i32, buf: UserInPtr<u8>, count: usize) -> LinuxResult<isize> {
    let buf = buf.get_as_slice(count)?;
    let file_like = fd_lookup(fd as _)?;
    sys_write_impl(&*file_like, buf)
}

/// structure for ctype `iovec`, used in readv/writev
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct UserIoVector {
    pub base_addr: *mut u8,
    pub length: isize,
}

#[syscall_trace]
pub fn sys_readv(
    fd: i32,
    io_vectors: UserInPtr<UserIoVector>,
    io_count: i32,
) -> LinuxResult<isize> {
    if io_count < 0 {
        return Err(LinuxError::EINVAL);
    }
    let io_vectors = io_vectors.get_as_slice(io_count as usize)?;
    let file_like = fd_lookup(fd as _)?;
    let mut total_read: isize = 0;
    for io in io_vectors {
        if io.length < 0 {
            return Err(LinuxError::EINVAL);
        }
        let buf_ptr = UserOutPtr::<u8>::from(io.base_addr as usize);
        let buf = buf_ptr.get_as_mut_slice(io.length as _)?;
        let read_len = sys_read_impl(&*file_like, buf)?;
        total_read += read_len;
    }
    Ok(total_read)
}

#[syscall_trace]
pub fn sys_writev(
    fd: i32,
    io_vectors: UserInPtr<UserIoVector>,
    io_count: i32,
) -> LinuxResult<isize> {
    if io_count < 0 {
        return Err(LinuxError::EINVAL);
    }
    let io_vectors = io_vectors.get_as_slice(io_count as usize)?;
    let file_like = fd_lookup(fd as _)?;
    let mut total_written: isize = 0;
    for io in io_vectors {
        if io.length < 0 {
            return Err(LinuxError::EINVAL);
        }
        let buf_ptr = UserInPtr::<u8>::from(io.base_addr as usize);
        let buf = buf_ptr.get_as_slice(io.length as _)?;
        let write_len = sys_write_impl(&*file_like, buf)?;
        total_written += write_len;
    }
    Ok(total_written)
}

#[syscall_trace]
pub fn sys_pread64(
    fd: c_int,
    buf: UserOutPtr<u8>,
    count: usize,
    offset: usize,
) -> LinuxResult<isize> {
    let buf = buf.get_as_mut_slice(count)?;
    let file = File::from_fd(fd)?;
    sys_pread_impl(&file, buf, offset as _)
}

#[syscall_trace]
pub fn sys_pwrite64(
    fd: c_int,
    buf: UserInPtr<u8>,
    count: usize,
    offset: usize,
) -> LinuxResult<isize> {
    let buf = buf.get_as_slice(count)?;
    let file = File::from_fd(fd)?;
    sys_pwrite_impl(&file, buf, offset as _)
}

#[syscall_trace]
pub fn sys_sendfile(
    out_fd: FileDescriptor,
    in_fd: FileDescriptor,
    offset: UserInOutPtr<usize>,
    count: usize,
) -> LinuxResult<isize> {
    let in_file = fd_lookup(in_fd)?;
    let out_file = fd_lookup(out_fd)?;
    let offset = nullable!(offset.get_as_mut_ref())?;
    let transferred = sys_copy_file_range_impl(in_file, None, out_file, offset, count)?;
    Ok(transferred as _)
}

#[syscall_trace]
pub fn sys_copy_file_range(
    in_fd: FileDescriptor,
    in_offset: UserInOutPtr<usize>,
    out_fd: FileDescriptor,
    out_offset: UserInOutPtr<usize>,
    count: usize,
    _flags: c_uint,
) -> LinuxResult<isize> {
    let in_file = fd_lookup(in_fd)?;
    let out_file = fd_lookup(out_fd)?;
    let in_offset = nullable!(in_offset.get_as_mut_ref())?;
    let out_offset = nullable!(out_offset.get_as_mut_ref())?;
    // TODO: flags
    // TODO: more checks
    // 该系统调用仅支持常规文件，但我们的实现允许在管道和套接字等文件上使用。
    // 此外，我们缺少对in_fd和out_fd相同时且区域可能重叠的处理。
    let transferred = sys_copy_file_range_impl(in_file, in_offset, out_file, out_offset, count)?;
    Ok(transferred as _)
}

#[syscall_trace]
pub fn sys_splice(
    in_fd: FileDescriptor,
    in_offset: UserInOutPtr<usize>,
    out_fd: FileDescriptor,
    out_offset: UserInOutPtr<usize>,
    count: usize,
    _flags: c_uint,
) -> LinuxResult<isize> {
    // the type of offset is `off_t *` in fact, so we should check if the value is negative.
    if let Ok(off) = in_offset.get_as_ref()
        && *off > isize::MAX as _
    {
        return Err(LinuxError::EINVAL);
    }
    if let Ok(off) = out_offset.get_as_ref()
        && *off > isize::MAX as _
    {
        return Err(LinuxError::EINVAL);
    }
    let in_file = fd_lookup(in_fd)?;
    let out_file = fd_lookup(out_fd)?;
    let in_offset = nullable!(in_offset.get_as_mut_ref())?;
    let out_offset = nullable!(out_offset.get_as_mut_ref())?;

    // One of the file descriptors must refer to a pipe.
    if file_like_as::<Pipe>(in_file.clone()).is_none()
        && file_like_as::<Pipe>(out_file.clone()).is_none()
    {
        return Err(LinuxError::EINVAL);
    }
    let transferred = sys_copy_file_range_impl(in_file, in_offset, out_file, out_offset, count)?;
    Ok(transferred as _)
}

#[syscall_trace]
pub fn sys_preadv(
    fd: i32,
    io_vectors: UserInPtr<UserIoVector>,
    io_count: i32,
    offset: isize,
) -> LinuxResult<isize> {
    if io_count < 0 {
        return Err(LinuxError::EINVAL);
    }
    let io_vectors = io_vectors.get_as_slice(io_count as usize)?;
    let file = File::from_fd(fd as _)?;
    let mut total_read: isize = 0;
    for io in io_vectors {
        if io.length < 0 {
            return Err(LinuxError::EINVAL);
        }
        let buf_ptr = UserOutPtr::<u8>::from(io.base_addr as usize);
        let buf = buf_ptr.get_as_mut_slice(io.length as _)?;
        let read_len = sys_pread_impl(&file, buf, offset)?;
        total_read += read_len;
    }
    Ok(total_read)
}

#[syscall_trace]
pub fn sys_pwritev(
    fd: i32,
    io_vectors: UserInPtr<UserIoVector>,
    io_count: i32,
    offset: isize,
) -> LinuxResult<isize> {
    if io_count < 0 {
        return Err(LinuxError::EINVAL);
    }
    let io_vectors = io_vectors.get_as_slice(io_count as usize)?;
    let file = File::from_fd(fd as _)?;
    let mut total_written: isize = 0;
    for io in io_vectors {
        if io.length < 0 {
            return Err(LinuxError::EINVAL);
        }
        let buf_ptr = UserInPtr::<u8>::from(io.base_addr as usize);
        let buf = buf_ptr.get_as_slice(io.length as _)?;
        let write_len = sys_pwrite_impl(&file, buf, offset)?;
        total_written += write_len;
    }
    Ok(total_written)
}
