use rmcp::Json;
use rmcp::ErrorData as McpError;
use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListMetricsRequest {
    #[schemars(description = "The name of the simulation world to query")]
    pub world_name: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ListMetricsResponse {
    #[schemars(description = "List of metric names")]
    pub metrics: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMetricsRequest {
    #[schemars(description = "The name of the simulation world to query")]
    pub world_name: String,
    #[schemars(description = "List of metric names to retrieve")]
    pub metrics: Vec<String>,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct GetMetricsResponse {
    #[schemars(description = "List of metric statistics")]
    pub metrics: Vec<crate::core::metrics::MetricStats>,
}

pub fn list_metrics(
    registry: &crate::core::registry::Registry,
    request: ListMetricsRequest,
) -> Result<Json<ListMetricsResponse>, McpError> {
    let world = registry.get(&request.world_name)?;

    let world_guard = world.read().unwrap();
    let metrics = world_guard.get_metrics_ref().list_metric_names();

    Ok(Json(ListMetricsResponse { metrics }))
}

pub fn get_metric(
    registry: &crate::core::registry::Registry,
    world_name: String,
    metric_name: String,
) -> Result<Json<crate::core::metrics::MetricStats>, McpError> {
    let world = registry.get(&world_name)?;

    let world_guard = world.read().unwrap();
    let metric_stats = world_guard.get_metrics_ref().compute_metric_stats(&metric_name)
        .ok_or_else(|| McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("Metric '{}' not found in world '{}'", metric_name, world_name),
            None,
        ))?;

    Ok(Json(metric_stats))
}

pub fn get_metrics(
    registry: &crate::core::registry::Registry,
    request: GetMetricsRequest,
) -> Result<Json<GetMetricsResponse>, McpError> {
    let world = registry.get(&request.world_name)?;

    let world_guard = world.read().unwrap();
    let mut metrics = Vec::new();

    for metric_name in request.metrics {
        match world_guard.get_metrics_ref().compute_metric_stats(&metric_name) {
            Some(stats) => metrics.push(stats),
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

    Ok(Json(GetMetricsResponse { metrics }))
}
