# Vivarium Agent Simulation Server
This MCP server provides tools to create and run agent-based simulations using Lua scripts.

## Usage Flow
1. **Create entities** using `create_entity` - each entity needs a unique ID and Lua script
2. **Run simulation** using `run_simulation_steps` to advance the simulation
3. **Check state** using `list_entities` to see all entities in the simulation

## Lua Script Requirements
Each entity script MUST define TWO functions:

1. **`update(msgs)`** - Processes messages and executes entity logic:
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

2. **`get_state()`** - Returns a table with the entity's current state (for inspection/debugging):
```lua
function get_state()
    return {
        health = health,
        position = position,
        active = active
    }
end
```

Each entity can maintain its own state using a global `state` table. Only the `state` table will be exposed to the server for inspection. Any relevant state variables should be stored here:
```lua
state = {
    counter = 0,
    position = {x = 0, y = 0}
}
```

## Available Lua API:
- `self.id` - entity's unique identifier
- `self.send_msg(target_id, msg_type, content, delay)` 
  - sends message to another entity
  - target_id is the recipient entity ID
  - content can be a string OR a Lua table
  - delay is in seconds
- `world.list_entities()` - get list of all entity IDs

## Message Content Types:
1. **String messages**: `self.send_msg("agent2", "Greeting", "Hello", 0)`
2. **Structured messages**: `self.send_msg("agent2", "Status", {health=100, x=10, y=20}, 0)`
   - Pass Lua tables directly - no JSON serialization overhead!
   - Nested tables fully supported
   - Supports: strings, numbers, booleans, and nested tables

## Example Scenario:
Create two communicating agents that exchange structured data, then run the simulation for several steps to see them interact.

Agent A (sends position and status):
```lua
pos_x = 10
pos_y = 20
velocity = 2.5

function update(msgs)
    for _, msg in ipairs(msgs) do
        if msg.kind == "status_report" then
            print("Agent A received status")
            print("  Health: " .. msg.content.health)
            print("  Position: (" .. msg.content.pos.x .. ", " .. msg.content.pos.y .. ")")
        end
    end
    
    -- Send structured data
    self.send_msg("agent_b", "position_update", {
        pos = {x = pos_x, y = pos_y},
        velocity = velocity,
        heading = 90
    }, 1)
end

function get_state()
    return {
        pos_x = pos_x,
        pos_y = pos_y,
        velocity = velocity
    }
end
```

Agent B (receives and responds with status):
```lua
health = 100
pos_x = 15
pos_y = 25

function update(msgs)
    for _, msg in ipairs(msgs) do
        if msg.kind == "position_update" then
            print("Agent B received position")
            print("  Position: (" .. msg.content.pos.x .. ", " .. msg.content.pos.y .. ")")
            
            -- Respond with structured status
            self.send_msg("agent_a", "status_report", {
                health = health,
                pos = {x = pos_x, y = pos_y},
                items = {"sword", "shield", "potion"}
            }, 1)
        end
    end
end

function get_state()
    return {
        health = health,
        pos_x = pos_x,
        pos_y = pos_y
    }
end
```

Run the simulation for 10 steps to observe the structured message exchange.