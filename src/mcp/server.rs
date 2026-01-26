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
use crate::core::messaging::JSONObject;

const SERVER_INSTRUCTIONS: &str = include_str!("../../docs/mcp/instructions.md");

pub struct SimulationToolServer {
    tool_router: ToolRouter<Self>,
    world: Arc<Mutex<crate::core::World>>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CreateEntitiesRequest {
    #[schemars(description = "The Lua script controlling the entities. Must define an 'update' function.")]
    pub lua_script: String,
    #[schemars(description = "The number of entities to create")]
    pub count: usize,
    #[schemars(description = "The prefix for the entity names")]
    pub name_prefix: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AdvanceSimulationRequest {
    #[schemars(description = "The duration of each step in seconds")]
    pub step_duration: u64,
    #[schemars(description = "The number of steps to run")]
    pub num_steps: u32,
    #[serde(default)]
    #[schemars(description = "Whether to include delivered messages in the response")]
    pub include_delivered_messages: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetEntityStateRequest {
    #[schemars(description = "The unique ID of the entity")]
    pub id: String,
    #[schemars(description = "The state as a JSON object to set for the entity")]
    pub state: JSONObject,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct Entity {
    #[schemars(description = "The unique ID of the entity")]
    pub id: String,
    #[schemars(description = "The current state of the entity as a JSON object")]
    pub state: JSONObject,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ListEntitiesResponse {
    #[schemars(description = "List of entity IDs in the simulation")]
    pub entities: Vec<Entity>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AdvanceSimulationResponse {
    #[schemars(description = "List of delivered messages during the simulation steps")]
    pub delivered_messages: Vec<String>,
    #[schemars(description = "Total number of delivered messages during the simulation steps")]
    pub number_of_messages: usize,
}

#[tool_router]
impl SimulationToolServer {
    pub fn new() -> Self {
        let tool_router = Self::tool_router();
        SimulationToolServer {
            tool_router,
            world: Arc::new(Mutex::new(crate::core::World::new())),
        }
    }

    #[tool(description = "Create new multiple entities with a Lua script")]
    fn create_entities(
        &self,
        Parameters(CreateEntitiesRequest { lua_script, count, name_prefix }): Parameters<CreateEntitiesRequest>,
    ) -> String {
        let mut world = self.world.lock().unwrap();
        match world.create_entities(&lua_script, count, &name_prefix) {
            Ok(_) => return format!("Created {} entities with Lua script: {}", count, lua_script),
            Err(e) => return format!("Failed to create entities: {}", e),
        }
    }

    #[tool(description = "Remove an entity from the simulation by its ID.")]
    fn remove_entity(&self, Parameters(id): Parameters<String>) -> String {
        let mut world = self.world.lock().unwrap();
        match world.remove_entity(&id) {
            Some(_) => return format!("Entity '{}' removed", id),
            None => return format!("Entity '{}' not found", id),
        }
    }

    #[tool(description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages.")]
    fn list_entities(&self) -> Result<CallToolResult, McpError> {
        let mut resp = ListEntitiesResponse {
            entities: Vec::new(),
        };

        let world = self.world.lock().unwrap();

        for (id, entity) in world.get_state_ref().get_entities() {
            match entity.borrow().get_lua_controller().get_state() {
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
            include_delivered_messages,
        }): Parameters<AdvanceSimulationRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut delivered_messages: Vec<String> = Vec::new();
        let mut number_of_messages = 0;

        let mut world = self.world.lock().unwrap();
        for _ in 0..num_steps {
            match world.update(step_duration) {
                Ok(result) => {
                    number_of_messages += result.delivered_messages.len();

                    if include_delivered_messages {
                        for msg in result.delivered_messages {
                            delivered_messages.push(format!("{:?}", msg));
                        }
                    }   
                }
                Err(e) => {
                    return Err(McpError::new(
                        rmcp::model::ErrorCode::INTERNAL_ERROR,
                        format!("Error during simulation step: {}", e),
                        None,
                    ));
                }
            };
        }

        Ok(CallToolResult::success(vec![
            Content::json(&AdvanceSimulationResponse { delivered_messages, number_of_messages }).unwrap(),
        ]))
    }

    #[tool(description = "Retrieve statistics for a specific recorded metric.")]
    fn get_metric_stats(
        &self,
        Parameters(name): Parameters<String>,
    ) -> Result<CallToolResult, McpError> {
        let world = self.world.lock().unwrap();
        let metrics = world.get_metrics_ref();

        match metrics.get_metric_stats(name.as_str()) {
            Some(stats) => {
                return Ok(CallToolResult::success(vec![
                    Content::json(&stats).unwrap(),
                ]));
            }
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
    pub fn get_all_metrics(&self) -> Result<CallToolResult, McpError> {
        let world = self.world.lock().unwrap();
        let metrics = world.get_metrics_ref();
        let all_metrics = metrics.get_all_metrics();

        Ok(CallToolResult::success(vec![
            Content::json(&all_metrics).unwrap(),
        ]))
    }

    #[tool(
        description = "Reset the simulation to its initial state, removing all entities and clearing all metrics."
    )]
    pub fn reset_simulation(&self) {
        let mut world = self.world.lock().unwrap();
        *world = crate::core::World::new();
    }

    #[tool(description = "Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script.")]
    pub fn set_entity_state(
        &self,
        Parameters(SetEntityStateRequest { id, state }): Parameters<SetEntityStateRequest>,
    ) -> Result<(), McpError> {
        let mut world = self.world.lock().unwrap();

        world.set_entity_state(&id, state).map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Failed to set state for entity '{}': {}", id, e),
                None,
            )
        })?;

        Ok(())
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
