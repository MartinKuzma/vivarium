use crate::core::{
    errors::CoreError, snapshot, world::World, world_config::{EntityCfg, ScriptCfg, WorldCfg}
};
use crate::core::persistence::project::{LoadedProject, LoadedSnapshot};
use std::path::{Path, PathBuf};
use super::schema::{ProjectManifest, ManifestEntityCfg, ManifestEntities, ManifestScriptCfg, ManifestSnapshot};

pub fn load_project_from_manifest_file(manifest_path: &str) -> Result<LoadedProject, CoreError> {
    let manifest_path = PathBuf::from(manifest_path);
    let project_root = manifest_path.parent().ok_or_else(|| {
        CoreError::DeserializationError(format!(
            "Unable to determine parent directory for manifest '{}'",
            manifest_path.display()
        ))
    })?;

    let manifest = load_manifest_from_yaml_file(&manifest_path)?;
	let snapshot = load_snapshot(&manifest.initial_snapshot_dir, project_root)?;

    let world_cfg = build_world_cfg_from_manifest(&manifest, project_root, &snapshot)?;	

    Ok(LoadedProject {
        project_root: project_root.to_path_buf(),
        manifest,
        world_cfg,
        snapshot,
    })
}

pub fn load_manifest_from_yaml_file(path: &Path) -> Result<ProjectManifest, CoreError> {
	let manifest : ProjectManifest = load_yaml_file(path)?;
    
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
    };

    cfg.validate()?;
    Ok(cfg)
}

fn load_script_file(script_path: &String, project_root: &Path) -> Result<String, CoreError> {
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



fn load_snapshot(snapshot_dir: &String, project_root: &Path) -> Result<LoadedSnapshot, CoreError> {
	let full_path = project_root.join(snapshot_dir);
	
	let snapshot_path = full_path.join("snapshot.yaml");
	let entities_path = full_path.join("entities.yaml");
	let messages_path = full_path.join("messages.yaml");

	let manifest_snapshot: ManifestSnapshot = load_yaml_file(&snapshot_path)?;
	let entities = load_entities(&entities_path)?;
	let pending_messages = load_messages(&messages_path)?;

	Ok(LoadedSnapshot {
		meta: manifest_snapshot,
		entities: entities,
		pending_messages: pending_messages,
		//metrics: manifest_snapshot.metrics,
	})
}

fn load_entities(entities_path: &PathBuf) -> Result<Vec<EntityCfg>, CoreError> {
	let manifest_entities: ManifestEntities = load_yaml_file(&PathBuf::from(entities_path))?;
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

fn load_messages(messages_path: &PathBuf) -> Result<Vec<crate::core::messaging::Message>, CoreError> {
	let manifest_messages: ManifestMessages = load_yaml_file(&PathBuf::from(messages_path))?;
	Ok(manifest_messages.messages)
}

fn load_or_default<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, CoreError> {
	if path.exists() {
		load_yaml_file(path)
	} else {
		serde_yaml::from_str("null").map_err(|e| {
			CoreError::DeserializationError(format!(
				"Failed to parse default value for missing file '{}': {}",
				path.display(),
				e
			))
		})
	}
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