# Vivarium
Agent based simulator with scriptable behaviours. Designed for LLM agents to reason and simulate in a structured environment.

# Why Vivarium?
Vivarium means "place of life", which kind of fits the idea of simulated worlds.

My motivation is to use LLMs to simulate complex systems of agents and build various simulations that can help understand emergent behaviour. With their reasoning capabilities, LLMs can create numerous simulations and explore different scenarios.

# Features
- Project-based simulation runtime with named entities
- Behaviour of entities can be scripted using Lua
- Simulation runtime where entities can send messages to each other over time
- Time-delayed message delivery system
- Metrics collection and querying
- Project snapshot save/load and state restoration
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
The MCP server exposes tools to interact with loaded projects and their entities.

Most runtime tools target a loaded project by `project_name`.

| Name | Description |
|------|-------------|
| initialize_project | Create a new project directory with `world.yaml`, starter Lua script, and initial snapshot files. |
| load_project | Load a project manifest and snapshot into memory. |
| unload_project | Unload a loaded project from memory. |
| list_projects | List all currently loaded projects. |
| list_entities | List all entities in a loaded project. Can include current entity state. |
| advance_simulation | Advance a loaded project by running multiple time steps. |
| get_project_state | Get simulation time, entity count, and pending message count for a loaded project. |
| set_entity_state | Set the state of a specific entity in a loaded project. |
| get_entity_state | Get the current state of a specific entity in a loaded project. |
| list_metrics | List the names of all available metrics in a loaded project. |
| get_metric | Get the current values of a specific metric by name. |
| get_metrics | Get the current values of multiple metrics by their names. |
| list_project_snapshots | List snapshots available under a loaded project's `snapshots/` directory. |
| save_project_snapshot | Save the current loaded project state as a named snapshot. |
| load_project_snapshot | Load a named snapshot (or `latest`) into a loaded project. |
