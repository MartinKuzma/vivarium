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

pub struct VivariumToolServer {
    tool_router: ToolRouter<Self>,
    world_registry: Arc<crate::core::registry::Registry>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AdvanceSimulationRequest {
    #[schemars(description = "The name of the simulation world to advance")]
    pub world_name: String,
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListEntitiesRequest {
    #[schemars(description = "The name of the simulation world to query")]
    pub world_name: String,
    #[schemars(description = "Whether to include the states of the entities in the response")]
    pub include_states: bool,
}


#[tool_router]
impl VivariumToolServer {
    pub fn new(world_registry: Arc<crate::core::registry::Registry>) -> Self {
        let tool_router = Self::tool_router();

        VivariumToolServer {
            tool_router,
            world_registry,
        }
    }

    #[tool(description = "Create a new simulation world with the specified configuration")]
    fn create_world(&self, Parameters(config): Parameters<crate::core::schema::WorldCfg>) -> String {
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

    #[tool(description = "List all existing simulation worlds")]
    fn list_worlds(&self) -> Result<CallToolResult, McpError> {
        let worlds = self.world_registry.list();
        Ok(CallToolResult::success(vec![
            Content::json(&worlds).unwrap(),
        ]))
    }

    #[tool(description = "List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages.")]
    fn list_entities(&self, Parameters(request): Parameters<ListEntitiesRequest>) -> Result<CallToolResult, McpError> {
        let mut resp = ListEntitiesResponse {
            entities: Vec::new(),
        };

        let world = self.world_registry.get(&request.world_name).ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("World '{}' not found", request.world_name),
                None,
            )
        })?;

        let world = world.read().unwrap();

        if !request.include_states {
            for (id, _entity) in world.get_state_ref().get_entities() {
                resp.entities.push(Entity {
                    id: id.clone(),
                    state: JSONObject::new(),
                });
            }

            return Ok(CallToolResult::success(vec![Content::json(&resp).unwrap()]));
        }

        for (id, entity) in world.get_state_ref().get_entities() {
            match entity.borrow().get_lua_controller().get_state() {
                Ok(state) => {
                    resp.entities.push(Entity {
                        id: id.clone(),
                        state: state.clone(),
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
        Parameters(request): Parameters<AdvanceSimulationRequest>,
    ) -> Result<CallToolResult, McpError> {
        let mut delivered_messages: Vec<String> = Vec::new();
        let mut number_of_messages = 0;

        let world = self.world_registry.get(&request.world_name).ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("World '{}' not found", request.world_name),
                None,
            )
        })?;

        for _ in 0..request.num_steps {
            match world.write().unwrap().update(request.step_duration) {
                Ok(result) => {
                    number_of_messages += result.delivered_messages.len();

                    if request.include_delivered_messages {
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


    #[tool(description = "Retrieve all recorded metrics and their values over time.")]
    pub fn get_all_metrics(&self, Parameters(world_name): Parameters<String>) -> Result<CallToolResult, McpError> {
        crate::mcp::tools::metrics::get_all_metrics(&self.world_registry, world_name)
    }

    // ) -> Result<CallToolResult, McpError> {
    //     let world = self.world.lock().unwrap();
    //     let metrics = world.get_metrics_ref();

    //     match metrics.get_metric_stats(name.as_str()) {
    //         Some(stats) => {
    //             return Ok(CallToolResult::success(vec![
    //                 Content::json(&stats).unwrap(),
    //             ]));
    //         }
    //         None => {
    //             return Err(McpError::new(
    //                 rmcp::model::ErrorCode::INVALID_PARAMS,
    //                 format!("Error retrieving metric '{}'", name),
    //                 None,
    //             ));
    //         }
    //     }
    // }

    // #[tool(description = "Retrieve all recorded metrics and their values over time.")]
    // pub fn get_all_metrics(&self,   Parameters(world_name): Parameters<String>) -> Result<CallToolResult, McpError> {
    //     let world = self.world_registry.get(&world_name).ok_or_else(|| {
    //         McpError::new(
    //             rmcp::model::ErrorCode::INVALID_PARAMS,
    //             format!("World '{}' not found", world_name),
    //             None,
    //         )
    //     })?;

    //     let world = world.read().unwrap();
    //     let metrics = world.get_metrics_ref();
    //     let all_metrics = metrics.get_all_metrics();

    //     Ok(CallToolResult::success(vec![
    //         Content::json(&all_metrics).unwrap(),
    //     ]))
    // }

    // #[tool(description = "Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script.")]
    // pub fn set_entity_state(
    //     &self,
    //     Parameters(SetEntityStateRequest { id, state }): Parameters<SetEntityStateRequest>,
    // ) -> Result<(), McpError> {
    //     let mut world = self.world.lock().unwrap();

    //     world.set_entity_state(&id, state).map_err(|e| {
    //         McpError::new(
    //             rmcp::model::ErrorCode::INVALID_PARAMS,
    //             format!("Failed to set state for entity '{}': {}", id, e),
    //             None,
    //         )
    //     })?;

    //     Ok(())
    // }
}

unsafe impl Send for VivariumToolServer {}
unsafe impl Sync for VivariumToolServer {}

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
