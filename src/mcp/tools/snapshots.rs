use crate::core::snapshot::WorldSnapshot;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{ErrorData as McpError, schemars};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateSnapshotRequest {
    #[schemars(description = "The name of the simulation world to snapshot")]
    pub world_name: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct CreateSnapshotResponse {
    #[schemars(description = "The snapshot data")]
    pub snapshot: WorldSnapshot,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RestoreSnapshotRequest {
    #[schemars(description = "The name of the simulation world to restore the snapshot into")]
    pub world_name: String,
    #[schemars(description = "The snapshot data")]
    pub snapshot: WorldSnapshot,
}

pub fn create_snapshot(
    registry: &crate::core::registry::Registry,
    request: CreateSnapshotRequest,
) -> Result<CallToolResult, McpError> {
    let snapshot = registry.get_snapshot(&request.world_name).map_err(|e| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!(
                "Failed to create snapshot for world '{}': {}",
                request.world_name, e
            ),
            None,
        )
    })?;

    Ok(CallToolResult::success(vec![
        Content::json(CreateSnapshotResponse { snapshot }).unwrap(),
    ]))
}

pub fn restore_snapshot(
    registry: &crate::core::registry::Registry,
    request: RestoreSnapshotRequest,
) -> Result<CallToolResult, McpError> {
    registry
        .restore_snapshot(&request.world_name, request.snapshot)
        .map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Failed to restore snapshot into world '{}': {}",
                    request.world_name, e
                ),
                None,
            )
        })?;

    Ok(CallToolResult::success(vec![
        Content::json(&format!(
            "Snapshot restored into world '{}'",
            request.world_name
        ))
        .unwrap(),
    ]))
}
