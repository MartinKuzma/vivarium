use mlua::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time;

use crate::simulator::messaging::Message;

struct LuaScriptManager {
    lua_vm: mlua::Lua,
    update_fn: mlua::RegistryKey,
    on_message_function: mlua::RegistryKey,
}

pub struct Entity {
    id: u32,
    name: String,
    lua_script_manager : LuaScriptManager,
}

impl Entity {
    pub fn new(id: u32, name: &str, script: String, world: Rc<RefCell<crate::simulator::world::World>>) -> Result<Self, mlua::Error> {
        let lua_handler = Self::init_lua(id, script, world)?;

         Ok(Entity {
            id,
            name: name.to_string(),
            lua_script_manager: lua_handler,
        })
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    fn init_lua(id: u32, script: String, world: Rc<RefCell<crate::simulator::world::World>>) -> LuaResult<LuaScriptManager> {
        let lua = Lua::new();
        let entity_lib = lua.create_table().unwrap();
        entity_lib.set("id", id)?;

        // Function to send message to another entity
        let world_clone = world.clone();
        entity_lib.set("send_msg", lua.create_function(move |_lua, (receiver_id, kind, content)| {
            world_clone.borrow().schedule_msg(
                receiver_id,
                kind,
                content,
                time::Duration::from_secs(1),
            );

            Ok(())
        })?)?;

        //TODO: broadcast message function

        lua.globals().set("entity", entity_lib)?;

        // Script needs to have update and on_message functions
        lua.load(script).exec()?;

        let update_function : LuaFunction = lua.globals().get("update")?;
        let on_message_function : LuaFunction = lua.globals().get("on_message")?;

        let update_function_reg = lua.create_registry_value(update_function)?;
        let on_message_function_reg = lua.create_registry_value(on_message_function)?;

        Ok(LuaScriptManager {
            lua_vm: lua,
            update_fn: update_function_reg,
            on_message_function: on_message_function_reg,
        })
    }

    pub fn update(&mut self) -> Result<(), String> {
        self.lua_script_manager.lua_vm
            .registry_value::<LuaFunction>(&self.lua_script_manager.update_fn)
            .and_then(|func| func.call::<()>(()))
            .or_else(|e| Err(format!("Error executing update function for entity {}: {}", self.id, e)))
    }

    pub fn receive_message(&mut self, message: &crate::simulator::messaging::Message, world: &crate::simulator::world::World) {
        self.lua_script_manager.lua_vm
            .registry_value::<LuaFunction>(&self.lua_script_manager.on_message_function)
            .and_then(|func| {
                let msg_table = self.lua_script_manager.lua_vm.create_table()?;
                match &message.content {
                    crate::simulator::messaging::MessageContent::Text(text) => {
                        msg_table.set("content", text.clone())?;
                    }
                    crate::simulator::messaging::MessageContent::Data(data) => {
                        msg_table.set("content", data.clone())?;
                    }
                }
                func.call::<()>(msg_table)
            })
            .unwrap_or_else(|e| {
                eprintln!("Error executing on_message function for entity {}: {}", self.id, e);
            });
    }
}