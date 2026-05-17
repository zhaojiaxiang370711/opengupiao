use std::{
    ffi::OsStr,
    io,
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use crate::{
    process::run_passthrough,
    project::{Project, BACKEND_PORT, FRONTEND_PORT},
};

pub fn doctor(project: &Project) -> io::Result<i32> {
    println!("AAAgupiao root: {}", project.root().display());
    check_path("backend", &project.backend_dir());
    check_path("frontend", &project.frontend_dir());
    check_path("OpenBB", &project.openbb_dir());
    check_path("local venv", &project.path(".venv"));
    check_path("backend .env", &project.path("backend/.env"));
    println!();

    check_tool("cargo", ["--version"]);
    check_tool("rustc", ["--version"]);
    check_tool("node", ["--version"]);
    check_tool("npm", ["--version"]);
    check_tool("python", ["--version"]);
    check_tool_path(
        "project python",
        project.path(".venv/bin/python"),
        ["--version"],
    );
    check_tool("git", ["--version"]);
    check_tool("psql", ["--version"]);
    check_tool("redis-cli", ["--version"]);
    check_tool("ssh", ["-V"]);
    check_tool("rsync", ["--version"]);
    println!();

    check_port("backend", BACKEND_PORT);
    check_port("frontend", FRONTEND_PORT);
    Ok(0)
}

pub fn status(project: &Project) -> io::Result<i32> {
    println!("AAAgupiao root: {}", project.root().display());
    println!(
        "backend target: {}",
        exists_label(&project.path("backend/target"))
    );
    println!(
        "frontend node_modules: {}",
        exists_label(&project.path("frontend/node_modules"))
    );
    println!(
        "frontend dist: {}",
        exists_label(&project.path("frontend/dist"))
    );
    println!(
        "gupiaocli target: {}",
        exists_label(&project.path("gupiaocli/target"))
    );
    println!(
        "OpenBB repo: {}",
        exists_label(&project.path("OpenBB/.git"))
    );
    println!();

    println!("ports:");
    check_port("backend", BACKEND_PORT);
    check_port("frontend", FRONTEND_PORT);
    println!();

    if project.path("OpenBB/.git").exists() {
        println!("OpenBB git:");
        run_passthrough(
            Command::new("git")
                .arg("-C")
                .arg(project.openbb_dir())
                .args(["status", "--short", "--branch", "--untracked-files=no"]),
        )?;
        println!();
    }

    if let Some(parent) = project.root().parent() {
        if parent.join(".git").exists() {
            println!("workspace git:");
            run_passthrough(Command::new("git").arg("-C").arg(parent).args([
                "status",
                "--short",
                "--branch",
                "--untracked-files=no",
            ]))?;
        }
    }

    Ok(0)
}

fn check_path(label: &str, path: &Path) {
    println!("{:<20} {}", label, exists_label(path));
}

fn exists_label(path: &Path) -> &'static str {
    if path.exists() {
        "ok"
    } else {
        "missing"
    }
}

fn check_tool<I, S>(tool: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    match Command::new(tool).args(args).output() {
        Ok(output) if output.status.success() || tool == "ssh" => {
            let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() {
                text = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            println!("{:<20} ok {}", tool, text.lines().next().unwrap_or(""));
        }
        Ok(_) => println!("{:<20} present but failed version check", tool),
        Err(_) => println!("{:<20} missing", tool),
    }
}

fn check_tool_path<I, S>(label: &str, path: PathBuf, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if !path.exists() {
        println!("{:<20} missing", label);
        return;
    }

    match Command::new(path).args(args).output() {
        Ok(output) if output.status.success() => {
            let mut text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if text.is_empty() {
                text = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            println!("{:<20} ok {}", label, text.lines().next().unwrap_or(""));
        }
        Ok(_) => println!("{:<20} present but failed version check", label),
        Err(_) => println!("{:<20} missing", label),
    }
}

fn check_port(label: &str, port: u16) {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let open = TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok();
    println!(
        "{:<20} :{} {}",
        label,
        port,
        if open { "open" } else { "closed" }
    );
}
