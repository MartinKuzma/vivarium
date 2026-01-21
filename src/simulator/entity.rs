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
    ) -> Result<Self, mlua::Error> {
        let lua_controller = LuaScriptController::new(id.clone(), script, msg_bus.clone(), world_state)?;

        Ok(Entity {
            lua_controller,
        })
    }

    pub fn update(&mut self) -> Result<(), String> {
        self.lua_controller.update()
    }

    pub fn receive_message(&mut self, message: Message) {
        self.lua_controller.push_message(message);
    }
}
