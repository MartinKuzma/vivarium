pub mod commands;

pub fn run() -> Result<(), String> {
	commands::run_from_env()
}
