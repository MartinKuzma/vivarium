use crate::simulator::messaging::{Message};
use crate::simulator::world::WorldState;
use std::cell::RefCell;
use std::rc::Rc;
use crate::simulator::lua::LuaScriptController;

pub struct Entity {
    lua_controller: LuaScriptController,
}

impl Entity {
    pub fn new(
        id: String,
        script: String,
        msg_bus: Rc<RefCell<crate::simulator::messaging::MessageBus>>,
        world_state: Rc<RefCell<WorldState>>,
        metrics: Rc<RefCell<crate::simulator::metrics::Metrics>>,
    ) -> Result<Self, mlua::Error> {
        let lua_controller = LuaScriptController::new(id.clone(), script, msg_bus.clone(), world_state, metrics)?;

        Ok(Entity {
            lua_controller,
        })
    }

    pub fn update(&mut self, current_time: u64) -> Result<(), String> {
        self.lua_controller.update(current_time)
    }

    pub fn receive_message(&mut self, message: Message) {
        self.lua_controller.push_message(message);
    }

    pub fn get_lua_controller(&self) -> &LuaScriptController {
        &self.lua_controller
    }
}
