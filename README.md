# vivarium
Agent based simulator with scriptable behaviours. Designed for LLM agents to reason and simulate in a structured environment.

# Why Vivarium?
Vivarium means "a place where life dwells", which kind of fits the idea of simulated worlds.

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
# MCP Tools
The MCP server exposes various tools to interact with the simulation worlds and entities.
| Name | Description |
|------|-------------|
