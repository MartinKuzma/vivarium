use crate::mcp::tools::world;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

const SERVER_INSTRUCTIONS: &str = include_str!("../../docs/mcp/instructions.md");

pub struct VivariumToolServer {
    pub tool_router: ToolRouter<Self>,
    world_registry: crate::core::registry::Registry,
}

#[tool_router]
impl VivariumToolServer {
    pub fn new(world_registry: crate::core::registry::Registry) -> Self {
        let tool_router = Self::tool_router();

        VivariumToolServer {
            tool_router,
            world_registry,
        }
    }

    #[tool(description = "Create a new simulation world with the specified configuration")]
    fn create_world(
        &self,
        Parameters(config): Parameters<crate::core::world_config::WorldCfg>,
    ) -> Result<rmcp::Json<world::CreateWorldResponse>, McpError> {
        self.world_registry.create(config).map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to create world: {}", e),
                None,
            )
        })?;

        Ok(rmcp::Json(world::CreateWorldResponse {
            message: "World created successfully".to_string(),
        }))
    }

    #[tool(description = "Delete an existing simulation world by name")]
    fn delete_world(&self, Parameters(name): Parameters<String>) -> Result<rmcp::Json<world::DeleteWorldResponse>, McpError> {
        self.world_registry.delete(&name).map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to delete world '{}': {}", name, e),
                None,
            )
        })?;

        Ok(rmcp::Json(world::DeleteWorldResponse {
            message: format!("World '{}' deleted successfully", name),
        }))
    }

    #[tool(
        description = "Copy an existing simulation world to a new world with the specified name"
    )]
    fn copy_world(
        &self,
        Parameters(request): Parameters<world::CopyWorldRequest>,
    ) -> Result<rmcp::Json<world::CopyWorldResponse>, McpError> {
        world::copy_world(&self.world_registry, request)
    }

    #[tool(description = "List all existing simulation worlds")]
    fn list_worlds(&self) -> Result<rmcp::Json<world::ListWorldsResponse>, McpError> {
        world::list_worlds(&self.world_registry)
    }

    #[tool(
        description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages."
    )]
    fn list_entities(
        &self,
        Parameters(request): Parameters<world::ListEntitiesRequest>,
    ) -> Result<rmcp::Json<world::ListEntitiesResponse>, McpError> {
        world::list_entities(&self.world_registry, Parameters(request))
    }

    #[tool(
        description = "Advance the simulation by running multiple time steps. Each step processes pending messages and executes entity update() functions. Use step_duration to control simulation time granularity."
    )]
    fn advance_simulation(
        &self,
        Parameters(request): Parameters<world::RunSimulationRequest>,
    ) -> Result<rmcp::Json<world::AdvanceSimulationResponse>, McpError> {
        world::advance_simulation(&self.world_registry, Parameters(request))
    }

    #[tool(description = "List the names of all available metrics in the simulation world.")]
    pub fn list_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::ListMetricsRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::metrics::ListMetricsResponse>, McpError> {
        crate::mcp::tools::metrics::list_metrics(&self.world_registry, request)
    }

    #[tool(description = "Get the current values of a specific metric by name.")]
    pub fn get_metric(
        &self,
        Parameters((world_name, metric_name)): Parameters<(String, String)>,
    ) -> Result<rmcp::Json<crate::core::metrics::MetricStats>, McpError> {
        crate::mcp::tools::metrics::get_metric(&self.world_registry, world_name, metric_name)
    }

    #[tool(description = "Get the current values of multiple metrics by their names.")]
    pub fn get_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::GetMetricsRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::metrics::GetMetricsResponse>, McpError> {
        crate::mcp::tools::metrics::get_metrics(&self.world_registry, request)
    }

    #[tool(
        description = "Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script."
    )]
    pub fn set_entity_state(
        &self,
        Parameters(request): Parameters<world::SetEntityStateRequest>,
    ) -> Result<rmcp::Json<world::SetEntityStateResponse>, McpError> {
        world::set_entity_state(&self.world_registry, request)
    }

    #[tool(description = "Get the current state of a specific entity by its ID.")]
    pub fn get_entity_state(
        &self,
        Parameters((world_name, entity_id)): Parameters<(String, String)>,
    ) -> Result<rmcp::Json<world::GetEntityStateResponse>, McpError> {
        world::get_entity_state(&self.world_registry, world_name, entity_id)
    }

    #[tool(
        description = "Get the overall state of the simulation world, including simulation time, entity count, and pending message count."
    )]
    pub fn get_world_state(
        &self,
        Parameters(request): Parameters<world::GetWorldStateRequest>,
    ) -> Result<rmcp::Json<world::GetWorldStateResponse>, McpError> {
        world::get_world_state(&self.world_registry, request)
    }

    #[tool(
        description = "Create a snapshot of the current state of the simulation world, including entity states and pending messages."
    )]
    pub fn create_world_snapshot(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::snapshots::CreateSnapshotRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::snapshots::CreateSnapshotResponse>, McpError> {
        crate::mcp::tools::snapshots::create_snapshot(&self.world_registry, request)
    }

    #[tool(description = "Restore a simulation world to a previously created snapshot state.")]
    pub fn restore_world_snapshot(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::snapshots::RestoreSnapshotRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::snapshots::RestoreSnapshotResponse>, McpError> {
        crate::mcp::tools::snapshots::restore_snapshot(&self.world_registry, request)
    }

    #[tool(description = "Save a simulation world snapshot to a YAML file.")]
    pub fn save_world_snapshot_to_file(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::snapshots::SaveSnapshotToFileRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::snapshots::SaveSnapshotToFileResponse>, McpError> {
        crate::mcp::tools::snapshots::save_snapshot_to_file(&self.world_registry, request)
    }
}

#[tool_handler]
impl ServerHandler for VivariumToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(SERVER_INSTRUCTIONS.into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

unsafe impl Send for VivariumToolServer {}
unsafe impl Sync for VivariumToolServer {}
