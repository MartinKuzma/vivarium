# vivarium
Agent based simulator for LLMs with scriptable behaviours. Let your LLM reason and simulate!

# Features
- Entities with Lua-scripted behaviours
- Messaging system between entities with delayed message delivery

## TODO
- Instead of entity owning the LuaScriptManager, have a list of LuaScriptManager in the world that can have references to entities. Entities would be only data holders. This would allow easier cross-entity interactions via Lua scripts.
- Implement MCP server for simulations (POC level)
- Implement simulation manager to handle multiple simulations