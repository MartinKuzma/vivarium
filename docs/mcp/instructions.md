# Vivarium Agent Simulation Server
This MCP server provides tools to run agent-based simulations using Lua scripts. Agents can send and receive messages, maintain state, and interact within the simulation environment. Simulation time is managed in discrete steps. Simulation can be reset to its initial state.

## Usage Flow
1. **Restart simulation** using `reset_simulation` to clear previous state
2. **Create entities** using `create_entity` - each entity needs a unique ID and Lua script
3. **Run simulation** using `run_simulation_steps` to advance the simulation
4. **Check state** using `list_entities` to see all entities in the simulation

## Lua Script Requirements
Each entity script MUST define TWO functions:

1. **`update(current_time, msgs)`** - Processes messages and executes entity logic:
current_time: current simulation time in seconds
msgs: table of incoming messages (each message has `kind` and `content` fields)
```lua
function update(current_time, msgs)
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

## Available Lua API:
- `self.id` - entity's unique identifier
- `self.send_msg(target_id, msg_type, content, delay)` 
  - sends message to another entity
  - target_id is the recipient entity ID
  - content can be a string OR a Lua table
  - delay is in seconds
- `world.list_entities()` - get list of all entity IDs
- `world.record_metric(name, value)` - record a custom metric
    - name: metric name (string)
    - value: metric value (number)

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

function update(current_time, msgs)
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

function update(current_time, msgs)
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