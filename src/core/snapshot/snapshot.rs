use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::messaging::{JSONObject, Message};

#[derive(Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub simulation_time: u64, // Simulation time at which the snapshot was taken
    pub entities: Vec<EntitySnapshot>, // Placeholder for entity snapshots
    pub messages: Vec<Message>,        // Placeholder for message snapshots
    pub description: String, // Holds a description of the snapshot set by the user
    pub metrics : MetricsSnapshot,
}

#[derive(Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub metrics: HashMap<String, Vec<(u64, f64)>>, // Metric name to list of (timestamp, value) pairs
}

impl WorldSnapshot {
    pub fn new(
        simulation_time: u64,
        entities: Vec<EntitySnapshot>,
        messages: Vec<Message>,
        description: String,
        metrics: MetricsSnapshot,
    ) -> Self {
        WorldSnapshot {
            simulation_time,
            entities,
            messages,
            description,
            metrics,
        }
    }
}

#[derive(Serialize, Deserialize)]
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