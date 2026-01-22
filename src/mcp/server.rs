use std::sync::Arc;
use std::sync::Mutex;

use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{
    ServerHandler,
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

pub struct SimulationToolServer {
    tool_router: ToolRouter<Self>,
    world: Arc<Mutex<crate::simulator::World>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateEntityRequest {
    #[schemars(description = "The name of the entity to create")]
    pub id: String,
    #[schemars(description = "The Lua script controlling the entity. Must define an 'update' function.")]
    pub lua_script: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AdvanceSimulationRequest {
    #[schemars(description = "The duration of each step in seconds")]
    pub step_duration: u64,
    #[schemars(description = "The number of steps to run")]
    pub num_steps: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ListEntitiesResponse {
    #[schemars(description = "List of entity IDs in the simulation")]
    pub entity_ids: Vec<String>,
}

#[tool_router]
impl SimulationToolServer {
    pub fn new() -> Self {
        let tool_router = ToolRouter::new();
        SimulationToolServer { 
            tool_router, 
            world: Arc::new(Mutex::new(crate::simulator::World::new())),
        }
    }

    //TODO: Reset simulation state

    #[tool(description="Create a new entity with a Lua script")]
    fn create_entity(
        &self,
        Parameters( CreateEntityRequest { id, lua_script} ): Parameters<CreateEntityRequest>,        
    ) -> String {
        let mut world = self.world.lock().unwrap();
        match world.create_entity(id.clone(), lua_script.clone()) {
            Ok(_) => return format!("Entity '{}' created with Lua script: {}", id, lua_script),
            Err(e) => return format!("Failed to create entity '{}': {}", id, e),
        }
    }

    #[tool(description="List all entities in the simulation and their states")]
    fn list_entities(&self) -> Result<CallToolResult, McpError> {
        let world = self.world.lock().unwrap();
        let resp = ListEntitiesResponse {
            entity_ids: world.get_state_ref().get_entities().keys().cloned().collect()
        };

        Ok(CallToolResult::success(vec![Content::json(&resp).unwrap()]))
    }

    #[tool(description="Advance the simulation by a number of steps")]
    fn run_simulation_steps(
        &self,
        Parameters(AdvanceSimulationRequest { step_duration, num_steps }): Parameters<AdvanceSimulationRequest>,
    ) -> String {
        let mut world = self.world.lock().unwrap();
        for _ in 0..num_steps {
            match world.update(std::time::Duration::from_secs(step_duration)) {
                Ok(_) => (),
                Err(e) => return format!("Error during simulation step: {}", e),
            }
        }

        format!("Simulation advanced by {} steps of {} seconds each.", num_steps, step_duration)
    }
}

unsafe impl Send for SimulationToolServer {}
unsafe impl Sync for SimulationToolServer {}

#[tool_handler]
impl ServerHandler for SimulationToolServer {
        fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Agent simulation server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}