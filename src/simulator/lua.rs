use crate::simulator::messaging::{Message, MessageContent};
use crate::simulator::world::WorldState;
use mlua::prelude::*;
use mlua::{Lua};
use std::cell::RefCell;
use std::rc::Rc;
use std::time;

pub struct LuaScriptController {
    lua_vm: mlua::Lua,
    update_fn: mlua::RegistryKey,
    messages : Vec<Message>,
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
        let entity_lib = lua.create_table()?;
        entity_lib.set("id", id.clone())?;

        // Function to send message to another entity
        let msg_bus_clone = msg_bus.clone();
        entity_lib.set(
            "send_msg",
            lua.create_function( move |_, (receiver_id, kind, content, delay)| {
                msg_bus_clone.borrow_mut().schedule_message(
                    crate::simulator::messaging::MessageReceiver::Entity { id: receiver_id },
                    kind,
                    MessageContent::Text(content),
                    time::Duration::from_secs(delay),
                );

                Ok(())
            })?,
        )?;

        entity_lib.set(
            "broadcast_msg",
            lua.create_function( move |_, (x, y, radius, kind, content)| {
                msg_bus.borrow_mut().schedule_message(
                    crate::simulator::messaging::MessageReceiver::Radius2D { x, y, radius },
                    kind,
                    MessageContent::Text(content),
                    time::Duration::from_secs(1),
                );
                Ok(())
            })?,
        )?;

        // List entities
        entity_lib.set(
            "list_entities",
            lua.create_function(move |lua_ctx, ()| {
                let res_table = lua_ctx.create_table()?;
                world_state
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

        lua.globals().set("entity", entity_lib)?;

        lua.load(script).exec()?;

        // Script needs to have update function
        let update_function: LuaFunction = lua.globals().get("update")?;
        let update_function_reg = lua.create_registry_value(update_function)?;
        
        Ok(LuaScriptController {
            lua_vm: lua,
            update_fn: update_function_reg,
            messages : Vec::new(),
        })
    }

     pub fn update(&mut self) -> Result<(), String> {
        let msgs_table = self.lua_vm
        .create_table()
        .map_err(|e| {e.to_string()})?;

        while let Some(msg) = self.messages.pop() {
            let msg_table = self.lua_vm.create_table()
            .map_err(|e| {e.to_string()})?;
            match &msg.content {
                crate::simulator::messaging::MessageContent::Text(text) => {
                    msg_table.set("content", text.clone())
                    .map_err(|e| {e.to_string()})?;
                }
            }

            msg_table.set("kind", msg.kind.clone())
            .map_err(|e| {e.to_string()})?;
            msgs_table.push(msg_table)
            .map_err(|e| {e.to_string()})?;
        }

        let result = self.lua_vm
            .registry_value::<LuaFunction>(&self.update_fn)
            .and_then(|func| func.call::<()>(msgs_table));

        result.map_err(|e| {
            format!("Error executing update function: {}", e)
        })
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }
}