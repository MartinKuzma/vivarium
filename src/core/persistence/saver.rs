use crate::core::errors::CoreError;
use crate::core::persistence::project::{ProjectContext, Snapshot};
use crate::core::persistence::schema::{
    DIR_SNAPSHOTS, FILE_SNAPSHOT_ENTITIES, FILE_SNAPSHOT_MANIFEST, FILE_SNAPSHOT_MESSAGES,
    ManifestEntities, ManifestEntityCfg, ManifestMessage, ManifestMessages, ManifestSnapshot,
};

pub fn save_project_snapshot(project: &ProjectContext, snapshot_name: &str, snapshot: Snapshot) -> Result<(), CoreError> {
    let snapshot_dir = project.project_root.join(DIR_SNAPSHOTS).join(snapshot_name);
    std::fs::create_dir_all(&snapshot_dir)
        .map_err(|e| CoreError::SerializationError(format!("Failed to create snapshot directory '{}': {}", snapshot_dir.display(), e)))?;

    let manifest_path = snapshot_dir.join(FILE_SNAPSHOT_MANIFEST);
    let entities_path = snapshot_dir.join(FILE_SNAPSHOT_ENTITIES);
    let messages_path = snapshot_dir.join(FILE_SNAPSHOT_MESSAGES);

    let manifest = ManifestSnapshot {
        schema_version: snapshot.meta.schema_version,
        id: snapshot_name.to_string(),
        simulation_time: snapshot.simulation_time,
        metrics: snapshot.metrics,
    };

    let entities = ManifestEntities {
        entities: snapshot
            .entities
            .into_iter()
            .map(|entity| ManifestEntityCfg {
                id: entity.id,
                script_id: entity.script_id,
                initial_state: entity.initial_state,
            })
            .collect(),
    };

    let mut manifest_messages = Vec::new();
    for message in snapshot.pending_messages {
        let receiver = match message.receiver {
            crate::core::messaging::MessageReceiver::Entity { id } => id,
            crate::core::messaging::MessageReceiver::Radius2D { .. } => {
                return Err(CoreError::SerializationError(
                    "Cannot persist snapshot message with Radius2D receiver to current schema"
                        .to_string(),
                ));
            }
        };

        manifest_messages.push(ManifestMessage {
            sender: message.sender,
            receiver,
            kind: message.kind,
            content: message.content,
            receive_step: message.receive_step,
        });
    }

    let messages = ManifestMessages {
        messages: manifest_messages,
    };

    save_yaml_file(&manifest_path, &manifest)?;
    save_yaml_file(&entities_path, &entities)?;
    save_yaml_file(&messages_path, &messages)?;

    Ok(())
}

fn save_yaml_file<T: serde::ser::Serialize>(path: &std::path::Path, data: &T) -> Result<(), CoreError> {
    let yaml_string = serde_yaml::to_string(data)
        .map_err(|e| CoreError::SerializationError(format!("Failed to serialize data to YAML for '{}': {}", path.display(), e)))?;

    std::fs::write(path, yaml_string)
        .map_err(|e| CoreError::SerializationError(format!("Failed to write YAML file '{}': {}", path.display(), e)))?;

    Ok(())
}
