use crate::core::persistence::loader::{self, SnapshotSelection};
use std::env;
use std::path::PathBuf;
use crate::core::{World, WorldSnapshotData};

pub fn run_project(project_dir: PathBuf, steps: u32, snapshot: SnapshotSelection, save_snapshot: Option<String>) -> Result<(), String> {
    let project_dir = if project_dir.as_os_str().is_empty() {
        env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?
    } else {
        project_dir
    };

    let project_dir = project_dir
        .to_str()
        .ok_or_else(|| "Project directory path is not valid".to_string())?;

    let project_ctx = loader::load_project_from_file(project_dir)?;
    let snapshot = loader::load_snapshot(&project_ctx, snapshot)?;

	let world_data = WorldSnapshotData {
		name: project_ctx.manifest.name.clone(),
		script_library: project_ctx.script_library.clone(),
		entities: snapshot.entities.clone(),
		pending_messages: snapshot.pending_messages.clone(),
		simulation_time: snapshot.simulation_time,
	};
    let mut world = World::new(world_data)?;

    for _ in 0..steps {
        world.update(1).map_err(|e| format!("Error during world update: {}", e))?;
    }

    //TODO: save snapshot
    Ok(())
}
