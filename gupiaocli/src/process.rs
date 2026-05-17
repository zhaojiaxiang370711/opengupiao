use std::{
    ffi::OsStr,
    io,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub fn base_command<S: AsRef<OsStr>>(program: S, dir: PathBuf) -> Command {
    let mut cmd = Command::new(program);
    cmd.current_dir(dir);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());
    cmd
}

pub fn run_passthrough(cmd: &mut Command) -> io::Result<i32> {
    let status = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}

pub fn run_all<const N: usize>(codes: [i32; N]) -> io::Result<i32> {
    Ok(codes.into_iter().find(|code| *code != 0).unwrap_or(0))
}

pub fn stop_child(name: &str, child: &mut Child) {
    if child.try_wait().ok().flatten().is_none() {
        println!("stopping {name}");
        let _ = child.kill();
        let _ = child.wait();
    }
}
