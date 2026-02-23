pub mod commands;
mod init_project;
mod run;

pub fn run() -> Result<(), String> {
	commands::run_from_env()
}
