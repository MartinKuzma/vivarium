use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use super::component::Component;
use super::messaging::{MessageBus, MessageContent};
use super::system::System;

pub struct World {
    systems: Vec<Box<dyn System>>, // Systems managing entity behavior
    components: HashMap<TypeId, Box<dyn Any>>, // Components categorized by type and entity ID
    msg_bus: MessageBus,
}

pub enum Command {
    RemoveEntity(u32),                     // Remove entity by ID
    AddComponent(u32, Box<dyn Component>), // Add component to entity by ID
    RemoveComponent(u32, TypeId),          // Remove component by entity ID and component type
}

impl World {
    pub fn new() -> Self {
        World {
            systems: Vec::new(),
            components: HashMap::new(),
            msg_bus: MessageBus::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn update(&mut self, current_step: u32) {
        let mut wctx = WorldContext::new(&mut self.components, &mut self.msg_bus);

        for system in self.systems.iter_mut() {
            system.update(current_step, &mut wctx);
        }

        let commands = wctx.commands;
        self.apply_commands(commands);
    }

    fn apply_commands(&mut self, commands: Vec<Command>) {
        for command in commands {
            match command {
                Command::RemoveEntity(entity_id) => {
                    // for comp_map in self.components.values_mut() {
                    //     comp_map.remove(&entity_id);
                    // }
                }
                Command::AddComponent(entity_id, component) => {
                    let comp_type = component.type_id(); //TODO: or as_ref?
                    //TODO: Fix this
                    // self.components
                    //     .entry(comp_type)
                    //     .or_insert_with(HashMap::new)
                    //     .insert(entity_id, component);
                }
                Command::RemoveComponent(entity_id, component_type) => {
                    // if let Some(comp_map) = self.components.get_mut(&component_type) {
                    //     comp_map.remove(&entity_id);
                    // }
                }
            }
        }
    }
}

pub struct WorldContext<'a> {
    commands: Vec<Command>,
    components: &'a mut HashMap<TypeId, Box<dyn Any>>,
    msg_bus: &'a mut MessageBus,
}

impl<'a> WorldContext<'a> {
    pub fn new(
        components: &'a mut HashMap<TypeId, Box<dyn Any>>,
        msg_bus: &'a mut MessageBus,
    ) -> Self {
        WorldContext {
            commands: Vec::new(),
            components: components,
            msg_bus: msg_bus,
        }
    }

    pub fn get_components<T: Component + 'static>(&mut self) -> Option<&mut HashMap<u32, T>> {
        self.components.get_mut(&TypeId::of::<T>()).and_then(|any| any.downcast_mut::<HashMap<u32, T>>())
    }

    pub fn remove_entity(&mut self, entity_id: u32) {
        self.commands.push(Command::RemoveEntity(entity_id));
    }

    pub fn add_component(&mut self, entity_id: u32, component: Box<dyn Component>) {
        self.commands.push(Command::AddComponent(entity_id, component));
    }

    pub fn remove_component(&mut self, entity_id: u32, component_type: TypeId) {
        self.commands.push(Command::RemoveComponent(entity_id, component_type));
    }

    // pub fn send_message(&mut self,  , content: MessageContent, current_step: u32, delay: u32, recipient_component: TypeId) {
    //     self.msg_bus.schedule_message(sender_id, recipient_id, content, current_step, delay, recipient_component);
    // }
}
