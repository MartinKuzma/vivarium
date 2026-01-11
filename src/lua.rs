use crate::ecs::component::Component;
use crate::ecs::messaging::{MessageBus, MessageContent, MessageReceiver};
use crate::ecs::system::System;
use std::any::TypeId;

use mlua::{Lua, Result as LuaResult, Value as LuaValue};

pub struct LuaScriptComponent {
    entity_id: u32,
    lua_script: String,
}

impl LuaScriptComponent {
    pub fn new(entity_id: u32, lua_script: String) -> Self {
        let lua = Lua::new();

        let _ = lua.load(&lua_script).exec();

        LuaScriptComponent {
            entity_id,
            lua_script,
        }
    }
}

impl Component for LuaScriptComponent {
    fn update(&mut self, current_step: u32) {
        print!(
            "Updating LuaScriptComponent of entity {} at step {}\n",
            self.entity_id, current_step
        );
        // Here you would typically execute the Lua script associated with this component
    }

    fn entity_id(&self) -> u32 {
        self.entity_id
    }
}

pub struct LuaScriptSystem {
    scripts: Vec<LuaScriptComponent>,
}

impl LuaScriptSystem {
    pub fn new() -> Self {
        LuaScriptSystem {
            scripts: Vec::new(),
        }
    }

    pub fn add_script(&mut self, script: LuaScriptComponent) {
        self.scripts.push(script);
    }
}

impl System for LuaScriptSystem {
    fn update(&mut self, current_step: u32, ctx: &mut crate::ecs::world::WorldContext) {
        print!("LuaScriptSystem updating at step {}\n", current_step);

        
        for script in self.scripts.iter_mut() {
            // Execute the update method of each LuaScriptComponent
            // Collect messages or interact with the world context
        }
    }
}