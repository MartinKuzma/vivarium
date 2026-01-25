pub struct Manager {
    snapshots: HashMap<String, WorldSnapshot>, // Map of snapshot name to WorldSnapshot
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            snapshots: HashMap::new(),
        }
    }

    // Take a snapshot of the world and store it with the given name
    pub fn take_snapshot(&mut self, name: &str, world: &World) -> Result<(), crate::core::errors::CoreError> {
        let snapshot = world.create_snapshot()?;

        self.snapshots.insert(name.to_string(), snapshot.clone());
        Ok(())
    }

    // Restore the world from a snapshot with the given name
    pub fn restore_snapshot(&mut self, name: &str) -> Result<World, crate::core::errors::CoreError> {
        let snapshot = self.snapshots.get(name).expect("Snapshot not found");
        World::new_from_snapshot(snapshot.clone())?; 
    }
}