# vivarium
Agent based simulator for LLMs with scriptable behaviours. Let your LLM reason and simulate!

# Features
- Entities with Lua-scripted behaviours
- Messaging system between entities with delayed message delivery

## TODO
- Instead of entity owning the LuaScriptManager, have a list of LuaScriptManager in the world that can have references to entities. Entities would be only data holders. This would allow easier cross-entity interactions via Lua scripts.
- Add RNG functions to Lua API
- Implement simulation manager to handle multiple simulations