use crate::simulator::messaging::{Message, MessageContent};
use crate::simulator::world::WorldState;
use mlua::Lua;
use mlua::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time;

// Serialize a Lua table into a Lua code string for efficient transfer between VMs
fn serialize_lua_table(lua: &Lua, table: &LuaTable) -> LuaResult<String> {
    let mut result = String::from("{");
    let mut first = true;

    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;

        if !first {
            result.push(',');
        }
        first = false;

        match key {
            LuaValue::String(s) => {
                result.push_str(&format!("[\"{}\"]", s.to_str()?));
            }
            LuaValue::Integer(i) => {
                result.push_str(&format!("[{}]", i));
            }
            _ => continue,
        }

        result.push('=');

        match value {
            LuaValue::String(s) => {
                result.push_str(&format!("\"{}\"", s.to_str()?.replace("\"", "\\\"")));
            }
            LuaValue::Integer(i) => {
                result.push_str(&i.to_string());
            }
            LuaValue::Number(n) => {
                result.push_str(&n.to_string());
            }
            LuaValue::Boolean(b) => {
                result.push_str(if b { "true" } else { "false" });
            }
            LuaValue::Table(nested) => {
                result.push_str(&serialize_lua_table(lua, &nested)?);
            }
            _ => {
                result.push_str("nil");
            }
        }
    }

    result.push('}');
    Ok(result)
}

pub struct LuaScriptController {
    lua_vm: mlua::Lua,
    update_fn: mlua::RegistryKey,
    get_state_fn: mlua::RegistryKey,
    messages: Vec<Message>,
}

impl LuaScriptController {
    pub fn new(
        id: String,
        script: String,
        msg_bus: Rc<RefCell<crate::simulator::messaging::MessageBus>>,
        world_state: Rc<RefCell<WorldState>>,
    ) -> Result<Self, mlua::Error> {
        Self::init_lua(&id, script, msg_bus, world_state)
    }

    fn init_lua(
        id: &String,
        script: String,
        msg_bus: Rc<RefCell<crate::simulator::messaging::MessageBus>>,
        world_state: Rc<RefCell<WorldState>>,
    ) -> LuaResult<LuaScriptController> {
        let lua = Lua::new();
        let self_lib = lua.create_table()?;
        self_lib.set("id", id.clone())?;

        // Function to send message to another entity
        let msg_bus_clone = msg_bus.clone();
        self_lib.set(
            "send_msg",
            lua.create_function(
                move |lua_ctx, (receiver_id, kind, content, delay): (String, String, LuaValue, u64)| {
                    // Serialize Lua value to Lua code string for efficient cross-VM transfer
                    let serialized = match content {
                        LuaValue::String(s) => {
                            // For strings, store as text for backward compatibility
                            let text: String = s.to_str()?.to_string();
                            MessageContent::Text(text)
                        }
                        LuaValue::Table(tbl) => {
                            // Serialize table to Lua code
                            let serialized = serialize_lua_table(lua_ctx, &tbl)?;
                            MessageContent::LuaTable(serialized)
                        }
                        _ => {
                            // For other types, use debug representation
                            MessageContent::Text(format!("{:?}", content))
                        }
                    };

                    msg_bus_clone.borrow_mut().schedule_message(
                        crate::simulator::messaging::MessageReceiver::Entity { id: receiver_id },
                        kind,
                        serialized,
                        time::Duration::from_secs(delay),
                    );

                    Ok(())
                },
            )?,
        )?;

        let msg_bus_clone = msg_bus.clone();
        self_lib.set(
            "broadcast_msg",
            lua.create_function(move |_, (x, y, radius, kind, content)| {
                msg_bus_clone.borrow_mut().schedule_message(
                    crate::simulator::messaging::MessageReceiver::Radius2D { x, y, radius },
                    kind,
                    MessageContent::Text(content),
                    time::Duration::from_secs(1),
                );
                Ok(())
            })?,
        )?;

        lua.globals().set("self", self_lib)?;

        let world_lib = lua.create_table()?;

        // List entities
        let world_state_clone = world_state.clone();
        world_lib.set(
            "list_entities",
            lua.create_function(move |lua_ctx, ()| {
                let res_table = lua_ctx.create_table()?;
                world_state_clone
                    .borrow()
                    .get_entities()
                    .keys()
                    .enumerate()
                    .for_each(|(_, entity_id)| {
                        res_table.push(entity_id.clone()).unwrap();
                    });
                Ok(res_table)
            })?,
        )?;

        // Get current simulation time
        let world_state_clone = world_state.clone();
        world_lib.set(
            "get_time",
            lua.create_function(move |_, ()| {
                let sim_time = world_state_clone.borrow().get_simulation_time();
                Ok(sim_time.as_secs())
            })?,
        )?;

        lua.globals().set("world", world_lib)?;

        lua.load(script).exec()?;

        // Script needs to have update function
        let update_function: LuaFunction = lua.globals().get("update")?;
        let update_function_reg = lua.create_registry_value(update_function)?;

        // Scripts needs to have get_state function in order to serialize state
        let get_state_function: LuaFunction = lua.globals().get("get_state")?;
        let get_state_function_reg = lua.create_registry_value(get_state_function)?;

        Ok(LuaScriptController {
            lua_vm: lua,
            update_fn: update_function_reg,
            get_state_fn: get_state_function_reg,
            messages: Vec::new(),
        })
    }

    pub fn update(&mut self) -> Result<(), String> {
        let msgs_table = self.lua_vm.create_table().map_err(|e| e.to_string())?;

        //TODO: create separate function for this part
        while let Some(msg) = self.messages.pop() {
            let msg_table = self.lua_vm.create_table().map_err(|e| e.to_string())?;

            match &msg.content {
                crate::simulator::messaging::MessageContent::Text(text) => {
                    msg_table
                        .set("content", text.clone())
                        .map_err(|e| e.to_string())?;
                }
                crate::simulator::messaging::MessageContent::LuaTable(lua_code) => {
                    // Deserialize Lua table with zero parsing overhead
                    let value = self
                        .lua_vm
                        .load(lua_code)
                        .eval::<LuaValue>()
                        .map_err(|e| e.to_string())?;
                    msg_table.set("content", value).map_err(|e| e.to_string())?;
                }
            }

            msg_table
                .set("kind", msg.kind.clone())
                .map_err(|e| e.to_string())?;
            msgs_table.push(msg_table).map_err(|e| e.to_string())?;
        }

        let result = self
            .lua_vm
            .registry_value::<LuaFunction>(&self.update_fn)
            .and_then(|func| func.call::<()>(msgs_table));

        result.map_err(|e| format!("Error executing update function: {}", e))
    }

    pub fn get_serialized_state(&self) -> Result<String, String> {
        let state = self
            .lua_vm
            .registry_value::<LuaFunction>(&self.get_state_fn)
            .and_then(|func| func.call::<LuaTable>(()))
            .map_err(|e| format!("Error calling get_state function: {}", e))?;

        return serialize_lua_table(&self.lua_vm, &state)
            .map_err(|e| format!("Error serializing state table: {}", e));
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }
}
