use rmcp::schemars;
use crate::core::errors::CoreError;
use std::collections::HashMap;


#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EntityCfg {
    pub id: String,
    pub script_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BulkEntityCfg {
    pub count: usize,
    pub script_id: String,
    pub id_prefix: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WorldCfg {
    #[schemars(description = "The name of the new world to create")]
    pub name: String,
    #[schemars(description = "The scripts available for entities in the world. Each entry is a tuple of (id, lua_script).")]
    pub script_library: HashMap<String, String>,
    #[schemars(description = "The entities to initialize in the new world")]
    pub entities: Vec<EntityCfg>,
    #[schemars(description = "Bulk entity creation configurations")]
    pub bulk_entities: Vec<BulkEntityCfg>,
}

impl WorldCfg {
    pub fn new(name: String) -> Self {
        WorldCfg {
            name,
            script_library: HashMap::new(),
            entities: Vec::new(),
            bulk_entities: Vec::new(),
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

        self.entities.push(EntityCfg { id, script_id });
        Ok(())
    }

    pub fn add_bulk_entities(&mut self, count: usize, script_id: String, id_prefix: String) -> Result<(), CoreError> {
        // Is script defined?
        if !self.script_library.contains_key(&script_id) {
            return Err(CoreError::DeserializationError(format!("Script ID '{}' not found in script library", script_id)));
        }

        self.bulk_entities.push(BulkEntityCfg { count, script_id, id_prefix });
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

        for bulk_entity in &self.bulk_entities {
            if !script_ids.contains(&bulk_entity.script_id) {
                return Err(CoreError::DeserializationError(format!("Bulk entity configuration references undefined script ID: {}", bulk_entity.script_id)));
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