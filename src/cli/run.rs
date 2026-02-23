use crate::core::persistence::loader::{self, SnapshotSelection};
use std::env;
use std::path::PathBuf;

pub fn run_project(project_dir: PathBuf, steps: u32, snapshot: SnapshotSelection) -> Result<(), String> {
	let project_dir = if project_dir.as_os_str().is_empty() {
		env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
	} else {
		project_dir
	};

	let project_dir = project_dir
		.to_str()
		.ok_or_else(|| "Project directory path is not valid UTF-8".to_string())?;

	let loaded_project = loader::load_project_from_manifest_file(project_dir, snapshot)
		.map_err(|e| format!("Failed to load project: {}", e))?;

	let mut world = loaded_project
		.instantiate_world()
		.map_err(|e| format!("Failed to instantiate world: {}", e))?;

	for _ in 0..steps {
		world
			.update(1)
			.map_err(|e| format!("Error during world update: {}", e))?;
	}

	Ok(())
}
