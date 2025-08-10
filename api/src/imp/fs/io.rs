use crate::core::file::fd::{FileLike, file_like_as};
use crate::core::file::file::File;
use alloc::sync::Arc;
use alloc::vec;
use axerrno::{LinuxError, LinuxResult};
use axfs_ng::api::FileFlags;
use core::cmp::min;

pub fn sys_truncate_impl(file: &File, length: u64) -> LinuxResult<isize> {
    // set file size to length
    file.inner().resize(length)?;
    Ok(0)
}

pub fn sys_read_impl(file_like: &dyn FileLike, buf: &mut [u8]) -> LinuxResult<isize> {
    let read_len = file_like.read(buf)?;
    Ok(read_len as _)
}

pub fn sys_write_impl(file_like: &dyn FileLike, buf: &[u8]) -> LinuxResult<isize> {
    let write_len = file_like.write(buf)?;
    Ok(write_len as _)
}

pub fn sys_pwrite_impl(file: &File, buf: &[u8], offset: isize) -> LinuxResult<isize> {
    if offset < 0 {
        return Err(LinuxError::EINVAL);
    }
    let write_len = file.inner().write_at(buf, offset as _).map_err(|e| {
        if e == LinuxError::EACCES {
            LinuxError::EBADF
        } else {
            e
        }
    })?;
    Ok(write_len as _)
}

pub fn sys_pread_impl(file: &File, buf: &mut [u8], offset: isize) -> LinuxResult<isize> {
    if offset < 0 {
        return Err(LinuxError::EINVAL);
    }
    let read_len = file.inner().read_at(buf, offset as _).map_err(|e| {
        if e == LinuxError::EACCES {
            LinuxError::EBADF
        } else {
            e
        }
    })?;
    Ok(read_len as _)
}

pub fn sys_copy_file_range_impl(
    in_file: Arc<dyn FileLike>,
    in_offset: Option<&mut usize>,
    out_file: Arc<dyn FileLike>,
    out_offset: Option<&mut usize>,
    count: usize,
    // TODO: support flags
) -> LinuxResult<usize> {
    if !in_file.poll()?.readable || !out_file.poll()?.writable {
        return Err(LinuxError::EBADF);
    }
    if out_file.get_flags().contains(FileFlags::APPEND) {
        // out_fd has the O_APPEND flag set.
        // This is not currently supported by sendfile().
        return Err(LinuxError::EINVAL);
    }
    // TODO: allow other seekable file types
    enum FileWrapper {
        FileLike(Arc<dyn FileLike>),
        FileLikeSeekable(Arc<File>),
    }
    impl FileWrapper {
        fn try_new(file: Arc<dyn FileLike>, seekable: bool) -> LinuxResult<Self> {
            if seekable {
                if let Some(file) = file_like_as::<File>(file) {
                    Ok(FileWrapper::FileLikeSeekable(file))
                } else {
                    Err(LinuxError::ESPIPE)
                }
            } else {
                Ok(FileWrapper::FileLike(file))
            }
        }
    }
    let in_file = FileWrapper::try_new(in_file, in_offset.is_some())?;
    let out_file = FileWrapper::try_new(out_file, out_offset.is_some())?;
    let dummy_in_offset = &mut 0;
    let dummy_out_offset = &mut 0;
    let in_offset = in_offset.unwrap_or(dummy_in_offset);
    let out_offset = out_offset.unwrap_or(dummy_out_offset);

    // TODO: optimize buffer size based on file size and filesystem block size
    let buf_size = count.min(40960);
    let mut buf = vec![0u8; buf_size];
    let mut transferred = 0;

    while transferred < count {
        let expected_len = min(buf_size, count - transferred);
        let buffer = &mut buf[..expected_len];
        let read_len = match &in_file {
            FileWrapper::FileLike(in_file) => in_file.read(buffer)?,
            FileWrapper::FileLikeSeekable(in_file) => in_file
                .inner()
                .read_at(buffer, (*in_offset + transferred) as _)?,
        };

        if read_len == 0 {
            break; // EOF
        }
        let write_len = match &out_file {
            FileWrapper::FileLike(out_file) => out_file.write(&buffer[..read_len])?,
            FileWrapper::FileLikeSeekable(out_file) => out_file
                .inner()
                .write_at(&buffer[..read_len], (*out_offset + transferred) as _)?,
        };

        transferred += write_len;

        // TODO: 不确定这样做对于socket等是否正确
        if write_len < read_len || read_len < expected_len {
            break;
        }
    }

    *in_offset += transferred;
    *out_offset += transferred;

    Ok(transferred)
}
