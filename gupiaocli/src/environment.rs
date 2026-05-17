use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{project::Project, project::BACKEND_PORT};

pub fn apply_project_env(cmd: &mut Command, project: &Project) {
    let root = project.root();
    let venv = project.path(".venv");
    let venv_bin = venv.join("bin");
    if venv_bin.exists() {
        prepend_env_path(cmd, "PATH", &venv_bin);
        cmd.env("VIRTUAL_ENV", &venv);
    }
    if let Some(lib_dir) = python_lib_dir(&venv) {
        prepend_env_path(cmd, "LD_LIBRARY_PATH", &lib_dir);
    }

    let openbb_platform = project.path("OpenBB/openbb_platform");
    let mut python_paths = Vec::new();
    if let Some(site_packages) = first_site_packages(&venv) {
        python_paths.push(site_packages);
    }
    python_paths.push(openbb_platform.clone());

    if let Some(existing) = env::var_os("PYTHONPATH") {
        python_paths.push(PathBuf::from(existing));
    }

    let joined =
        env::join_paths(python_paths).unwrap_or_else(|_| openbb_platform.clone().into_os_string());
    cmd.env("PYTHONPATH", joined);
    set_default_env(cmd, root, "OPENBB_PY_PATH", openbb_platform);
    set_default_env(
        cmd,
        root,
        "DATABASE_URL",
        "postgres://postgres:postgres@localhost:5432/aaagupiao",
    );
    set_default_env(cmd, root, "REDIS_URL", "redis://localhost:6379");
    set_default_env(cmd, root, "BIND_ADDR", format!("0.0.0.0:{BACKEND_PORT}"));
    set_default_env(cmd, root, "RUST_LOG", "info");
}

fn prepend_env_path(cmd: &mut Command, key: &str, path: &Path) {
    let mut paths = vec![path.to_path_buf()];
    if let Some(existing) = env::var_os(key) {
        paths.extend(env::split_paths(&existing));
    }
    if let Ok(joined) = env::join_paths(paths) {
        cmd.env(key, joined);
    }
}

fn set_default_env<V: AsRef<OsStr>>(cmd: &mut Command, root: &Path, key: &str, value: V) {
    if env::var_os(key).is_none() && !dotenv_has_key(&root.join("backend/.env"), key) {
        cmd.env(key, value);
    }
}

fn dotenv_has_key(path: &Path, key: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };

    content.lines().any(|line| {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return false;
        }

        trimmed
            .split_once('=')
            .map(|(name, _)| name.trim() == key)
            .unwrap_or(false)
    })
}

fn first_site_packages(venv: &Path) -> Option<PathBuf> {
    let lib = venv.join("lib");
    let entries = fs::read_dir(lib).ok()?;

    for entry in entries.flatten() {
        let path = entry.path().join("site-packages");
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn python_lib_dir(venv: &Path) -> Option<PathBuf> {
    let python = venv.join("bin/python");
    if !python.exists() {
        return None;
    }

    let output = Command::new(python)
        .arg("-c")
        .arg("import sysconfig; print(sysconfig.get_config_var('LIBDIR') or '')")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        None
    } else {
        Some(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::dotenv_has_key;
    use std::fs;

    #[test]
    fn dotenv_key_detection_ignores_comments() {
        let path = std::env::temp_dir().join(format!("gupiao-dotenv-test-{}", std::process::id()));
        fs::write(&path, "# DATABASE_URL=nope\n DATABASE_URL = yes\n").unwrap();

        assert!(dotenv_has_key(&path, "DATABASE_URL"));
        assert!(!dotenv_has_key(&path, "REDIS_URL"));

        let _ = fs::remove_file(path);
    }
}
