use rmcp::schemars;
use crate::core::{errors::CoreError, messaging::JSONObject};
use std::collections::HashMap;


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct EntityCfg {
    #[schemars(description = "The unique ID of the entity")]
    pub id: String,
    #[schemars(description = "The ID of the script to use for this entity")]
    pub script_id: String,
    #[schemars(description = "Optional initial state for the entity as a JSON object")]
    pub initial_state: Option<JSONObject>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, schemars::JsonSchema)]
pub struct WorldCfg {
    #[schemars(description = "The name of the new world to create")]
    pub name: String,
    #[schemars(description = "The scripts available for entities in the world. Each entry is a tuple of (id, lua_script).")]
    pub script_library: HashMap<String, String>,
    #[schemars(description = "The entities to initialize in the new world")]
    pub entities: Vec<EntityCfg>,
}

impl WorldCfg {
    pub fn new(name: String) -> Self {
        WorldCfg {
            name,
            script_library: HashMap::new(),
            entities: Vec::new(),
        }
    }

    pub fn add_script(&mut self, id: String, lua_script: String) {
        self.script_library.insert(id, lua_script);
    }

    pub fn add_entity(&mut self, id: String, script_id: String) -> Result<(), CoreError> {
        // Is script defined?
        if !self.script_library.contains_key(&script_id) {
            return Err(CoreError::DeserializationError(format!("Script ID '{}' not found in script library", script_id)));
        }

        self.entities.push(EntityCfg { id, script_id, initial_state: None });
        Ok(())
    }

    // Update or insert entity
    pub fn upsert_entity(&mut self, id: &String, script_id: &String, initial_state: Option<JSONObject>) -> Result<(), CoreError> {
        // Is script defined?
        if !self.script_library.contains_key(script_id) {
            return Err(CoreError::DeserializationError(format!("Script ID '{}' not found in script library", script_id)));
        }

        if let Some(entity_cfg) = self.entities.iter_mut().find(|e| e.id.eq(id)) {
            entity_cfg.script_id = script_id.clone();
            entity_cfg.initial_state = initial_state;
        } else {
            self.entities.push(EntityCfg { id: id.clone(), script_id: script_id.clone(), initial_state: initial_state });
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<(), CoreError> {
        let mut script_ids = std::collections::HashSet::new();
        for id in self.script_library.keys() {
            if !script_ids.insert(id) {
                return Err(CoreError::DeserializationError(format!("Duplicate script ID found in script library: {}", id)));
            }
        }

        for entity in &self.entities {
            if !script_ids.contains(&entity.script_id) {
                return Err(CoreError::DeserializationError(format!("Entity '{}' references undefined script ID: {}", entity.id, entity.script_id)));
            }
        }

        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, CoreError> {
        let config_data = std::fs::read_to_string(path)
            .map_err(|e| CoreError::DeserializationError(format!("Failed to read world config file: {}", e)))?;
        let cfg: WorldCfg = serde_json::from_str(&config_data)
            .map_err(|e| CoreError::DeserializationError(format!("Failed to parse world config JSON: {}", e)))?;

        cfg.validate()?;
        Ok(cfg)
    }
}
