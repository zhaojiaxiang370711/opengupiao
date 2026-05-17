use std::{
    env,
    path::{Path, PathBuf},
};

pub const BACKEND_PORT: u16 = 8080;
pub const FRONTEND_PORT: u16 = 5173;

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
}

impl Project {
    pub fn from_env() -> Self {
        if let Ok(root) = env::var("GUPIAO_ROOT") {
            return Self {
                root: PathBuf::from(root),
            };
        }

        Self {
            root: Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("gupiaocli should live under AAAgupiao")
                .to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, rel: &str) -> PathBuf {
        self.root.join(rel)
    }

    pub fn backend_dir(&self) -> PathBuf {
        self.path("backend")
    }

    pub fn frontend_dir(&self) -> PathBuf {
        self.path("frontend")
    }

    pub fn openbb_dir(&self) -> PathBuf {
        self.path("OpenBB")
    }
}
