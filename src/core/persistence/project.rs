use rmcp::schemars;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::world_config::{EntityCfg, ScriptCfg, WorldCfg};
use crate::core::errors::CoreError;
use crate::core::persistence::schema::{ProjectManifest, ManifestSnapshot, ManifestMessages};

#[derive(Debug, Clone)]
pub struct LoadedProject {
    pub project_root: PathBuf,
    pub manifest: ProjectManifest,
    pub world_cfg: WorldCfg,
    pub snapshot : LoadedSnapshot,
}

impl LoadedProject {
    pub fn instantiate_world(&self) -> Result<crate::core::world::World, CoreError> {
        crate::core::world::World::new(&self.world_cfg)
    }
}

#[derive(Debug, Clone)]
pub struct LoadedSnapshot {
	pub meta : ManifestSnapshot,
	pub entities: Vec<EntityCfg>,
	pub pending_messages: Vec<crate::core::messaging::Message>,
	//metrics: snapshot::MetricsSnapshot,
}