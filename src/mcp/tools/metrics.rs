use std::sync::Arc;
use std::sync::Mutex;

use crate::core::messaging::JSONObject;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

pub fn get_all_metrics(
    registry: &Arc<crate::core::registry::Registry>,
    world_name: String,
) -> Result<CallToolResult, McpError> {
    // actual implementation - no self, just plain function
    let world = registry.get(&world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", world_name),
            None,
        )
    })?;

    let world_guard = world.read().unwrap();
    let all_metrics = world_guard.get_metrics_ref().get_all_metrics();

    Ok(CallToolResult::success(vec![
        Content::json(&all_metrics).unwrap(),
    ]))
}
