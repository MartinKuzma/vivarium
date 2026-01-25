use crate::core::messaging::{Message};
use crate::core::world::WorldState;
use crate::core::messaging::Command;
use std::cell::RefCell;
use std::rc::Rc;
use crate::core::scripting::lua::LuaScriptController;

pub struct Entity {
    lua_controller: LuaScriptController,
}

impl Entity {
    pub fn new(
        id: String,
        script: String,
        world_state: Rc<RefCell<WorldState>>,
    ) -> Result<Self, mlua::Error> {
        let lua_controller = LuaScriptController::new(id.clone(), script, world_state)?;

        Ok(Entity {
            lua_controller,
        })
    }

    pub fn update(&mut self, current_time: u64) -> Result<Vec<Command>, String> {
        self.lua_controller.update(current_time)
    }

    pub fn receive_message(&mut self, message: Message) {
        self.lua_controller.push_message(message);
    }

    pub fn get_lua_controller(&self) -> &LuaScriptController {
        &self.lua_controller
    }

    pub fn get_lua_controller_mut(&mut self) -> &mut LuaScriptController {
        &mut self.lua_controller
    }
}
