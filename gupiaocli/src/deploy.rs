use std::{
    env, io,
    path::Path,
    process::{Command, Stdio},
};

use crate::{
    process::run_passthrough,
    project::{Project, FRONTEND_PORT},
    tasks,
};

const DEFAULT_DEPLOY_HOST: &str = "x@192.168.0.109";
const DEFAULT_DEPLOY_ROOT: &str = "/home/x/code/AAAgupiao";

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeployOptions {
    target: String,
    host: String,
    remote_root: String,
    release: bool,
    skip_check: bool,
    no_build: bool,
    no_restart: bool,
    dry_run: bool,
}

impl DeployOptions {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            target: "all".to_string(),
            host: env::var("GUPIAO_DEPLOY_HOST")
                .unwrap_or_else(|_| DEFAULT_DEPLOY_HOST.to_string()),
            remote_root: env::var("GUPIAO_DEPLOY_ROOT")
                .unwrap_or_else(|_| DEFAULT_DEPLOY_ROOT.to_string()),
            release: true,
            skip_check: false,
            no_build: false,
            no_restart: false,
            dry_run: false,
        };

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            match arg.as_str() {
                "all" | "backend" | "frontend" => options.target = arg.clone(),
                "--backend-only" => options.target = "backend".to_string(),
                "--frontend-only" => options.target = "frontend".to_string(),
                "--host" => {
                    i += 1;
                    options.host = args
                        .get(i)
                        .ok_or_else(|| "--host requires a value".to_string())?
                        .clone();
                }
                "--root" => {
                    i += 1;
                    options.remote_root = args
                        .get(i)
                        .ok_or_else(|| "--root requires a value".to_string())?
                        .clone();
                }
                "--release" => options.release = true,
                "--debug" => options.release = false,
                "--skip-check" => options.skip_check = true,
                "--no-build" => options.no_build = true,
                "--no-restart" => options.no_restart = true,
                "--dry-run" => options.dry_run = true,
                _ if arg.starts_with("--host=") => {
                    options.host = arg.trim_start_matches("--host=").to_string();
                }
                _ if arg.starts_with("--root=") => {
                    options.remote_root = arg.trim_start_matches("--root=").to_string();
                }
                _ => return Err(format!("unknown deploy option: {arg}")),
            }
            i += 1;
        }

        if options.host.trim().is_empty() {
            return Err("deploy host cannot be empty".to_string());
        }
        if options.remote_root.trim().is_empty() {
            return Err("deploy root cannot be empty".to_string());
        }

        Ok(options)
    }
}

pub fn deploy(project: &Project, args: &[String]) -> io::Result<i32> {
    let options = match DeployOptions::parse(args) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            return Ok(2);
        }
    };

    println!(
        "deploying {} to {}:{}",
        options.target, options.host, options.remote_root
    );

    if !options.skip_check {
        let verify_args = if options.release {
            vec![options.target.clone(), "--release".to_string()]
        } else {
            vec![options.target.clone(), "--debug".to_string()]
        };
        let code = tasks::verify(project, &verify_args)?;
        if code != 0 {
            return Ok(code);
        }
    }

    if options.dry_run {
        print_dry_run(project, &options);
        return Ok(0);
    }

    ensure_remote_root(&options)?;
    sync_source(project.root(), &options)?;
    prepare_remote_dependencies(&options)?;

    if !options.no_build {
        remote_build(&options)?;
    }

    if !options.no_restart {
        restart_remote(&options)?;
    }

    println!("deploy complete");
    println!(
        "frontend: http://{}:{FRONTEND_PORT}/",
        host_without_user(&options.host)
    );
    println!(
        "backend health: http://{}:8080/health",
        host_without_user(&options.host)
    );
    Ok(0)
}

fn print_dry_run(project: &Project, options: &DeployOptions) {
    println!("dry-run:");
    println!("  local root: {}", project.root().display());
    println!("  sync target: {}:{}", options.host, options.remote_root);
    println!(
        "  build: {}",
        if options.no_build { "skip" } else { "remote" }
    );
    println!(
        "  restart: {}",
        if options.no_restart { "skip" } else { "remote" }
    );
}

fn ensure_remote_root(options: &DeployOptions) -> io::Result<i32> {
    run_remote(
        options,
        &format!("mkdir -p {}", sh_quote(&options.remote_root)),
    )
}

fn sync_source(root: &Path, options: &DeployOptions) -> io::Result<i32> {
    let mut cmd = rsync_command();
    cmd.args(["-az", "--delete"])
        .arg("--exclude=.git/")
        .arg("--exclude=.venv/")
        .arg("--exclude=.ruff_cache/")
        .arg("--exclude=backend/.env")
        .arg("--exclude=backend/target/")
        .arg("--exclude=gupiaocli/target/")
        .arg("--exclude=frontend/node_modules/")
        .arg("--exclude=frontend/dist/")
        .arg("--exclude=logs/")
        .arg("--exclude=run/")
        .arg("--exclude=OpenBB/.git/")
        .arg(format!("{}/", root.display()))
        .arg(format!("{}:{}/", options.host, options.remote_root));

    println!("== deploy: rsync source ==");
    run_passthrough(&mut cmd)
}

fn prepare_remote_dependencies(options: &DeployOptions) -> io::Result<i32> {
    let root = sh_quote(&options.remote_root);
    let script = format!(
        r#"
set -e
ROOT={root}
export PATH="$HOME/.cargo/bin:$HOME/.local/node-v24.11.0-linux-x64/bin:$PATH"
mkdir -p "$ROOT/logs" "$ROOT/run"
if [ -d "$ROOT/frontend" ] && [ ! -d "$ROOT/frontend/node_modules" ]; then
  cd "$ROOT/frontend"
  npm ci
fi
"#
    );
    run_remote(options, &script)
}

fn remote_build(options: &DeployOptions) -> io::Result<i32> {
    let root = sh_quote(&options.remote_root);
    let release_flag = if options.release { " --release" } else { "" };
    let script = format!(
        r#"
set -e
ROOT={root}
export PATH="$HOME/.cargo/bin:$HOME/.local/node-v24.11.0-linux-x64/bin:$PATH"
cd "$ROOT/gupiaocli"
cargo run -- build {}{release_flag}
"#,
        options.target
    );
    run_remote(options, &script)
}

fn restart_remote(options: &DeployOptions) -> io::Result<i32> {
    match options.target.as_str() {
        "backend" => restart_backend(options),
        "frontend" => restart_frontend(options),
        "all" => {
            let backend = restart_backend(options)?;
            if backend != 0 {
                return Ok(backend);
            }
            restart_frontend(options)
        }
        _ => Ok(2),
    }
}

fn restart_backend(options: &DeployOptions) -> io::Result<i32> {
    let root = sh_quote(&options.remote_root);
    let binary = if options.release {
        "target/release/backend"
    } else {
        "target/debug/backend"
    };
    let script = format!(
        r#"
set -e
ROOT={root}
mkdir -p "$ROOT/logs" "$ROOT/run"
if [ -s "$ROOT/run/backend.pid" ] && kill -0 "$(cat "$ROOT/run/backend.pid")" 2>/dev/null; then
  kill "$(cat "$ROOT/run/backend.pid")" 2>/dev/null || true
  sleep 1
fi
cd "$ROOT/backend"
PY_SITE="$ROOT/.venv/lib/python3.11/site-packages"
PY_PLATFORM="$ROOT/OpenBB/openbb_platform"
nohup env \
  LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu \
  PYTHONPATH="$PY_SITE:$PY_PLATFORM" \
  VIRTUAL_ENV="$ROOT/.venv" \
  PATH="$ROOT/.venv/bin:$HOME/.cargo/bin:$HOME/.local/node-v24.11.0-linux-x64/bin:$PATH" \
  RUST_LOG=info \
  ./{binary} > "$ROOT/logs/backend.log" 2>&1 &
echo $! > "$ROOT/run/backend.pid"
sleep 2
curl -fsS http://127.0.0.1:8080/health
"#
    );
    run_remote(options, &script)
}

fn restart_frontend(options: &DeployOptions) -> io::Result<i32> {
    let root = sh_quote(&options.remote_root);
    let script = format!(
        r#"
set -e
ROOT={root}
mkdir -p "$ROOT/logs" "$ROOT/run"
if [ -s "$ROOT/run/frontend.pid" ] && kill -0 "$(cat "$ROOT/run/frontend.pid")" 2>/dev/null; then
  kill "$(cat "$ROOT/run/frontend.pid")" 2>/dev/null || true
fi
for port in 5173 5174; do
  pids=$(ss -ltnp "( sport = :$port )" 2>/dev/null | sed -n 's/.*pid=\([0-9]*\).*/\1/p' | sort -u)
  if [ -n "$pids" ]; then kill $pids 2>/dev/null || true; fi
done
sleep 1
cd "$ROOT/frontend"
nohup env PATH="$HOME/.local/node-v24.11.0-linux-x64/bin:$PATH" \
  npm run preview -- --host 0.0.0.0 --port 5173 > "$ROOT/logs/frontend.log" 2>&1 &
echo $! > "$ROOT/run/frontend.pid"
sleep 2
curl -fsSI http://127.0.0.1:5173/ >/dev/null
"#
    );
    run_remote(options, &script)
}

fn run_remote(options: &DeployOptions, script: &str) -> io::Result<i32> {
    let mut cmd = ssh_command();
    let remote_command = format!("sh -lc {}", sh_quote(script));
    cmd.arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg(&options.host)
        .arg(remote_command)
        .stdin(Stdio::null());

    run_passthrough(&mut cmd)
}

fn ssh_command() -> Command {
    if env::var_os("SSHPASS").is_some() {
        let mut cmd = Command::new("sshpass");
        cmd.arg("-e").arg("ssh");
        cmd
    } else {
        Command::new("ssh")
    }
}

fn rsync_command() -> Command {
    if env::var_os("SSHPASS").is_some() {
        let mut cmd = Command::new("sshpass");
        cmd.arg("-e").arg("rsync");
        cmd
    } else {
        Command::new("rsync")
    }
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn host_without_user(host: &str) -> &str {
    host.rsplit_once('@').map(|(_, host)| host).unwrap_or(host)
}

#[cfg(test)]
mod tests {
    use super::{host_without_user, sh_quote, DeployOptions, DEFAULT_DEPLOY_ROOT};

    #[test]
    fn deploy_parse_defaults_to_all_release() {
        let options = DeployOptions::parse(&[]).unwrap();
        assert_eq!(options.target, "all");
        assert!(options.release);
        assert_eq!(options.remote_root, DEFAULT_DEPLOY_ROOT);
    }

    #[test]
    fn deploy_parse_host_root_and_target() {
        let args = vec![
            "frontend".to_string(),
            "--host=me@example".to_string(),
            "--root".to_string(),
            "/srv/app".to_string(),
            "--debug".to_string(),
            "--no-restart".to_string(),
        ];
        let options = DeployOptions::parse(&args).unwrap();

        assert_eq!(options.target, "frontend");
        assert_eq!(options.host, "me@example");
        assert_eq!(options.remote_root, "/srv/app");
        assert!(!options.release);
        assert!(options.no_restart);
    }

    #[test]
    fn deploy_parse_rejects_unknown_option() {
        let args = vec!["--wat".to_string()];
        assert!(DeployOptions::parse(&args).is_err());
    }

    #[test]
    fn shell_quote_handles_single_quotes() {
        assert_eq!(sh_quote("/tmp/a'b"), "'/tmp/a'\\''b'");
    }

    #[test]
    fn host_label_strips_user() {
        assert_eq!(host_without_user("x@192.168.0.109"), "192.168.0.109");
        assert_eq!(host_without_user("192.168.0.109"), "192.168.0.109");
    }
}
