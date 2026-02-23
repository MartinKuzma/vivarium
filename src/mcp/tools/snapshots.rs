use rmcp::Json;
use rmcp::{ErrorData as McpError, schemars};

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct CreateSnapshotRequest {
    #[schemars(description = "The name of the simulation world to snapshot")]
    pub world_name: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct CreateSnapshotResponse {
    #[schemars(description = "The success message confirming snapshot creation")]
    pub success_message: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct SaveSnapshotToFileRequest {
    #[schemars(description = "The name of the simulation world to snapshot")]
    pub world_name: String,
    #[schemars(description = "The file path to save the snapshot to. Use a .yaml extension.")]
    pub file_path: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct SaveSnapshotToFileResponse {
    #[schemars(description = "The file path where the snapshot was saved")]
    pub file_path: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct RestoreSnapshotResponse {
    #[schemars(description = "Success message")]
    pub message: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct RestoreSnapshotRequest {
    #[schemars(description = "The name of the simulation world to restore the snapshot into")]
    pub world_name: String,
    #[schemars(description = "The success message confirming snapshot restoration")]
    pub success_message: String,
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
pub struct LoadSnapshotFromFileRequest {
    #[schemars(description = "The name of the simulation world to load the snapshot into")]
    pub world_name: String,
    #[schemars(description = "The file path of yaml snapshot to load")]
    pub file_path: String,
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct LoadSnapshotFromFileResponse {
    #[schemars(description = "Success message")]
    pub message: String,
}

// pub fn create_snapshot(
//     registry: &crate::core::registry::Registry,
//     request: CreateSnapshotRequest,
// ) -> Result<Json<CreateSnapshotResponse>, McpError> {
//     let snapshot = registry.get_snapshot(&request.world_name).map_err(|e| {
//         McpError::new(
//             rmcp::model::ErrorCode::INVALID_PARAMS,
//             format!(
//                 "Failed to create snapshot for world '{}': {}",
//                 request.world_name, e
//             ),
//             None,
//         )
//     })?;

//     Ok(Json(CreateSnapshotResponse { snapshot }))
// }

// pub fn restore_snapshot(
//     registry: &crate::core::registry::Registry,
//     request: RestoreSnapshotRequest,
// ) -> Result<Json<RestoreSnapshotResponse>, McpError> {
//     registry
//         .restore_snapshot(&request.world_name, request.snapshot)
//         .map_err(|e| {
//             McpError::new(
//                 rmcp::model::ErrorCode::INVALID_PARAMS,
//                 format!(
//                     "Failed to restore snapshot into world '{}': {}",
//                     request.world_name, e
//                 ),
//                 None,
//             )
//         })?;

//     Ok(Json(RestoreSnapshotResponse {
//         message: format!("Snapshot restored into world '{}'", request.world_name),
//     }))
// }

// pub fn save_snapshot_to_file(
//     registry: &crate::core::registry::Registry,
//     request: SaveSnapshotToFileRequest,
// ) -> Result<Json<SaveSnapshotToFileResponse>, McpError> {
//     let snapshot = registry.get_snapshot(&request.world_name).map_err(|e| {
//         McpError::new(
//             rmcp::model::ErrorCode::INVALID_PARAMS,
//             format!(
//                 "Failed to create snapshot for world '{}': {}",
//                 request.world_name, e
//             ),
//             None,
//         )
//     })?;

//     snapshot.to_yaml_file(&request.file_path).map_err(|e| {
//         McpError::new(
//             rmcp::model::ErrorCode::INTERNAL_ERROR,
//             format!(
//                 "Failed to save snapshot to file '{}': {}",
//                 request.file_path, e
//             ),
//             None,
//         )
//     })?;

//     Ok(Json(SaveSnapshotToFileResponse {
//         file_path: request.file_path,
//     }))
// }

// pub fn load_snapshot_from_file(
//     registry: &crate::core::registry::Registry,
//     request: LoadSnapshotFromFileRequest,
// ) -> Result<Json<LoadSnapshotFromFileResponse>, McpError> {

//     let snapshot = WorldSnapshot::from_yaml_file(&request.file_path)?;
//     registry.restore_snapshot(&request.world_name, snapshot)?;
//     Ok(Json(LoadSnapshotFromFileResponse {
//         message: format!("Snapshot loaded from file '{}' into '{}'", request.file_path, request.world_name),
//     }))
// }