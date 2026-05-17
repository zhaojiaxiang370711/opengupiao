mod checks;
mod cli;
mod deploy;
mod environment;
mod process;
mod project;
mod tasks;

use std::{env, process::ExitCode};

use project::Project;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let project = Project::from_env();

    match cli::dispatch(&project, &args) {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(1)
        }
    }
}
