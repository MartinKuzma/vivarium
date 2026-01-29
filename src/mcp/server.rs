use crate::mcp::tools::world;
use rmcp::model::CallToolResult;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

const SERVER_INSTRUCTIONS: &str = include_str!("../../docs/mcp/instructions.md");

pub struct VivariumToolServer {
    tool_router: ToolRouter<Self>,
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
    ) -> String {
        match self.world_registry.create(config) {
            Ok(_) => "World created successfully".to_string(),
            Err(e) => format!("Failed to create world: {}", e),
        }
    }

    #[tool(description = "Delete an existing simulation world by name")]
    fn delete_world(&self, Parameters(name): Parameters<String>) -> String {
        match self.world_registry.delete(&name) {
            Ok(_) => format!("World '{}' deleted successfully", name),
            Err(e) => format!("Failed to delete world '{}': {}", name, e),
        }
    }

    #[tool(
        description = "Copy an existing simulation world to a new world with the specified name"
    )]
    fn copy_world(
        &self,
        Parameters(request): Parameters<world::CopyWorldRequest>,
    ) -> Result<CallToolResult, McpError> {
        world::copy_world(&self.world_registry, request)
    }

    #[tool(description = "List all existing simulation worlds")]
    fn list_worlds(&self) -> Result<CallToolResult, McpError> {
        world::list_worlds(&self.world_registry)
    }

    #[tool(
        description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages."
    )]
    fn list_entities(
        &self,
        Parameters(request): Parameters<world::ListEntitiesRequest>,
    ) -> Result<CallToolResult, McpError> {
        world::list_entities(&self.world_registry, Parameters(request))
    }

    #[tool(
        description = "Advance the simulation by running multiple time steps. Each step processes pending messages and executes entity update() functions. Use step_duration to control simulation time granularity."
    )]
    fn advance_simulation(
        &self,
        Parameters(request): Parameters<world::RunSimulationRequest>,
    ) -> Result<CallToolResult, McpError> {
        world::advance_simulation(&self.world_registry, Parameters(request))
    }

    #[tool(description = "List the names of all available metrics in the simulation world.")]
    pub fn list_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::ListMetricsRequest>,
    ) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::metrics::list_metrics(&self.world_registry, request)
    }

    #[tool(description = "Get the current values of a specific metric by name.")]
    pub fn get_metric(
        &self,
        Parameters((world_name, metric_name)): Parameters<(String, String)>,
    ) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::metrics::get_metric(&self.world_registry, world_name, metric_name)
    }

    #[tool(description = "Get the current values of multiple metrics by their names.")]
    pub fn get_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::GetMetricsRequest>,
    ) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::metrics::get_metrics(&self.world_registry, request)
    }

    #[tool(
        description = "Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script."
    )]
    pub fn set_entity_state(
        &self,
        Parameters(request): Parameters<world::SetEntityStateRequest>,
    ) -> Result<(), McpError> {
        world::set_entity_state(&self.world_registry, request)
    }

    #[tool(description = "Get the current state of a specific entity by its ID.")]
    pub fn get_entity_state(
        &self,
        Parameters((world_name, entity_id)): Parameters<(String, String)>,
    ) -> Result<CallToolResult, McpError> {
        world::get_entity_state(&self.world_registry, world_name, entity_id)
    }

    #[tool(
        description = "Get the overall state of the simulation world, including simulation time, entity count, and pending message count."
    )]
    pub fn get_world_state(
        &self,
        Parameters(request): Parameters<world::GetWorldStateRequest>,
    ) -> Result<CallToolResult, McpError> {
        world::get_world_state(&self.world_registry, request)
    }

    #[tool(
        description = "Create a snapshot of the current state of the simulation world, including entity states and pending messages."
    )]
    pub fn create_world_snapshot(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::snapshots::CreateSnapshotRequest>,
    ) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::snapshots::create_snapshot(&self.world_registry, request)
    }

    #[tool(description = "Restore a simulation world to a previously created snapshot state.")]
    pub fn restore_world_snapshot(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::snapshots::RestoreSnapshotRequest>,
    ) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::snapshots::restore_snapshot(&self.world_registry, request)
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
