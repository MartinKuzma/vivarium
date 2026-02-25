use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PROJECT_SCHEMA_VERSION_V1: &str = "v1";
pub const DIR_SNAPSHOTS: &str = "snapshots";
pub const FILE_SNAPSHOT_MANIFEST: &str = "snapshot.yaml";
pub const FILE_SNAPSHOT_ENTITIES: &str = "entities.yaml";
pub const FILE_SNAPSHOT_MESSAGES: &str = "messages.yaml";

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Project manifest file loaded from world.yaml")]
pub struct ProjectManifest {
    #[schemars(description = "Manifest schema version")]
    pub schema_version: String,
    #[schemars(description = "Logical project/world name")]
    pub name: String,
    #[schemars(description = "Library of scripts keyed by script identifier")]
    pub script_library: HashMap<String, ManifestScriptCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Script definition entry in the project manifest")]
pub struct ManifestScriptCfg {
    #[schemars(description = "Unique script identifier")]
    pub id: String,
    #[schemars(description = "Script runtime kind; currently only 'lua'")]
    pub kind: String,
    #[schemars(description = "Relative path to the script file from project root")]
    pub script_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Snapshot metadata stored in snapshot.yaml")]
pub struct ManifestSnapshot {
    #[schemars(description = "Snapshot schema version")]
    pub schema_version: String,
    #[schemars(description = "Snapshot identifier, usually matching directory name")]
    pub id: String,
    #[schemars(description = "Simulation time at which the snapshot was captured")]
    pub simulation_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Entity list container stored in entities.yaml")]
pub struct ManifestEntities {
    #[schemars(description = "Entities restored into world configuration")]
    pub entities: Vec<ManifestEntityCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Message list container stored in messages.yaml")]
pub struct ManifestMessages {
    #[schemars(description = "Pending messages to be scheduled in the world")]
    pub messages: Vec<ManifestMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Single pending message record in snapshot messages")]
pub struct ManifestMessage {
    #[schemars(description = "Sender entity identifier")]
    pub sender: String,
    #[schemars(description = "Receiver entity identifier")]
    pub receiver: String,
    #[schemars(description = "Application-defined message kind")]
    pub kind: String,
    #[schemars(description = "JSON object payload")]
    pub content: serde_json::Map<String, serde_json::Value>,
    #[schemars(description = "Simulation step at which the message should be delivered")]
    pub receive_step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[schemars(description = "Entity configuration restored from snapshot")]
pub struct ManifestEntityCfg {
    #[schemars(description = "Unique entity identifier")]
    pub id: String,
    #[schemars(description = "Referenced script identifier from script_library")]
    pub script_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Optional initial state for the entity")]
    pub initial_state: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ProjectManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROJECT_SCHEMA_VERSION_V1 {
            return Err(format!(
                "Unsupported schema_version '{}'. Expected '{}'",
                self.schema_version, PROJECT_SCHEMA_VERSION_V1
            ));
        }

        if self.name.trim().is_empty() {
            return Err("Project name cannot be empty".to_string());
        }

        if self.script_library.is_empty() {
            return Err("Project script_library cannot be empty".to_string());
        }

        for (key, script_cfg) in &self.script_library {
            if key.trim().is_empty() {
                return Err("Script library contains an empty key".to_string());
            }

            script_cfg.validate()?;
            if script_cfg.id != *key {
                return Err(format!(
                    "Script library key '{}' does not match script id '{}'",
                    key, script_cfg.id
                ));
            }
        }
        Ok(())
    }
}

impl ManifestScriptCfg {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Script id cannot be empty".to_string());
        }

        if self.kind != "lua" {
            return Err(format!(
                "Unsupported script kind '{}'. Only 'lua' is currently supported",
                self.kind
            ));
        }

        if self.script_path.trim().is_empty() {
            return Err("Script must have a 'script_path' defined".to_string());
        }

        Ok(())
    }
}