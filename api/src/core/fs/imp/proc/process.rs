use crate::core::fs::imp::proc::task_stat::TaskStat;
use crate::core::fs::imp::proc::{DUMMY_MAPS, DUMMY_STATUS};
use crate::core::fs::pseudo::dir::PseudoDirOps;
use crate::core::fs::pseudo::dynamic::{DirMaker, DynNodeOps, DynamicDir, DynamicFs};
use crate::core::fs::pseudo::file::{SimpleFile, WithResult};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use alloc::sync::Arc;
use alloc::vec::Vec;
use starry_core::process::get_process_data;
use undefined_process::Pid;
use undefined_process::process::get_all_processes;
use undefined_vfs::{VfsError, VfsResult};

fn process_info_builder(fs: Arc<DynamicFs>, pid: Pid) -> Option<DirMaker> {
    let process_data = get_process_data(pid)?;

    let mut root = DynamicDir::builder(fs.clone());

    // cmdline
    let data = process_data.clone();
    root.add(
        "cmdline",
        SimpleFile::new(fs.clone(), move || {
            // if command line is "foo bar baz", it should return "foo\0bar\0baz\0"
            let command_line = data.command_line.lock();
            let mut buffer = Vec::new();
            for arg in command_line.iter() {
                buffer.extend_from_slice(arg.as_bytes());
                buffer.push(0); // null-terminate each argument
            }
            buffer
        }),
    );

    // stat
    root.add(
        "stat",
        SimpleFile::new(
            fs.clone(),
            WithResult(move || {
                let stat = TaskStat::from_process(pid)?;
                Ok(format!("{stat}").into_bytes())
            }),
        ),
    );

    // status
    root.add("status", SimpleFile::new(fs.clone(), || DUMMY_STATUS));

    // maps
    root.add("maps", SimpleFile::new(fs.clone(), || DUMMY_MAPS));

    Some(root.build())
}

pub struct ProcessInfoDir {
    fs: Arc<DynamicFs>,
}

impl ProcessInfoDir {
    pub fn new(fs: Arc<DynamicFs>) -> Self {
        Self { fs }
    }
}

impl PseudoDirOps for ProcessInfoDir {
    fn list_children<'a>(&'a self) -> Box<dyn Iterator<Item = Cow<'a, str>> + 'a> {
        Box::new(
            get_all_processes()
                .into_iter()
                .map(|p| p.get_pid().to_string().into()),
        )
    }

    fn get_child(&self, name: &str) -> VfsResult<DynNodeOps> {
        let pid = name.parse::<Pid>().map_err(|_| VfsError::ENOENT)?;
        let builder = process_info_builder(self.fs.clone(), pid).ok_or(VfsError::ENOENT)?;
        Ok(builder.into())
    }
}
