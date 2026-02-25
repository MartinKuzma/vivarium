use crate::core::errors::CoreError;
use crate::core::persistence::project::{ProjectContext, Snapshot};
use crate::core::persistence::schema::{DIR_SNAPSHOTS, FILE_SNAPSHOT_ENTITIES, FILE_SNAPSHOT_MANIFEST, FILE_SNAPSHOT_MESSAGES, ProjectManifest};

pub fn save_project_snapshot(project: &ProjectContext, snapshot_name: &str, snapshot: Snapshot) -> Result<(), CoreError> {
    let snapshot_dir = project.project_root.join(DIR_SNAPSHOTS).join(snapshot_name);
    std::fs::create_dir_all(&snapshot_dir)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create snapshot directory '{}': {}", snapshot_dir.display(), e)))?;

    let manifest_path = snapshot_dir.join(FILE_SNAPSHOT_MANIFEST);
    let entities_path = snapshot_dir.join(FILE_SNAPSHOT_ENTITIES);
    let messages_path = snapshot_dir.join(FILE_SNAPSHOT_MESSAGES);

    // save_yaml_file(&manifest_path, &snapshot.meta)?;
    // save_yaml_file(&entities_path, &snapshot.entities)?;
    // save_yaml_file(&messages_path, &snapshot.pending_messages)?;

    Ok(())
}

fn save_yaml_file<T: serde::ser::Serialize>(path: &std::path::Path, data: &T) -> Result<(), CoreError> {
    let yaml_string = serde_yaml::to_string(data)
        .map_err(|e| CoreError::SerializationError(format!("Failed to serialize data to YAML for '{}': {}", path.display(), e)))?;

    std::fs::write(path, yaml_string)
        .map_err(|e| CoreError::SerializationError(format!("Failed to write YAML file '{}': {}", path.display(), e)))?;

    Ok(())
}
