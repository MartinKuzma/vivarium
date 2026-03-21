pub mod commands;
mod run;

pub fn run() -> Result<(), String> {
	commands::run_from_env()
}
