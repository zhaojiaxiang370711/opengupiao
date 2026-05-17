use std::{
    fs, io,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{
    environment::apply_project_env,
    process::{base_command, run_all, run_passthrough, stop_child},
    project::{Project, BACKEND_PORT, FRONTEND_PORT},
};

pub fn check(project: &Project, target: &str) -> io::Result<i32> {
    match target {
        "backend" => run_backend_check(project),
        "frontend" => run_frontend_check(project),
        "all" => run_all([run_backend_check(project)?, run_frontend_check(project)?]),
        other => {
            eprintln!("unknown check target: {other}");
            Ok(2)
        }
    }
}

pub fn test(project: &Project, target: &str) -> io::Result<i32> {
    match target {
        "backend" => run_backend_tests(project),
        "frontend" => run_frontend_tests(project),
        "all" => run_all([run_backend_tests(project)?, run_frontend_tests(project)?]),
        other => {
            eprintln!("unknown test target: {other}");
            Ok(2)
        }
    }
}

pub fn build(project: &Project, args: &[String]) -> io::Result<i32> {
    let (target, release) = target_and_release(args, "all", false);
    build_target(project, target, release)
}

pub fn verify(project: &Project, args: &[String]) -> io::Result<i32> {
    let (target, release) = target_and_release(args, "all", false);

    let check_code = check(project, target)?;
    if check_code != 0 {
        return Ok(check_code);
    }

    let test_code = test(project, target)?;
    if test_code != 0 {
        return Ok(test_code);
    }

    build_target(project, target, release)
}

pub fn dev(project: &Project, args: &[String]) -> io::Result<i32> {
    let release = args.iter().any(|arg| arg == "--release");
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_signal = shutdown.clone();

    if let Err(err) = ctrlc::set_handler(move || {
        shutdown_signal.store(true, Ordering::SeqCst);
    }) {
        eprintln!("warning: could not install Ctrl-C handler: {err}");
    }

    println!("starting backend on :{BACKEND_PORT}");
    let mut backend = backend_command(project, "run", release)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    println!("starting frontend on :{FRONTEND_PORT}");
    let mut frontend = frontend_dev_command(project)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    loop {
        if shutdown.load(Ordering::SeqCst) {
            stop_child("backend", &mut backend);
            stop_child("frontend", &mut frontend);
            return Ok(130);
        }

        if let Some(status) = backend.try_wait()? {
            stop_child("frontend", &mut frontend);
            return Ok(status.code().unwrap_or(1));
        }

        if let Some(status) = frontend.try_wait()? {
            stop_child("backend", &mut backend);
            return Ok(status.code().unwrap_or(1));
        }

        thread::sleep(Duration::from_millis(200));
    }
}

pub fn run_backend(project: &Project, args: &[String]) -> io::Result<i32> {
    let release = args.iter().any(|arg| arg == "--release");
    run_passthrough(&mut backend_command(project, "run", release))
}

pub fn run_frontend(project: &Project) -> io::Result<i32> {
    run_passthrough(&mut frontend_dev_command(project))
}

pub fn update_openbb(project: &Project) -> io::Result<i32> {
    let openbb = project.openbb_dir();
    if !openbb.join(".git").exists() {
        eprintln!("OpenBB git repo not found at {}", openbb.display());
        return Ok(1);
    }

    let fetch = run_passthrough(
        Command::new("git")
            .arg("-C")
            .arg(&openbb)
            .args(["fetch", "--prune", "origin"]),
    )?;
    if fetch != 0 {
        return Ok(fetch);
    }

    run_passthrough(
        Command::new("git")
            .arg("-C")
            .arg(&openbb)
            .args(["pull", "--ff-only"]),
    )
}

pub fn clean(project: &Project, args: &[String]) -> io::Result<i32> {
    let yes = args.iter().any(|arg| arg == "--yes");
    let include_node_modules = args.iter().any(|arg| arg == "--include-node-modules");
    let include_venv = args.iter().any(|arg| arg == "--include-venv");

    let mut paths = vec![
        project.path("backend/target"),
        project.path("frontend/dist"),
        project.path("gupiaocli/target"),
    ];

    if include_node_modules {
        paths.push(project.path("frontend/node_modules"));
    }
    if include_venv {
        paths.push(project.path(".venv"));
        paths.push(project.path(".ruff_cache"));
    }

    for path in paths {
        if !path.exists() {
            continue;
        }
        if yes {
            println!("removing {}", path.display());
            fs::remove_dir_all(&path)?;
        } else {
            println!("would remove {}", path.display());
        }
    }

    if !yes {
        println!("dry-run only; pass --yes to remove these paths");
    }

    Ok(0)
}

pub fn target_and_release<'a>(
    args: &'a [String],
    default_target: &'a str,
    default_release: bool,
) -> (&'a str, bool) {
    let target = args
        .iter()
        .find(|arg| !arg.starts_with('-'))
        .map(String::as_str)
        .unwrap_or(default_target);
    let release = if args.iter().any(|arg| arg == "--debug") {
        false
    } else if args.iter().any(|arg| arg == "--release") {
        true
    } else {
        default_release
    };

    (target, release)
}

pub fn build_target(project: &Project, target: &str, release: bool) -> io::Result<i32> {
    match target {
        "backend" => run_backend_build(project, release),
        "frontend" => run_frontend_build(project),
        "all" => run_all([
            run_backend_build(project, release)?,
            run_frontend_build(project)?,
        ]),
        other => {
            eprintln!("unknown build target: {other}");
            Ok(2)
        }
    }
}

fn run_backend_check(project: &Project) -> io::Result<i32> {
    println!("== backend: cargo check ==");
    run_passthrough(&mut backend_command(project, "check", false))
}

fn run_backend_tests(project: &Project) -> io::Result<i32> {
    println!("== backend: cargo test ==");
    let mut cmd = base_command("cargo", project.backend_dir());
    cmd.arg("test");
    apply_project_env(&mut cmd, project);
    run_passthrough(&mut cmd)
}

fn run_backend_build(project: &Project, release: bool) -> io::Result<i32> {
    println!(
        "== backend: cargo build{} ==",
        if release { " --release" } else { "" }
    );
    run_passthrough(&mut backend_command(project, "build", release))
}

fn run_frontend_check(project: &Project) -> io::Result<i32> {
    println!("== frontend: vue-tsc ==");
    let mut cmd = base_command("npm", project.frontend_dir());
    cmd.args(["exec", "vue-tsc", "--", "-b"]);
    apply_project_env(&mut cmd, project);
    run_passthrough(&mut cmd)
}

fn run_frontend_tests(project: &Project) -> io::Result<i32> {
    let package_json =
        fs::read_to_string(project.path("frontend/package.json")).unwrap_or_default();
    if package_json.contains(r#""test""#) {
        println!("== frontend: npm test ==");
        let mut cmd = base_command("npm", project.frontend_dir());
        cmd.arg("test");
        apply_project_env(&mut cmd, project);
        run_passthrough(&mut cmd)
    } else {
        println!("== frontend: no test script; running vue-tsc instead ==");
        run_frontend_check(project)
    }
}

fn run_frontend_build(project: &Project) -> io::Result<i32> {
    println!("== frontend: npm run build ==");
    let mut cmd = base_command("npm", project.frontend_dir());
    cmd.args(["run", "build"]);
    apply_project_env(&mut cmd, project);
    run_passthrough(&mut cmd)
}

fn backend_command(project: &Project, cargo_subcommand: &str, release: bool) -> Command {
    let mut cmd = base_command("cargo", project.backend_dir());
    cmd.arg(cargo_subcommand);
    if release {
        cmd.arg("--release");
    }
    apply_project_env(&mut cmd, project);
    cmd
}

fn frontend_dev_command(project: &Project) -> Command {
    let mut cmd = base_command("npm", project.frontend_dir());
    cmd.args(["run", "dev", "--", "--host", "0.0.0.0"]);
    apply_project_env(&mut cmd, project);
    cmd
}

#[cfg(test)]
mod tests {
    use super::target_and_release;

    #[test]
    fn target_defaults_to_all() {
        let args: Vec<String> = vec![];
        assert_eq!(target_and_release(&args, "all", false), ("all", false));
    }

    #[test]
    fn target_and_release_are_parsed_independently() {
        let args = vec!["backend".to_string(), "--release".to_string()];
        assert_eq!(target_and_release(&args, "all", false), ("backend", true));
    }

    #[test]
    fn debug_overrides_release_default() {
        let args = vec!["frontend".to_string(), "--debug".to_string()];
        assert_eq!(target_and_release(&args, "all", true), ("frontend", false));
    }
}
