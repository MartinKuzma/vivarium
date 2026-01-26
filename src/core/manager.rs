use std::collections::HashMap;
use crate::core::world::World;
use crate::core::snapshot::WorldSnapshot;
use std::sync::{RwLock, Arc};


pub struct WorldManager {
    worlds: RwLock<HashMap<String, Arc<RwLock<World>>>>,
    snapshots : RwLock<HashMap<String, WorldSnapshot>>,
}

impl WorldManager {
    pub fn new() -> Self {
        WorldManager {
            worlds: RwLock::new(HashMap::new()),
            snapshots: RwLock::new(HashMap::new()),
        }
    }

    pub fn create(&self, name: &str) -> Result<(), crate::core::errors::CoreError> {
        let world = World::new();

        let mut self_worlds = self.worlds.write().unwrap();

        if self_worlds.contains_key(name) {
            return Err(crate::core::errors::CoreError::WorldAlreadyExists);
        }

        self_worlds.insert(name.to_string(), Arc::new(RwLock::new(world)));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<Arc<RwLock<World>>> {
        self.worlds.read().unwrap().get(name).cloned()
    }

    pub fn delete(&self, name: &str) -> Result<(), crate::core::errors::CoreError> {
        if self.worlds.write().unwrap().remove(name).is_none() {
            return Err(crate::core::errors::CoreError::WorldNotFound { name: name.to_string() });
        }
        Ok(())
    }

    pub fn copy(&self, source_name: &str, target_name: &str, replace: bool) -> Result<(), crate::core::errors::CoreError> {
        let source_world = self.get(source_name).ok_or(
            crate::core::errors::CoreError::WorldNotFound { name: source_name.to_string() }
        )?;

        let source_world_guard = source_world.read().unwrap();
        let snapshot = source_world_guard.create_snapshot()?;

        let mut target_worlds = self.worlds.write().unwrap();

        if !replace && target_worlds.contains_key(target_name) {
            return Err(crate::core::errors::CoreError::WorldAlreadyExists);
        }

        let target_world = World::new_from_snapshot(&snapshot)?;
        target_worlds.insert(target_name.to_string(), Arc::new(RwLock::new(target_world)));
        Ok(())
    }

    pub fn list(&self) -> Vec<String> {
        self.worlds.read().unwrap().keys().cloned().collect()
    }

    pub fn take_snapshot(&self, world_name: &str, snapshot_name: &str) -> Result<(), crate::core::errors::CoreError> {
        let snapshot = self.worlds.read().unwrap().get(world_name).unwrap().read().unwrap().create_snapshot()?;
        self.snapshots.write().unwrap().insert(snapshot_name.to_string(), snapshot);
        Ok(())
    }

    pub fn restore_snapshot(&self, world_name: &str, snapshot_name: &str) -> Result<(), crate::core::errors::CoreError> {
        let self_snapshots = self.snapshots.read().unwrap();

        let snapshot = self_snapshots.get(snapshot_name).ok_or(
            crate::core::errors::CoreError::SnapshotNotFound { name: snapshot_name.to_string() }
        )?;

        let restored_world = World::new_from_snapshot(snapshot)?;

        let mut worlds = self.worlds.write().unwrap();
        worlds.insert(world_name.to_string(), Arc::new(RwLock::new(restored_world)));
        Ok(())
    }
}