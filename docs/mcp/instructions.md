# Vivarium Agent Simulation Server
This MCP server provides tools to run agent-based simulations using Lua scripts. Agents can send and receive messages, maintain state, and interact within the simulation environment. Simulation time is managed in discrete steps. Simulation can be reset to its initial state.

## Usage Flow
1. **Initialize project structure** with `initialize_project` (optional)
2. **Load project into memory** with `load_project`
3. **Run simulation** using `advance_simulation`
4. **Inspect state** with `list_entities`, `get_entity_state`, and `get_project_state`
5. **Inspect metrics** with `list_metrics`, `get_metric`, and `get_metrics`
6. **Manage snapshots** with `list_project_snapshots`, `save_project_snapshot`, and `load_project_snapshot`
7. **Unload project** with `unload_project`

## Failures
The server will return errors for invalid operations, such as attempting to load a project that already exists. If a loaded project exceeds the maximum allowed number of entities (10,000), a `WorldCapacityExceeded` error will be returned. Unload the project if no longer needed.

## Lua Script Requirements
Each entity script MUST define THREE functions;

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

3. **`set_state(state)`** - Accepts a table to restore the entity's state (for snapshots):
```lua
function set_state(state)
    health = state.health
    position = state.position
    active = state.active
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
- `self.destroy(entity_id)` - destroy the entity with the given ID
- `self.spawn_entity(entity_id, script_id, initial_state)` - spawn a new entity with the given script and optional initial state
    - entity_id: ID of the new entity
    - script_id: ID of the script to use for the new entity
    - initial_state: optional table to set the initial state of the new entity

**Structured messages**: `self.send_msg("agent2", "Status", {health=100, x=10, y=20}, 0)`
   - No unsafe msg.content access patterns!
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

function set_state(state)
    pos_x = state.pos_x
    pos_y = state.pos_y
    velocity = state.velocity
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

function set_state(state)
    health = state.health
    pos_x = state.pos_x
    pos_y = state.pos_y
end
```

Run the simulation for 10 steps to observe the structured message exchange.

## Available Tools

### World Management
- **`initialize_project`** - Create a new project directory containing `world.yaml`, starter Lua script, and initial snapshot files
- **`load_project`** - Load a project from `world.yaml` and selected snapshot into the in-memory runtime
- **`unload_project`** - Unload a loaded project from memory by project name
- **`list_projects`** - List all loaded projects
- **`get_project_state`** - Get the overall state of the loaded project, including simulation time, entity count, and pending message count

### Simulation Control
- **`advance_simulation`** - Advance the simulation by running multiple time steps with a specified step duration. Each step processes pending messages and executes entity update() functions

### Snapshot Management
- **`list_project_snapshots`** - List available snapshots for a loaded project
- **`save_project_snapshot`** - Save current loaded world state to a named snapshot under project `snapshots/`
- **`load_project_snapshot`** - Restore loaded world state from a named snapshot or `latest`

### Entity Management
- **`list_entities`** - List all entities currently in the simulation. Returns their IDs which can be used as targets for sending messages (optionally include entity states)
- **`get_entity_state`** - Get the current state of a specific entity by its ID
- **`set_entity_state`** - Set the state of a specific entity by its ID. The state must be a JSON object compatible with the entity's Lua script

### Metrics
- **`list_metrics`** - List the names of all available metrics in the simulation world
- **`get_metric`** - Get the current values of a specific metric by name
- **`get_metrics`** - Get the current values of multiple metrics by their names