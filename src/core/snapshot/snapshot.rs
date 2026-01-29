use crate::core::{messaging::Message, world_config::WorldCfg};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rmcp::schemars;

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct WorldSnapshot {
    
    pub configuration: WorldCfg,
    pub simulation_time: u64, // Simulation time at which the snapshot was taken
    pub metrics: MetricsSnapshot,
    pub pending_messages: Vec<Message>,
}

#[derive(Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MetricsSnapshot {
    pub metrics: HashMap<String, Vec<(u64, f64)>>, // Metric name to list of (timestamp, value) pairs
}

impl WorldSnapshot {
    pub fn new(
        configuration: WorldCfg,
        simulation_time: u64,
        pending_messages: Vec<Message>,
        metrics: MetricsSnapshot,
    ) -> Self {
        WorldSnapshot {
            configuration,
            simulation_time,
            pending_messages,
            metrics,
        }
    }
}
