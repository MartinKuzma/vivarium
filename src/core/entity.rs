use crate::core::errors::CoreError;
use crate::core::messaging::Command;
use crate::core::messaging::{JSONObject, Message};
use crate::core::scripting::lua::LuaScriptController;
use crate::core::world::WorldState;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Entity {
    id: String,
    script_id: String,
    lua_controller: LuaScriptController,
}

impl Entity {
    pub fn new(
        id: String,
        script_id: String,
        script: String,
        initial_state: Option<JSONObject>,
        world_state: Rc<RefCell<WorldState>>,
    ) -> Result<Self, CoreError> {
        let controller_result = LuaScriptController::new(id.clone(), &script, world_state);
        if let Err(e) = &controller_result {
            return Err(CoreError::EntityCreation {
                id,
                message: format!("Failed to create LuaScriptController: {}", e),
            });
        }

        let mut lua_controller = controller_result.unwrap();

        if let Some(state) = initial_state {
            lua_controller.set_state(state)?;
        }

        Ok(Entity {
            id: id.clone(),
            script_id: script_id.clone(),
            lua_controller,
        })
    }

    pub fn update(&mut self, current_time: u64) -> Result<Vec<Command>, CoreError> {
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

    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn get_script_id(&self) -> &String {
        &self.script_id
    }
}
