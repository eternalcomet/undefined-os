use arceos_posix_api::File;
use axerrno::LinuxResult;
use axfs::fops;
use axfs::fops::OpenOptions;

pub fn open_file(path: &str, options: Option<OpenOptions>) -> LinuxResult<File> {
    let options = options.unwrap_or_else(|| {
        let mut options = OpenOptions::new();
        options.read(true);
        options
    });
    let file_inner = fops::File::open(path, &options)?;
    Ok(File::new(file_inner, path))
}
