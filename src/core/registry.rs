use std::collections::HashMap;
use crate::core::world::World;
use std::sync::{RwLock, Arc};
use crate::core::world_config::WorldCfg;
use crate::core::errors::CoreError;


// Registry for managing multiple simulations.
pub struct Registry {
    worlds: RwLock<HashMap<String, Arc<RwLock<World>>>>,
}

impl Registry {
    pub fn new() -> Self {
        Registry {
            worlds: RwLock::new(HashMap::new()),
        }
    }

    pub fn add(&self, name: String, world: World) -> Result<(), CoreError> {
        let mut self_worlds = self.worlds.write().unwrap();

        if self_worlds.contains_key(&name) {
            return Err(CoreError::WorldAlreadyExists);
        }

        self_worlds.insert(name, Arc::new(RwLock::new(world)));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Arc<RwLock<World>>, CoreError> {
        match self.worlds.read().unwrap().get(name) {
            Some(world) => Ok(world.clone()),
            None => Err(CoreError::WorldNotFound { name: name.to_string() }),
        }
    }

    pub fn delete(&self, name: &str) -> Result<(), CoreError> {
        if self.worlds.write().unwrap().remove(name).is_none() {
            return Err(CoreError::WorldNotFound { name: name.to_string() });
        }
        Ok(())
    }

    // pub fn copy(&self, source_name: &str, target_name: &str, replace: bool) -> Result<(), CoreError> {
    //     let source_world = self.get(source_name)?;

    //     let source_world_guard = source_world.read().unwrap();
    //     let snapshot = source_world_guard.create_snapshot()?;

    //     let mut target_worlds = self.worlds.write().unwrap();

    //     if !replace && target_worlds.contains_key(target_name) {
    //         return Err(CoreError::WorldAlreadyExists);
    //     }

    //     let target_world = World::new_from_snapshot(snapshot)?;
    //     target_worlds.insert(target_name.to_string(), Arc::new(RwLock::new(target_world)));
    //     Ok(())
    // }

    pub fn list(&self) -> Vec<String> {
        self.worlds.read().unwrap().keys().cloned().collect()
    }

    // pub fn restore_snapshot(&self, world_name: &str, snapshot: WorldSnapshot) -> Result<(), CoreError> {
    //     let restored_world = World::new_from_snapshot(snapshot)?;

    //     let mut worlds = self.worlds.write().unwrap();
    //     worlds.insert(world_name.to_string(), Arc::new(RwLock::new(restored_world)));
    //     Ok(())
    // }

    // pub fn get_snapshot(&self, world_name: &str) -> Result<WorldSnapshot, CoreError> {
    //     match self.worlds.read().unwrap().get(world_name) {
    //         Some(world) => {
    //             let world_guard = world.read().unwrap();
    //             world_guard.create_snapshot()
    //         },
    //         None => Err(CoreError::WorldNotFound { name: world_name.to_string() }),
    //     }
    // }

    // pub fn save_snapshot_to_file(&self, snapshot_name: &str, file_path: &str) -> Result<(), CoreError> {
    //     let self_snapshots = self.snapshots.read().unwrap();
    //     let snapshot =  self_snapshots.get(snapshot_name).ok_or(
    //         CoreError::SnapshotNotFound { name: snapshot_name.to_string() }
    //     )?;

    //     let serialized = serde_json::to_string_pretty(&snapshot)
    //     .map_err(|e| CoreError::SerializationError(format!("Failed to serialize snapshot: {}", e)))?;

    //     std::fs::write(file_path, serialized)
    //         .map_err(|e| CoreError::SerializationError(format!("Failed to write snapshot to file: {}", e)))?;

    //     Ok(())
    // }

    // pub fn load_snapshot_from_file(&self, file_path: &str) -> Result<(), CoreError> {
    //     let data = std::fs::read_to_string(file_path)
    //         .map_err(|e| CoreError::DeserializationError(format!("Failed to read snapshot file: {}", e)))?;

    //     let snapshot: WorldSnapshot = serde_json::from_str(&data)
    //         .map_err(|e| CoreError::DeserializationError(format!("Failed to deserialize snapshot: {}", e)))?;

    //     self.snapshots.write().unwrap().insert(file_path.to_string(), snapshot);
    //     Ok(())
    // }
}
