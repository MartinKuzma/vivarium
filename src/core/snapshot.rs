use std::collections::HashMap;

use crate::core::messaging::Message;
use crate::core::world::World;

pub struct SnapshotManager {
    snapshots: HashMap<String, WorldSnapshot>, // Map of snapshot name to WorldSnapshot
}

struct WorldSnapshot {
    name : String, // Name of the snapshot, must be unique
    simulation_time: u64, // Simulation time at which the snapshot was taken
    entities: Vec<EntitySnapshot>, // Placeholder for entity snapshots
    messages: Vec<Message>,        // Placeholder for message snapshots
    description: String, // Holds a description of the snapshot set by the user
}

struct EntitySnapshot {
    id: String,
    state: String, // Serialized state of the entity
}

impl SnapshotManager {
    pub fn new() -> Self {
        SnapshotManager {
            snapshots: HashMap::new(),
        }
    }

    pub fn take_snapshot(&mut self, world: &mut World) {
        todo!("Implement snapshot taking");
    }

    pub fn restore_snapshot(&mut self, name: &str) -> World {
        todo!("Implement snapshot restoration");
    }

}
