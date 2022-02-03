use std::borrow::Cow;
use std::process::{Command, Stdio};

pub fn spawn_process(path: &str, params: Option<Cow<'_, str>>, working_directory: Option<Cow<'_, str>>) -> Result<u32, std::io::Error> {
	let mut cmd = Command::new(path);
    if let Some(args ) = params {
        for arg in args.split(' ') {
            cmd.arg(arg.trim());
        }
    }

    if let Some(dir) = working_directory {
        cmd.current_dir(std::path::PathBuf::from(dir.as_ref()));
    }

    let child = cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn();

    match child {
        Ok(mut child) => Ok(child.id()),
        Err(e) => Err(e)
    }
}