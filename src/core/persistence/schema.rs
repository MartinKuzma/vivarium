use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PROJECT_SCHEMA_VERSION_V1: &str = "v1";
pub const DIR_SNAPSHOTS: &str = "snapshots";
pub const FILE_SNAPSHOT_MANIFEST: &str = "snapshot.yaml";
pub const FILE_SNAPSHOT_ENTITIES: &str = "entities.yaml";
pub const FILE_SNAPSHOT_MESSAGES: &str = "messages.yaml";


#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ProjectManifest {
    pub schema_version: String,
    pub name: String,
    pub script_library: HashMap<String, ManifestScriptCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestScriptCfg {
    pub id: String,
    pub kind: String,
    pub script_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestSnapshot {
    pub schema_version: String,
    pub id: String,
    pub simulation_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestEntities {
    pub entities: Vec<ManifestEntityCfg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestMessages {
    pub messages: Vec<ManifestMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestMessage {
    pub sender: String,
    pub receiver: String,
    pub kind: String,
    pub content: serde_json::Map<String, serde_json::Value>,
    pub receive_step: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct ManifestEntityCfg {
    pub id: String,
    pub script_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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