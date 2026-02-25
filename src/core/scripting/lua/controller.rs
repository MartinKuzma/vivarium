use crate::core::errors::CoreError;
use crate::core::messaging;
use crate::core::messaging::Command;
use crate::core::messaging::JSONObject;
use crate::core::messaging::Message;
use crate::core::scripting::lua::convert::{convert_to_json, convert_to_lua_table};
use crate::core::world::State;

use mlua::Lua;
use mlua::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct LuaScriptController {
    lua_vm: mlua::Lua,
    update_fn: mlua::RegistryKey,
    get_state_fn: mlua::RegistryKey,
    set_state_fn: mlua::RegistryKey,

    incoming_msgs: Vec<Message>, // Incoming messages to be processed on next update
    command_queue: Rc<RefCell<Vec<Command>>>, // Queue of commands to be executed by the world after update
}

impl LuaScriptController {
    pub fn new(
        id: String,
        script: &String,
        world_state: Rc<RefCell<State>>,
    ) -> Result<Self, mlua::Error> {
        Self::init_lua(&id, &script, world_state)
    }

    fn init_lua(
        id: &String,
        script: &String,
        world_state: Rc<RefCell<State>>,
    ) -> LuaResult<LuaScriptController> {
        let lua = Lua::new();
        let command_queue = Rc::new(RefCell::new(Vec::new()));

        register_lua_functions(&lua, id, command_queue.clone(), world_state)?;

        lua.load(script).exec()?;

        // Script needs to have update function
        let update_function: LuaFunction = lua.globals().get("update")?;
        let update_function_reg = lua.create_registry_value(update_function)?;

        // Scripts need to have get_state function in order to serialize state
        let get_state_function: LuaFunction = lua.globals().get("get_state")?;
        let get_state_function_reg = lua.create_registry_value(get_state_function)?;
        // Scripts need to have set_state function in order to restore the state
        let set_state_function: LuaFunction = lua.globals().get("set_state")?;
        let set_state_function_reg = lua.create_registry_value(set_state_function)?;

        Ok(LuaScriptController {
            lua_vm: lua,
            update_fn: update_function_reg,
            get_state_fn: get_state_function_reg,
            set_state_fn: set_state_function_reg,
            incoming_msgs: Vec::new(),
            command_queue: command_queue,
        })
    }

    // Set the internal state of the Lua script from a serialized Lua table string
    pub fn set_state(&mut self, state: messaging::JSONObject) -> Result<(), CoreError> {
        let state_table =
            convert_to_lua_table(&self.lua_vm, &state).map_err(|e| CoreError::ScriptState {
                message: format!("Error converting JSON to Lua table: {}", e),
            })?;

        let result = self
            .lua_vm
            .registry_value::<LuaFunction>(&self.set_state_fn)
            .and_then(|func| func.call::<()>(state_table));

        result.map_err(|e| CoreError::ScriptState {
            message: format!("Error executing set_state function: {}", e),
        })
    }

    pub fn update(&mut self, simulation_time: u64) -> Result<Vec<Command>, CoreError> {
        let msgs_table = self
            .create_messages_table()
            .map_err(|e| CoreError::ScriptExecution {
                message: format!("Error creating messages table: {}", e),
            })?;

        let result = self
            .lua_vm
            .registry_value::<LuaFunction>(&self.update_fn)
            .and_then(|func| func.call::<()>((simulation_time, msgs_table)));

        result.map_err(|e| CoreError::ScriptExecution {
            message: format!("Error executing update function: {}", e),
        })?;

        Ok(std::mem::take(&mut *self.command_queue.borrow_mut()))
    }

    fn create_messages_table(&mut self) -> LuaResult<LuaTable> {
        let msgs_table = self.lua_vm.create_table()?;

        for msg in &self.incoming_msgs {
            let msg_table = self.lua_vm.create_table()?;

            msg_table.set("content", convert_to_lua_table(&self.lua_vm, &msg.content)?)?;
            msg_table.set("kind", msg.kind.clone())?;
            msgs_table.push(msg_table)?;
        }

        self.incoming_msgs.clear();

        Ok(msgs_table)
    }

    pub fn get_state(&self) -> Result<JSONObject, CoreError> {
        let state = self
            .lua_vm
            .registry_value::<LuaFunction>(&self.get_state_fn)
            .and_then(|func| func.call::<LuaTable>(()))
            .map_err(|e| CoreError::ScriptState {
                message: format!("Error calling get_state function: {}", e),
            })?;

        return convert_to_json(&self.lua_vm, &state).map_err(|e| CoreError::ScriptState {
            message: format!("Error serializing state table: {}", e),
        });
    }

    pub fn push_message(&mut self, msg: Message) {
        self.incoming_msgs.push(msg);
    }
}

fn register_lua_functions(
    lua: &Lua,
    id: &String,
    command_queue: Rc<RefCell<Vec<Command>>>,
    world_state: Rc<RefCell<State>>,
) -> LuaResult<()> {
    register_self_lib(lua, id, command_queue.clone())?;
    register_world_lib(lua, command_queue, world_state)?;
    Ok(())
}

fn register_self_lib(
    lua: &Lua,
    id: &String,
    command_queue: Rc<RefCell<Vec<Command>>>,
) -> LuaResult<()> {
    let self_lib = lua.create_table()?;
    self_lib.set("id", id.clone())?;

    // Function to send message to another entity
    let command_queue_clone = command_queue.clone();
    let id_clone = id.clone();
    let send_msg_fn = lua.create_function(
        move |lua_ctx, (receiver_id, kind, content, delay): (String, String, LuaTable, u64)| {
            command_queue_clone.borrow_mut().push(Command::SendMessage {
                sender: id_clone.clone(),
                receiver: crate::core::messaging::MessageReceiver::Entity { id: receiver_id },
                kind,
                content: convert_to_json(lua_ctx, &content)?,
                delay,
            });

            Ok(())
        },
    )?;

    // Broadcast message to entities within a radius
    let id_clone = id.clone();
    let command_queue_clone = command_queue.clone();
    let broadcast_msg_fn = lua.create_function(move |lua_ctx, (x, y, radius, kind, content)| {
        command_queue_clone.borrow_mut().push(Command::SendMessage {
            sender: id_clone.clone(),
            receiver: crate::core::messaging::MessageReceiver::Radius2D { x, y, radius },
            kind,
            content: convert_to_json(lua_ctx, &content)?,
            delay: 1,
        });
        Ok(())
    })?;

    // Send system message to destroy an entity
    let command_queue_clone = command_queue.clone();
    let destroy_fn = lua.create_function(move |_, entity_id| {
        command_queue_clone
            .borrow_mut()
            .push(Command::RemoveEntity { id: entity_id });

        Ok(())
    })?;

    let command_queue_clone = command_queue.clone();
    let spawn_fn = lua.create_function(
        move |lua_ctx,
              (entity_id, script_id, initial_state): (String, String, Option<LuaTable>)| {
            let initial_state_json = match initial_state {
                Some(table) => Some(convert_to_json(&lua_ctx, &table)?),
                None => None,
            };

            let spawn_cmd = Command::SpawnEntity {
                entity_id,
                script_id,
                initial_state: initial_state_json,
            };

            command_queue_clone.borrow_mut().push(spawn_cmd);
            Ok(())
        },
    )?;

    self_lib.set("destroy", destroy_fn)?;
    self_lib.set("broadcast_msg", broadcast_msg_fn)?;
    self_lib.set("send_msg", send_msg_fn)?;
    self_lib.set("spawn_entity", spawn_fn)?;

    lua.globals().set("self", self_lib)?;
    Ok(())
}

fn register_world_lib(
    lua: &Lua,
    command_queue: Rc<RefCell<Vec<Command>>>,
    world_state: Rc<RefCell<State>>,
) -> LuaResult<()> {
    let world_lib = lua.create_table()?;

    // List entities in the world
    let world_state_clone = world_state.clone();
    let list_entities_fn = lua.create_function(move |lua_ctx, ()| {
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
    })?;

    // Record a metric
    let record_metric_fn = lua.create_function(move |_, (name, value): (String, f64)| {
        command_queue
            .borrow_mut()
            .push(Command::RecordMetric { name, value });
        Ok(())
    })?;

    world_lib.set("list_entities", list_entities_fn)?;
    world_lib.set("record_metric", record_metric_fn)?;

    lua.globals().set("world", world_lib)?;
    Ok(())
}
