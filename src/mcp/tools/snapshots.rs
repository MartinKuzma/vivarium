use crate::core::persistence::{loader, project::Snapshot, saver, schema};
use crate::mcp::project_store::ProjectStore;
use rmcp::Json;
use rmcp::{ErrorData as McpError, schemars};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct ListProjectSnapshotsRequest {
    #[schemars(description = "Loaded project name")]
    pub project_name: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct ListProjectSnapshotsResponse {
    #[schemars(description = "Available snapshot names (directory names under snapshots/)")]
    pub snapshots: Vec<String>,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SaveProjectSnapshotRequest {
    #[schemars(description = "Loaded project name")]
    pub project_name: String,
    #[schemars(description = "Snapshot name (directory name under snapshots/)")]
    pub snapshot_name: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct SaveProjectSnapshotResponse {
    #[schemars(description = "Success message")]
    pub message: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LoadProjectSnapshotRequest {
    #[schemars(description = "Loaded project name")]
    pub project_name: String,
    #[serde(default = "default_snapshot_selection")]
    #[schemars(description = "Snapshot to load: use 'latest' or a specific snapshot name")]
    pub snapshot: String,
    #[serde(default)]
    #[schemars(description = "Reset metrics when loading snapshot")]
    pub reset_metrics: bool,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct LoadProjectSnapshotResponse {
    #[schemars(description = "Success message")]
    pub message: String,
}

fn default_snapshot_selection() -> String {
    "latest".to_string()
}

pub fn list_project_snapshots(
    store: &ProjectStore,
    request: ListProjectSnapshotsRequest,
) -> Result<Json<ListProjectSnapshotsResponse>, McpError> {
    let project_ctx = store.get_project_context(&request.project_name)?;
    let snapshots_dir = project_ctx.project_root.join(schema::DIR_SNAPSHOTS);

    let mut snapshots = std::fs::read_dir(&snapshots_dir)
        .map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!(
                    "Failed to read snapshots directory '{}': {}",
                    snapshots_dir.display(),
                    e
                ),
                None,
            )
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let snapshot_manifest = entry.path().join(schema::FILE_SNAPSHOT_MANIFEST);
            if snapshot_manifest.exists() {
                Some(entry.file_name().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    snapshots.sort_by(|a, b| b.cmp(a));

    Ok(Json(ListProjectSnapshotsResponse { snapshots }))
}

pub fn save_project_snapshot(
    store: &ProjectStore,
    request: SaveProjectSnapshotRequest,
) -> Result<Json<SaveProjectSnapshotResponse>, McpError> {
    let world_arc = store.get(&request.project_name)?;
    let world = world_arc.read().unwrap();
    let project_ctx = store.get_project_context(&request.project_name)?;

    let snapshot = Snapshot {
        meta: schema::ManifestSnapshot {
            schema_version: schema::PROJECT_SCHEMA_VERSION_V1.to_string(),
            id: request.snapshot_name.clone(),
            simulation_time: world.get_simulation_time(),
            metrics: Some(world.get_metrics_snapshot()),
        },
        simulation_time: world.get_simulation_time(),
        entities: world.get_entities_snapshot()?,
        pending_messages: world.get_pending_messages(),
        metrics: Some(world.get_metrics_snapshot()),
    };

    saver::save_project_snapshot(&project_ctx, &request.snapshot_name, snapshot)?;

    Ok(Json(SaveProjectSnapshotResponse {
        message: format!(
            "Snapshot '{}' saved for project '{}'",
            request.snapshot_name, request.project_name
        ),
    }))
}

pub fn load_project_snapshot(
    store: &ProjectStore,
    request: LoadProjectSnapshotRequest,
) -> Result<Json<LoadProjectSnapshotResponse>, McpError> {
    let project_ctx = store.get_project_context(&request.project_name)?;
    let snapshot_selection = request
        .snapshot
        .parse::<loader::SnapshotSelection>()
        .map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Invalid snapshot selection '{}': {}", request.snapshot, e),
                None,
            )
        })?;

    let snapshot = loader::load_snapshot(&project_ctx, snapshot_selection)?;
    let world_data = crate::core::WorldSnapshotData {
        name: project_ctx.manifest.name.clone(),
        script_library: project_ctx.script_library.clone(),
        entities: snapshot.entities,
        pending_messages: snapshot.pending_messages,
        metrics: if request.reset_metrics {
            None
        } else {
            snapshot.metrics
        },
        simulation_time: snapshot.simulation_time,
    };

    let world = crate::core::World::new(world_data)?;
    store.replace_world(&request.project_name, world)?;

    Ok(Json(LoadProjectSnapshotResponse {
        message: format!(
            "Snapshot '{}' loaded into project '{}'",
            request.snapshot, request.project_name
        ),
    }))
}