use crate::core::messaging::JSONObject;
use rmcp::model::CallToolResult;
use rmcp::model::Content;
use rmcp::{ErrorData as McpError, handler::server::wrapper::Parameters, schemars};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CopyWorldRequest {
    #[schemars(description = "The name of the source simulation world to copy from")]
    pub source_world_name: String,
    #[schemars(description = "The name of the target simulation world to copy to")]
    pub target_world_name: String,
    #[schemars(description = "Whether to replace the target world if it already exists")]
    pub replace_if_exists: bool,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RunSimulationRequest {
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
    #[schemars(description = "The name of the simulation world containing the entity")]
    pub world_name: String,
    #[schemars(description = "The unique ID of the entity")]
    pub entity_id: String,
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetWorldStateRequest {
    pub world_name: String,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct GetWorldStateResponse {
    pub simulation_time: u64,
    pub entities_count: usize,
    pub pending_messages_count: usize,
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
pub struct ListEntitiesResponse {
    #[schemars(description = "List of entity IDs in the simulation")]
    pub entities: Vec<Entity>,
}

pub fn list_entities(
    registry: &crate::core::registry::Registry,
    Parameters(request): Parameters<ListEntitiesRequest>,
) -> Result<CallToolResult, McpError> {
    let mut resp = ListEntitiesResponse {
        entities: Vec::new(),
    };

    let world = registry.get(&request.world_name).ok_or_else(|| {
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

pub fn advance_simulation(
    registry: &crate::core::registry::Registry,
    Parameters(request): Parameters<RunSimulationRequest>,
) -> Result<CallToolResult, McpError> {
    let mut delivered_messages: Vec<String> = Vec::new();
    let mut number_of_messages = 0;

    let world = registry.get(&request.world_name).ok_or_else(|| {
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
        Content::json(&AdvanceSimulationResponse {
            delivered_messages,
            number_of_messages,
        })
        .unwrap(),
    ]))
}

pub fn list_worlds(registry: &crate::core::registry::Registry) -> Result<CallToolResult, McpError> {
    let worlds = registry.list();
    Ok(CallToolResult::success(vec![
        Content::json(&worlds).unwrap(),
    ]))
}

pub fn set_entity_state(
    registry: &crate::core::registry::Registry,
    request: SetEntityStateRequest,
) -> Result<(), McpError> {
    let world = registry.get(&request.world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", request.world_name),
            None,
        )
    })?;

    world
        .write()
        .unwrap()
        .set_entity_state(&request.entity_id, request.state)
        .map_err(|e| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!(
                    "Failed to set state for entity '{}': {}",
                    request.entity_id, e
                ),
                None,
            )
        })?;

    Ok(())
}

pub fn get_entity_state(
    registry: &crate::core::registry::Registry,
    world_name: String,
    entity_id: String,
) -> Result<CallToolResult, McpError> {
    let world = registry.get(&world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", world_name),
            None,
        )
    })?;

    let state = world
        .read()
        .unwrap()
        .get_entity_state(&entity_id)
        .ok_or_else(|| {
            McpError::new(
                rmcp::model::ErrorCode::INVALID_PARAMS,
                format!("Entity '{}' not found in world '{}'", entity_id, world_name),
                None,
            )
        })?;

    Ok(CallToolResult::success(vec![
        Content::json(&state).unwrap(),
    ]))
}

pub fn copy_world(
    registry: &crate::core::registry::Registry,
    request: CopyWorldRequest,
) -> Result<CallToolResult, McpError> {
    match registry.copy(
        &request.source_world_name,
        &request.target_world_name,
        request.replace_if_exists,
    ) {
        Ok(_) => Ok(CallToolResult::success(vec![
            Content::json(&format!(
                "World '{}' copied to '{}' successfully",
                request.source_world_name, request.target_world_name
            ))
            .unwrap(),
        ])),
        Err(e) => Err(McpError::new(
            rmcp::model::ErrorCode::INTERNAL_ERROR,
            format!(
                "Failed to copy world '{}' to '{}': {}",
                request.source_world_name, request.target_world_name, e
            ),
            None,
        )),
    }
}

pub fn get_world_state(
    registry: &crate::core::registry::Registry,
    request: GetWorldStateRequest,
) -> Result<CallToolResult, McpError> {
    let world_rc = registry.get(&request.world_name).ok_or_else(|| {
        McpError::new(
            rmcp::model::ErrorCode::INVALID_PARAMS,
            format!("World '{}' not found", request.world_name),
            None,
        )
    })?;

    let world = world_rc.read().unwrap();
    
    let response = GetWorldStateResponse {
        simulation_time: world.get_simulation_time(),
        entities_count: world.get_entities_count(),
        pending_messages_count: world.get_pending_messages_count(),
    };

    Ok(CallToolResult::success(vec![
        Content::json(&response).unwrap(),
    ]))
}