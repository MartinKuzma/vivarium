use crate::ecs::Component;
use crate::ecs::messaging::{MessageBus, MessageContent, MessageReceiver};
use crate::ecs::system::System;
use std::any::TypeId;
use std::cell::RefCell;
use crate::grid::GridComponent;

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
        print!("LuaScriptComponent of entity {} updating at step {}\n", self.entity_id, current_step);
    }

    fn entity_id(&self) -> u32 {
        self.entity_id
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) {
        self
    }
}

pub struct LuaScriptSystem {
    scripts: Vec<LuaScriptComponent>,
}

impl LuaScriptSystem {
    pub fn new() -> Self {
        //let lua = Lua::new();
        //lua.create_function(move |_, msg: )


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

        let luaComponents = ctx.get_components::<LuaScriptComponent>();
        if let Some(luaComps) = luaComponents {
            for (_, comp) in luaComps.iter() {
                 

                if let Some(grid) = ctx.get_component::<GridComponent>(comp.entity_id()) {
                    println!(
                        "Entity {} is at position ({}, {})\n",
                        comp.entity_id(), grid.pos_x, grid.pos_y
                    );
                }
            }
        }
    }
}