# Vivarium Agent Simulation Server
This MCP server provides tools to create and run agent-based simulations using Lua scripts.

## Usage Flow:
1. **Create entities** using `create_entity` - each entity needs a unique ID and Lua script
2. **Run simulation** using `run_simulation_steps` to advance the simulation
3. **Check state** using `list_entities` to see all entities in the simulation

## Lua Script Requirements:
Each entity script MUST define an `update` function that receives messages:
```lua
function update(msgs)
    -- Process incoming messages
    for _, msg in ipairs(msgs) do
        print("Received: " .. msg.content)
    end
    
    -- Entity logic here
    -- Use self.send_msg(target_id, msg_type, content, delay) to communicate
end
```

## Available Lua API:
- `self.id` - entity's unique identifier
- `self.send_msg(target_id, msg_type, content, delay)` - send message to another entity
- `world.list_entities()` - get list of all entity IDs

## Example Scenario:
Create two communicating agents, then run the simulation for several steps to see them interact.
Agent A:
```lua
function update(msgs)
    for _, msg in ipairs(msgs) do
        print("Agent A received: " .. msg.content)
    end
    self.send_msg("agent_b", "greeting", "Hello from Agent A!", 1)
end
```
Agent B:
```lua
function update(msgs)
    for _, msg in ipairs(msgs) do
        print("Agent B received: " .. msg.content)
    end
    self.send_msg("agent_a", "response", "Hello from Agent B!", 1)
end
```
Run the simulation for 10 steps to observe the message exchange.