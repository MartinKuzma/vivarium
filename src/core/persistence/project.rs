use crate::core::errors::CoreError;
use crate::core::persistence::schema::{ManifestSnapshot, ProjectManifest};
use crate::core::world_config::{EntityCfg};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub project_root: PathBuf,
    pub manifest: ProjectManifest,
    pub script_library: std::collections::HashMap<String, crate::core::world_config::ScriptCfg>,
}

#[derive(Debug, Clone)]
pub struct Snapshot {
	pub meta : ManifestSnapshot,
    pub simulation_time: u64,
	pub entities: Vec<EntityCfg>,
	pub pending_messages: Vec<crate::core::messaging::Message>,
	//metrics: snapshot::MetricsSnapshot,
}