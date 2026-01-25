use std::collections::HashMap;
use serde::Serialize;
use crate::core::messaging::{JSONObject, Message};
use crate::core::world::World;

pub struct SnapshotManager {
    snapshots: HashMap<String, WorldSnapshot>, // Map of snapshot name to WorldSnapshot
}

#[derive(Serialize)]
pub struct WorldSnapshot {
    pub simulation_time: u64, // Simulation time at which the snapshot was taken
    pub entities: Vec<EntitySnapshot>, // Placeholder for entity snapshots
    pub messages: Vec<Message>,        // Placeholder for message snapshots
    pub description: String, // Holds a description of the snapshot set by the user
}

impl WorldSnapshot {
    pub fn new(
        simulation_time: u64,
        entities: Vec<EntitySnapshot>,
        messages: Vec<Message>,
        description: String,
    ) -> Self {
        WorldSnapshot {
            simulation_time,
            entities,
            messages,
            description,
        }
    }
}

#[derive(Serialize)]
pub struct EntitySnapshot {
    pub id: String,
    pub script: String,
    pub state: JSONObject, // Serialized state of the entity
}


impl EntitySnapshot {
    pub fn new(id: String, script: String, state: JSONObject) -> Self {
        EntitySnapshot {
            id,
            script,
            state,
        }
    }
}

impl SnapshotManager {
    pub fn new() -> Self {
        SnapshotManager {
            snapshots: HashMap::new(),
        }
    }

    pub fn take_snapshot(&mut self, world: &World) {
        todo!("Implement snapshot taking");
    }

    pub fn restore_snapshot(&mut self, name: &str) -> World {
        todo!("Implement snapshot restoration");
    }
}
