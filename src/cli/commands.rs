use clap::{Parser, Subcommand};
use crate::core::persistence::schema::{
	DIR_SNAPSHOTS, FILE_SNAPSHOT_ENTITIES, FILE_SNAPSHOT_MANIFEST, FILE_SNAPSHOT_MESSAGES,
	ManifestEntities, ManifestEntityCfg, ManifestMessages, ManifestScriptCfg, ManifestSnapshot,
	PROJECT_SCHEMA_VERSION_V1, ProjectManifest,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const DIR_SCRIPTS: &str = "scripts";
const INITIAL_SNAPSHOT_ID: &str = "0001-initial";
const FILE_WORLD_MANIFEST: &str = "world.yaml";
const FILE_DEFAULT_SCRIPT: &str = "agent_script.lua";

#[derive(Debug, Parser)]
#[command(name = "vivarium-cli", about = "Vivarium project utilities")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(name = "init-project", alias = "init", about = "Create a new Vivarium project structure")]
    InitProject {
        #[arg(value_name = "target-dir")]
        target_dir: PathBuf,
    },
}

pub fn run_from_env() -> Result<(), String> {
	let cli = Cli::try_parse_from(env::args()).map_err(|e| e.to_string())?;

	match cli.command {
		Commands::InitProject { target_dir } => init_project(&target_dir),
	}
}

fn init_project(target_dir: &Path) -> Result<(), String> {
	fs::create_dir_all(target_dir).map_err(|e| {
		format!(
			"Failed to create target directory '{}': {}",
			target_dir.display(),
			e
		)
	})?;

	let scripts_dir = target_dir.join(DIR_SCRIPTS);
	let snapshot_dir = target_dir.join(DIR_SNAPSHOTS).join(INITIAL_SNAPSHOT_ID);

	fs::create_dir_all(&scripts_dir).map_err(|e| {
		format!(
			"Failed to create scripts directory '{}': {}",
			scripts_dir.display(),
			e
		)
	})?;

	fs::create_dir_all(&snapshot_dir).map_err(|e| {
		format!(
			"Failed to create snapshot directory '{}': {}",
			snapshot_dir.display(),
			e
		)
	})?;

	let project_name = target_dir
		.file_name()
		.map(|name| name.to_string_lossy().to_string())
		.filter(|name| !name.trim().is_empty())
		.unwrap_or_else(|| "vivarium_project".to_string());

	let mut script_library = std::collections::HashMap::new();
	script_library.insert(
		"agent_script".to_string(),
		ManifestScriptCfg {
			id: "agent_script".to_string(),
			kind: "lua".to_string(),
			script_path: format!("{}/{}", DIR_SCRIPTS, FILE_DEFAULT_SCRIPT),
		},
	);

	let world_manifest = ProjectManifest {
		schema_version: PROJECT_SCHEMA_VERSION_V1.to_string(),
		name: project_name,
		script_library,
	};

	let script_content = "counter = 0\n\nfunction update(current_time, msgs)\n    counter = counter + 1\nend\n\nfunction get_state()\n    return { counter = counter }\nend\n\nfunction set_state(state)\n    counter = state.counter or 0\nend\n";

	let snapshot_manifest = ManifestSnapshot {
		schema_version: PROJECT_SCHEMA_VERSION_V1.to_string(),
		id: INITIAL_SNAPSHOT_ID.to_string(),
		simulation_time: 0,
	};

	let mut initial_state = serde_json::Map::new();
	initial_state.insert("counter".to_string(), serde_json::Value::from(0));

	let entities_manifest = ManifestEntities {
		entities: vec![ManifestEntityCfg {
			id: "agent_1".to_string(),
			script_id: "agent_script".to_string(),
			initial_state: Some(initial_state),
		}],
	};

	let messages_manifest = ManifestMessages {
		messages: Vec::new(),
	};

	write_if_missing_yaml(&target_dir.join(FILE_WORLD_MANIFEST), &world_manifest)?;
	write_if_missing(&scripts_dir.join(FILE_DEFAULT_SCRIPT), script_content)?;
	write_if_missing_yaml(
		&snapshot_dir.join(FILE_SNAPSHOT_MANIFEST),
		&snapshot_manifest,
	)?;
	write_if_missing_yaml(
		&snapshot_dir.join(FILE_SNAPSHOT_ENTITIES),
		&entities_manifest,
	)?;
	write_if_missing_yaml(
		&snapshot_dir.join(FILE_SNAPSHOT_MESSAGES),
		&messages_manifest,
	)?;

	println!("Project initialized at '{}'", target_dir.display());
	Ok(())
}

fn write_if_missing(path: &Path, content: &str) -> Result<(), String> {
	if path.exists() {
		return Ok(());
	}

	fs::write(path, content)
		.map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
}

fn write_if_missing_yaml<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
	if path.exists() {
		return Ok(());
	}

	let content = serde_yaml::to_string(value)
		.map_err(|e| format!("Failed to serialize YAML for '{}': {}", path.display(), e))?;

	fs::write(path, content)
		.map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn init_project_creates_required_loader_files() {
		let unique = format!(
			"vivarium-cli-test-{}",
			std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap()
				.as_nanos()
		);

		let target = std::env::temp_dir().join(unique);

		init_project(&target).expect("init_project should create the project tree");

		assert!(target.join(FILE_WORLD_MANIFEST).exists());
		assert!(target.join(DIR_SCRIPTS).join(FILE_DEFAULT_SCRIPT).exists());
		assert!(target
			.join(DIR_SNAPSHOTS)
			.join(INITIAL_SNAPSHOT_ID)
			.join(FILE_SNAPSHOT_MANIFEST)
			.exists());
		assert!(target
			.join(DIR_SNAPSHOTS)
			.join(INITIAL_SNAPSHOT_ID)
			.join(FILE_SNAPSHOT_ENTITIES)
			.exists());
		assert!(target
			.join(DIR_SNAPSHOTS)
			.join(INITIAL_SNAPSHOT_ID)
			.join(FILE_SNAPSHOT_MESSAGES)
			.exists());

		let _ = fs::remove_dir_all(&target);
	}
}
