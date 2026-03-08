use crate::{core::persistence, mcp::tools::{snapshots, world}, mcp::project_store::ProjectStore};
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};

const SERVER_INSTRUCTIONS: &str = include_str!("../../docs/mcp/instructions.md");

pub struct VivariumToolServer {
    pub tool_router: ToolRouter<Self>,
    store: crate::mcp::project_store::ProjectStore,
}

#[tool_router]
impl VivariumToolServer {
    pub fn new(store: ProjectStore) -> Self {
        let tool_router = Self::tool_router();

        VivariumToolServer {
            tool_router,
            store,
        }
    }

    #[tool(description = "Load a project from its manifest and snapshot into the in-memory runtime")]
    fn load_project(
        &self,
        Parameters(request): Parameters<world::LoadProjectRequest>,
    ) -> Result<rmcp::Json<world::LoadProjectResponse>, McpError> {
        let project_ctx = persistence::loader::load_project_from_file(&request.manifest_file_path)?;
        let snapshot_selection = request
            .snapshot
            .parse::<persistence::loader::SnapshotSelection>()
            .map_err(|e| {
                McpError::new(
                    rmcp::model::ErrorCode::INVALID_PARAMS,
                    format!("Invalid snapshot selection '{}': {}", request.snapshot, e),
                    None,
                )
            })?;
        let snapshot = persistence::loader::load_snapshot(
            &project_ctx,
            snapshot_selection,
        )?;

        let world_data = crate::core::WorldSnapshotData {
            name: project_ctx.manifest.name.clone(),
            script_library: project_ctx.script_library.clone(),
            entities: snapshot.entities,
            pending_messages: snapshot.pending_messages,
            metrics: if request.reset_metrics { None } else { snapshot.metrics },
            simulation_time: snapshot.simulation_time,
        };

        let world = crate::core::World::new(world_data)?;
        self.store.add_project(
            project_ctx.manifest.name.clone(),
            world,
            project_ctx.clone(),
            request.replace_if_loaded,
        )?;

        Ok(rmcp::Json(world::LoadProjectResponse {
            message: format!(
                "Project '{}' loaded successfully from '{}' (snapshot='{}')",
                project_ctx.manifest.name, request.manifest_file_path, request.snapshot
            ),
        }))
    }

    #[tool(description = "Initialize a new Vivarium project folder with starter scripts and an initial snapshot")]
    fn initialize_project(
        &self,
        Parameters(request): Parameters<world::InitializeProjectRequest>,
    ) -> Result<rmcp::Json<world::InitializeProjectResponse>, McpError> {
        crate::core::persistence::init_project::init_project(std::path::Path::new(&request.target_dir)).map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to initialize project at '{}': {}", request.target_dir, e),
                None,
            )
        })?;

        Ok(rmcp::Json(world::InitializeProjectResponse {
            message: format!("Project initialized successfully at '{}'", request.target_dir),
        }))
    }

    #[tool(description = "Unload a loaded project from memory by name")]
    fn unload_project(&self, Parameters(name): Parameters<String>) -> Result<rmcp::Json<world::UnloadProjectResponse>, McpError> {
        self.store.delete(&name).map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INTERNAL_ERROR,
                format!("Failed to unload project '{}': {}", name, e),
                None,
            )
        })?;

        Ok(rmcp::Json(world::UnloadProjectResponse {
            message: format!("Project '{}' unloaded successfully", name),
        }))
    }

    #[tool(description = "List all loaded projects")]
    fn list_projects(&self) -> Result<rmcp::Json<world::ListProjectsResponse>, McpError> {
        world::list_projects(&self.store)
    }

    #[tool(description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages.")]
    fn list_entities(
        &self,
        Parameters(request): Parameters<world::ListEntitiesRequest>,
    ) -> Result<rmcp::Json<world::ListEntitiesResponse>, McpError> {
        world::list_entities(&self.store, Parameters(request))
    }

    #[tool(description = "Advance the simulation by running multiple time steps. Each step processes pending messages and executes entity update() functions. Use step_duration to control simulation time granularity.")]
    fn advance_simulation(
        &self,
        Parameters(request): Parameters<world::RunSimulationRequest>,
    ) -> Result<rmcp::Json<world::AdvanceSimulationResponse>, McpError> {
        world::advance_simulation(&self.store, Parameters(request))
    }

    #[tool(description = "List the names of all available metrics in the loaded project.")]
    pub fn list_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::ListMetricsRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::metrics::ListMetricsResponse>, McpError> {
        crate::mcp::tools::metrics::list_metrics(&self.store, request)
    }

    #[tool(description = "Get the current values of a specific metric by name.")]
    pub fn get_metric(
        &self,
        Parameters((project_name, metric_name)): Parameters<(String, String)>,
    ) -> Result<rmcp::Json<crate::core::metrics::MetricStats>, McpError> {
        crate::mcp::tools::metrics::get_metric(&self.store, project_name, metric_name)
    }

    #[tool(description = "Get the current values of multiple metrics by their names.")]
    pub fn get_metrics(
        &self,
        Parameters(request): Parameters<crate::mcp::tools::metrics::GetMetricsRequest>,
    ) -> Result<rmcp::Json<crate::mcp::tools::metrics::GetMetricsResponse>, McpError> {
        crate::mcp::tools::metrics::get_metrics(&self.store, request)
    }

    #[tool(
        description = "Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script."
    )]
    pub fn set_entity_state(
        &self,
        Parameters(request): Parameters<world::SetEntityStateRequest>,
    ) -> Result<rmcp::Json<world::SetEntityStateResponse>, McpError> {
        world::set_entity_state(&self.store, request)
    }

    #[tool(description = "Get the current state of a specific entity by its ID.")]
    pub fn get_entity_state(
        &self,
        Parameters((project_name, entity_id)): Parameters<(String, String)>,
    ) -> Result<rmcp::Json<world::GetEntityStateResponse>, McpError> {
        world::get_entity_state(&self.store, project_name, entity_id)
    }

    #[tool(
        description = "Get the overall state of the loaded project, including simulation time, entity count, and pending message count."
    )]
    pub fn get_project_state(
        &self,
        Parameters(request): Parameters<world::GetProjectStateRequest>,
    ) -> Result<rmcp::Json<world::GetProjectStateResponse>, McpError> {
        world::get_project_state(&self.store, request)
    }

    #[tool(description = "List available snapshot names for a loaded project")]
    pub fn list_project_snapshots(
        &self,
        Parameters(request): Parameters<snapshots::ListProjectSnapshotsRequest>,
    ) -> Result<rmcp::Json<snapshots::ListProjectSnapshotsResponse>, McpError> {
        snapshots::list_project_snapshots(&self.store, request)
    }

    #[tool(description = "Save a snapshot for a loaded project")]
    pub fn save_project_snapshot(
        &self,
        Parameters(request): Parameters<snapshots::SaveProjectSnapshotRequest>,
    ) -> Result<rmcp::Json<snapshots::SaveProjectSnapshotResponse>, McpError> {
        snapshots::save_project_snapshot(&self.store, request)
    }

    #[tool(description = "Load a snapshot into a loaded project")]
    pub fn load_project_snapshot(
        &self,
        Parameters(request): Parameters<snapshots::LoadProjectSnapshotRequest>,
    ) -> Result<rmcp::Json<snapshots::LoadProjectSnapshotResponse>, McpError> {
        snapshots::load_project_snapshot(&self.store, request)
    }


    // #[tool(description = "Load a simulation world snapshot from a YAML file.")]
    // pub fn load_world_snapshot_from_file(
    //     &self,
    //     Parameters(request): Parameters<crate::mcp::tools::snapshots::LoadSnapshotFromFileRequest>,
    // ) -> Result<rmcp::Json<crate::mcp::tools::snapshots::LoadSnapshotFromFileResponse>, McpError> {
    //     crate::mcp::tools::snapshots::load_snapshot_from_file(&self.store, request)
    // }
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
