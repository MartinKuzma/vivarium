use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListMetricsRequest {
    #[schemars(description = "The name of the simulation world to query")]
    pub world_name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMetricsRequest {
    #[schemars(description = "The name of the simulation world to query")]
    pub world_name: String,
    #[schemars(description = "List of metric names to retrieve")]
    pub metrics: Vec<String>,
}

pub fn list_metrics(
    registry: &crate::core::registry::Registry,
    request: ListMetricsRequest,
) -> Result<CallToolResult, McpError> {
    let world = registry.get(&request.world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", request.world_name),
            None,
        )
    })?;

    let world_guard = world.read().unwrap();
    let metric_names = world_guard.get_metrics_ref().list_metric_names();

    Ok(CallToolResult::success(vec![
        Content::json(&metric_names).unwrap(),
    ]))
}

pub fn get_metric(
    registry: &crate::core::registry::Registry,
    world_name: String,
    metric_name: String,
) -> Result<CallToolResult, McpError> {
    let world = registry.get(&world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", world_name),
            None,
        )
    })?;

    let world_guard = world.read().unwrap();
    let metric_stats = match world_guard.get_metrics_ref().compute_metric_stats(&metric_name) {
        Some(_) => world_guard.get_metrics_ref().compute_metric_stats(&metric_name),
        None => {
            return Err(McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Metric '{}' not found in world '{}'", metric_name, world_name),
                None,
            ));
        }
    };

    Ok(CallToolResult::success(vec![
        Content::json(&metric_stats).unwrap(),
    ]))
}

pub fn get_metrics(
    registry: &crate::core::registry::Registry,
    request: GetMetricsRequest,
) -> Result<CallToolResult, McpError> {
    let world = registry.get(&request.world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", request.world_name),
            None,
        )
    })?;

    let world_guard = world.read().unwrap();
    let mut metrics_stats = Vec::new();

    for metric_name in request.metrics {
        match world_guard.get_metrics_ref().compute_metric_stats(&metric_name) {
            Some(stats) => metrics_stats.push(stats),
            None => {
                return Err(McpError::new(
                    rmcp::model::ErrorCode::INVALID_PARAMS,
                    format!(
                        "Metric '{}' not found in world '{}'",
                        metric_name, request.world_name
                    ),
                    None,
                ));
            }
        }
    }

    Ok(CallToolResult::success(vec![
        Content::json(&metrics_stats).unwrap(),
    ]))
}
