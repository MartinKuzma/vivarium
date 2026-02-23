use super::schema::{
    ManifestEntities, ManifestMessages, ManifestSnapshot, ProjectManifest, DIR_SNAPSHOTS,
    FILE_SNAPSHOT_ENTITIES, FILE_SNAPSHOT_MANIFEST, FILE_SNAPSHOT_MESSAGES,
};
use crate::core::persistence::project::{LoadedProject, LoadedSnapshot};
use crate::core::{
    errors::CoreError,
    world_config::{EntityCfg, ScriptCfg, WorldCfg},
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum SnapshotSelection {
	Name(String), // Select a specific snapshot by name (directory name under snapshots/)
	Latest, // Automatically select the latest snapshot based on creation time of snapshot directories under snapshots/
}

pub fn load_project_from_manifest_file(manifest_path: &str, snapshot_selection: SnapshotSelection) -> Result<LoadedProject, CoreError> {
    let manifest_path = PathBuf::from(manifest_path);
    let project_root = manifest_path.parent().ok_or_else(|| {
        CoreError::DeserializationError(format!(
            "Unable to determine parent directory for manifest '{}'",
            manifest_path.display()
        ))
    })?;

    let manifest = load_manifest_from_yaml_file(&manifest_path)?;
    let snapshot = load_snapshot(project_root, snapshot_selection)?;
    let world_cfg = build_world_cfg_from_manifest(&manifest, project_root, snapshot)?;

    Ok(LoadedProject {
        project_root: project_root.to_path_buf(),
        manifest,
        world_cfg,
    })
}

fn load_manifest_from_yaml_file(path: &Path) -> Result<ProjectManifest, CoreError> {
    let manifest: ProjectManifest = load_yaml_file(path)?;

    manifest.validate().map_err(CoreError::DeserializationError)?;
    Ok(manifest)
}

fn build_world_cfg_from_manifest(
    manifest: &ProjectManifest,
    project_root: &Path,
    snapshot: LoadedSnapshot,
) -> Result<WorldCfg, CoreError> {
    let mut script_library = std::collections::HashMap::new();

    for (script_key, script_cfg) in &manifest.script_library {
        let script_content = load_script_file(&script_cfg.script_path, project_root)?;

        script_library.insert(
            script_key.clone(),
            ScriptCfg {
                id: script_cfg.id.clone(),
                kind: script_cfg.kind.clone(),
                script: script_content,
            },
        );
    }

    let cfg = WorldCfg {
        name: manifest.name.clone(),
        script_library,
        entities: snapshot.entities,
        pending_messages: snapshot.pending_messages,
        simulation_time: snapshot.meta.simulation_time,
    };

    cfg.validate()?;
    Ok(cfg)
}

fn load_script_file(script_path: &str, project_root: &Path) -> Result<String, CoreError> {
    let full_path = project_root.join(script_path);
    return std::fs::read_to_string(&full_path).map_err(|e| {
        CoreError::DeserializationError(format!(
            "Failed to read script_path '{}' (resolved '{}'): {}",
            script_path,
            full_path.display(),
            e
        ))
    });
}

fn load_snapshot(project_root: &Path, snapshot_selection: SnapshotSelection) -> Result<LoadedSnapshot, CoreError> {
    let full_path: PathBuf = project_root.join(DIR_SNAPSHOTS);

	let snapshot_dir = match snapshot_selection {
		SnapshotSelection::Name(name) => full_path.join(name),
		SnapshotSelection::Latest => find_latest_snapshot_in_dir(&full_path)?,
	};

	if !snapshot_dir.exists() || !snapshot_dir.is_dir() {
		return Err(CoreError::DeserializationError(format!(
			"Snapshot directory '{}' does not exist or is not a directory",
			snapshot_dir.display(),
		)));
	}

    let snapshot_path = snapshot_dir.join(FILE_SNAPSHOT_MANIFEST);
    let entities_path = snapshot_dir.join(FILE_SNAPSHOT_ENTITIES);
    let messages_path = snapshot_dir.join(FILE_SNAPSHOT_MESSAGES);

    let manifest_snapshot: ManifestSnapshot = load_yaml_file(&snapshot_path)?;
    let entities = load_entities(&entities_path)?;
    let pending_messages = load_messages(&messages_path)?;

    Ok(LoadedSnapshot {
        meta: manifest_snapshot,
        entities: entities,
        pending_messages: pending_messages,
		//TODO: Load metrics from snapshot as well
        //metrics: manifest_snapshot.metrics,
    })
}

fn load_entities(entities_path: &Path) -> Result<Vec<EntityCfg>, CoreError> {
    let manifest_entities: ManifestEntities = load_yaml_file(entities_path)?;
    let entities = manifest_entities
        .entities
        .into_iter()
        .map(|e| EntityCfg {
            id: e.id,
            script_id: e.script_id,
            initial_state: e.initial_state,
        })
        .collect::<Vec<_>>();

    Ok(entities)
}

fn load_messages(messages_path: &Path) -> Result<Vec<crate::core::messaging::Message>, CoreError> {
    let manifest_messages: ManifestMessages = load_yaml_file(messages_path)?;

    manifest_messages
        .messages
        .iter()
        .map(|m| {
            Ok(crate::core::messaging::Message {
                sender: m.sender.clone(),
                //TODO: Handle different receiver types (e.g., Broadcast, Group) when we add them to the schema
                receiver: crate::core::messaging::MessageReceiver::Entity {
                    id: m.receiver.clone(),
                },
                kind: m.kind.clone(),
                content: m.content.clone(),
                receive_step: m.receive_step,
            })
        })
        .collect()
}

fn load_yaml_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CoreError> {
    let data = std::fs::read_to_string(path).map_err(|e| {
        CoreError::DeserializationError(format!(
            "Failed to read YAML file '{}' (resolved '{}'): {}",
            path.display(),
            path.display(),
            e
        ))
    })?;

    serde_yaml::from_str(&data).map_err(|e| {
        CoreError::DeserializationError(format!(
            "Failed to parse YAML from file '{}' (resolved '{}'): {}",
            path.display(),
            path.display(),
            e
        ))
    })
}

fn find_latest_snapshot_in_dir(dir: &Path) -> Result<PathBuf, CoreError> {
    let mut snapshot_dirs = std::fs::read_dir(dir)
		.map_err(|e| CoreError::DeserializationError(format!(
			"Failed to read snapshot directory '{}': {}",
			dir.display(),
			e
		)))?		
		.filter_map(|entry| entry.ok())
		.filter(|entry| entry.path().is_dir())
		.collect::<Vec<_>>();

    snapshot_dirs.sort_by(|a, b| {
        let a_name = a.file_name();
        let b_name = b.file_name();
        b_name.to_string_lossy().cmp(&a_name.to_string_lossy())
    });

	for entry in snapshot_dirs {
        let snapshot_path = entry.path().join(FILE_SNAPSHOT_MANIFEST);
		if snapshot_path.exists() {
			return Ok(entry.path());
		}
	}

	Err(CoreError::DeserializationError(format!(
		"No valid snapshot found in directory '{}'",
		dir.display(),
	)))
}