use std::io;

use crate::{checks, deploy, project::Project, tasks};

pub fn dispatch(project: &Project, args: &[String]) -> io::Result<i32> {
    match args.first().map(String::as_str) {
        None | Some("-h") | Some("--help") | Some("help") => {
            print_help();
            Ok(0)
        }
        Some("doctor") => checks::doctor(project),
        Some("status") => checks::status(project),
        Some("check") => tasks::check(project, args.get(1).map(String::as_str).unwrap_or("all")),
        Some("test") => tasks::test(project, args.get(1).map(String::as_str).unwrap_or("all")),
        Some("build") => tasks::build(project, &args[1..]),
        Some("verify") => tasks::verify(project, &args[1..]),
        Some("dev") => tasks::dev(project, &args[1..]),
        Some("run-backend") => tasks::run_backend(project, &args[1..]),
        Some("run-frontend") => tasks::run_frontend(project),
        Some("deploy") => deploy::deploy(project, &args[1..]),
        Some("update-openbb") => tasks::update_openbb(project),
        Some("clean") => tasks::clean(project, &args[1..]),
        Some(other) => {
            eprintln!("unknown command: {other}");
            print_help();
            Ok(2)
        }
    }
}

fn print_help() {
    println!(
        r#"gupiao - local AAAgupiao development helper

Usage:
  gupiao doctor                  Check local tools, ports, and project layout
  gupiao status                  Show Git/status hints for AAAgupiao and OpenBB
  gupiao check [target]          Run static checks: all | backend | frontend
  gupiao test [target]           Run tests: all | backend | frontend
  gupiao build [target]          Build all | backend | frontend [--release]
  gupiao verify [target]         Run check, test, then build [--release]
  gupiao dev [--release]         Run backend and frontend together
  gupiao run-backend [--release] Run Rust backend
  gupiao run-frontend            Run Vite frontend
  gupiao deploy [target]         Sync, build, and restart a remote deployment
  gupiao update-openbb           Fetch and fast-forward OpenBB
  gupiao clean [--yes]           Remove build output; dry-run unless --yes

Deploy:
  gupiao deploy [all|backend|frontend] [--host user@host] [--root remote_path]
                [--skip-check] [--no-build] [--no-restart] [--debug] [--dry-run]

Environment:
  GUPIAO_ROOT can override the detected project root.
  GUPIAO_DEPLOY_HOST defaults to x@192.168.0.109.
  GUPIAO_DEPLOY_ROOT defaults to /home/x/code/AAAgupiao.
  SSHPASS can be set to make deploy use sshpass -e for password auth.
"#
    );
}
