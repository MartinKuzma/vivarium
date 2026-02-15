# Vivarium
Agent based simulator with scriptable behaviours. Designed for LLM agents to reason and simulate in a structured environment.

# Why Vivarium?
Vivarium means "place of life", which kind of fits the idea of simulated worlds.

My motivation is to use LLMs to simulate complex systems of agents and build various simulations that can help understand emergent behaviour. With their reasoning capabilities, LLMs can create numerous simulations and explore different scenarios.

# Features
- Multiple simulation worlds with named entities
- Behaviour of entities can be scripted using Lua
- Simulation world with entities that can send messages to each other over time
- Time-delayed message delivery system
- Metrics collection and querying
- Snapshot and restore simulation state
- MCP server exposing tools to interact with the simulation

# Scripts
Scripts are used to define the behaviour of entities in the simulation. Currently, only Lua is supported.
## Lua
Each entity script must define the following functions:
- `update`: called each simulation step to update the entity's state and process incoming messages
- `get_state`: returns the current state of the entity as a Lua table
- `set_state`: sets the entity's state from a Lua table

Example Lua script:
```lua
health = 100
x = 0
y = 0

function update(current_time, msgs)
    -- Process incoming messages
    for _, msg in ipairs(msgs) do
        -- Handle message.kind and message.content
    end
    
    -- Send messages to other entities
    self.send_msg(target_id, msg_type, {field = "value"})
end

function get_state()
    -- Return entity state as Lua table
    return {health = health, position = {x = x, y = y}}
end

function set_state(state)
    -- Restore entity state from Lua table
    health = state.health
    x = state.position.x
    y = state.position.y
end
```

## Lua API
Scripts have access to the following APIs for interacting with the simulation:

### self - Entity API
| Function | Description |
|----------|-------------|
| self.id | The unique ID of the current entity |
| self.send_msg(receiver_id, kind, content, delay) | Send a message to another entity with an optional delay (in simulation steps) |
| self.destroy(entity_id) | Destroy an entity by its ID |

### world - World API
| Function | Description |
|----------|-------------|
| world.list_entities() | Returns a table of all entity IDs in the simulation |
| world.record_metric(name, value) | Record a custom metric value for analysis |

# MCP Tools
The MCP server exposes various tools to interact with the simulation worlds and entities.
| Name | Description |
|------|-------------|
| create_world | Create a new simulation world with the specified configuration |
| delete_world | Delete an existing simulation world by name |
| copy_world | Copy an existing simulation world to a new world with the specified name |
| list_worlds | List all existing simulation worlds |
| list_entities | List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages. |
| advance_simulation | Advance the simulation by running multiple time steps. Each step processes pending messages and executes entity update() functions. |
| get_world_state | Get the overall state of the simulation world, including simulation time, entity count, and pending message count. |
| set_entity_state | Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script. |
| get_entity_state | Get the current state of a specific entity by its ID. |
| list_metrics | List the names of all available metrics in the simulation world. |
| get_metric | Get the current values of a specific metric by name. |
| get_metrics | Get the current values of multiple metrics by their names. |
| create_world_snapshot | Create a snapshot of the current state of the simulation world, including entity states and pending messages. |
| restore_world_snapshot | Restore a simulation world to a previously created snapshot state. |
| save_world_snapshot_to_file | Save a simulation world snapshot to a YAML file. |
| load_world_snapshot_from_file | Load a simulation world snapshot from a YAML file. |
