use std::sync::Arc;
use std::sync::Mutex;

use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{
    ErrorData as McpError, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

const SERVER_INSTRUCTIONS: &str = include_str!("../../docs/mcp/instructions.md");

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

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Entity {
    #[schemars(description = "The unique ID of the entity")]
    pub id: String,
    #[schemars(description = "The serialized state of the entity's Lua script")]
    pub state: String,
}

#[derive(Debug, serde::Serialize,schemars::JsonSchema)]
pub struct ListEntitiesResponse {
    #[schemars(description = "List of entity IDs in the simulation")]
    pub entities: Vec<Entity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AdvanceSimulationResponse {
    #[schemars(description = "List of delivered messages during the simulation steps")]
    pub delivered_messages: Vec<String>,
}

#[tool_router]
impl SimulationToolServer {
    pub fn new() -> Self {
        let tool_router = Self::tool_router();
        SimulationToolServer {
            tool_router,
            world: Arc::new(Mutex::new(crate::simulator::World::new())),
        }
    }

    //TODO: Reset simulation state

    #[tool(description = "Create a new entity with a Lua script. The script must define an 'update(msgs)' function. Each entity needs a unique ID.")]
    fn create_entity(
        &self,
        Parameters(CreateEntityRequest { id, lua_script }): Parameters<CreateEntityRequest>,
    ) -> String {
        let mut world = self.world.lock().unwrap();
        match world.create_entity(id.clone(), lua_script.clone()) {
            Ok(_) => return format!("Entity '{}' created with Lua script: {}", id, lua_script),
            Err(e) => return format!("Failed to create entity '{}': {}", id, e),
        }
    }

    #[tool(description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages.")]
    fn list_entities(&self) -> Result<CallToolResult, McpError> {
        let mut resp = ListEntitiesResponse {
            entities: Vec::new(),
        };

        let world = self.world.lock().unwrap();

        for (id, entity) in world.get_state_ref().get_entities() {
            match entity.borrow().get_lua_controller().get_serialized_state() {
                Ok(state_str) => {
                    resp.entities.push(Entity {
                        id: id.clone(),
                        state: state_str.clone(),
                    });
                }
                Err(e) => {
                    return Err(McpError::new(
                        rmcp::model::ErrorCode::INTERNAL_ERROR,
                        format!("Failed to serialize state for entity '{}': {}", id, e),
                        None,
                    ));
                }
            }
        }

        Ok(CallToolResult::success(vec![Content::json(&resp).unwrap()]))
    }

    #[tool(description = "Advance the simulation by running multiple time steps. Each step processes pending messages and executes entity update() functions. Use step_duration to control simulation time granularity.")]
    fn run_simulation_steps(
        &self,
        Parameters(AdvanceSimulationRequest {
            step_duration,
            num_steps,
        }): Parameters<AdvanceSimulationRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut delivered_messages: Vec<String> = Vec::new();

        let mut world = self.world.lock().unwrap();
        for _ in 0..num_steps {
            match world.update(step_duration) {
                Ok(result) => {
                    for msg in result.delivered_messages {
                        delivered_messages.push(format!("{:?}", msg));
                    }
                }
                Err(e) => {
                    return Err(McpError::new(
                        rmcp::model::ErrorCode::INTERNAL_ERROR,
                        format!("Error during simulation step: {}", e),
                        None,
                    ));
                }
            }
        }

        Ok(CallToolResult::success(vec![
            Content::json(&AdvanceSimulationResponse {
                delivered_messages,
            })
            .unwrap(),
        ]))
    }

    #[tool(description = "Retrieve statistics for a specific recorded metric.")]
    fn get_metric_stats(&self,
        Parameters(name): Parameters<String>,
    ) -> Result<CallToolResult, McpError> {
        let world = self.world.lock().unwrap();
        let metrics = world.get_metrics_ref();

        match metrics.get_metric_stats(name.as_str()) {
            Some(stats) => {
                return Ok(CallToolResult::success(vec![
                    Content::json(&stats)
                    .unwrap(),
                ]));
            },
            None => {
                return Err(McpError::new(
                    rmcp::model::ErrorCode::INVALID_PARAMS,
                    format!("Error retrieving metric '{}'", name),
                    None,
                ));
            }
        }
    }

    #[tool(description = "Retrieve all recorded metrics and their values over time.")]
    pub fn get_all_metrics(&self)  -> Result<CallToolResult, McpError> {
        let world = self.world.lock().unwrap();
        let metrics = world.get_metrics_ref();
        let all_metrics = metrics.get_all_metrics();

        Ok(CallToolResult::success(vec![
            Content::json(&all_metrics)
            .unwrap(),
        ]))
    }

    #[tool(description = "Reset the simulation to its initial state, removing all entities and clearing all metrics.")]
    pub fn reset_simulation(&self) {
        let mut world = self.world.lock().unwrap();
        *world = crate::simulator::World::new();
    }
}

unsafe impl Send for SimulationToolServer {}
unsafe impl Sync for SimulationToolServer {}

#[tool_handler]
impl ServerHandler for SimulationToolServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(SERVER_INSTRUCTIONS.into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
