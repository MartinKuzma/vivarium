use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use super::component::Component;
use super::messaging::{MessageBus, MessageContent};
use super::system::System;

pub struct World {
    components: HashMap<TypeId, HashMap<u32, Box<dyn Component>>>, // Components organized by type and entity ID
    systems: Vec<Box<dyn System>>, // Systems managing entity behavior
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
            components: HashMap::new(),
            systems: Vec::new(),
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

        // let commands = wctx.commands;
        // self.apply_commands(commands);
    }

    pub fn add_component(&mut self, entity_id: u32, component: Box<dyn Component>) {
        let comp_type = component.type_id();
        self.components
            .entry(comp_type)
            .or_insert_with(HashMap::new)
            .insert(entity_id, component);
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
    components: &'a mut HashMap<TypeId, HashMap<u32, Box<dyn Component>>>,
    commands: Vec<Command>,
    msg_bus: &'a mut MessageBus,
}

impl<'a> WorldContext<'a> {
    pub fn new(
        components: &'a mut HashMap<TypeId, HashMap<u32, Box<dyn Component>>>,
        msg_bus: &'a mut MessageBus,
    ) -> Self {
        WorldContext {
            commands: Vec::new(),
            components: components,
            msg_bus: msg_bus,
        }
    }

    pub unsafe fn get_components<T: Component + 'static>(&mut self) -> Option<&mut HashMap<u32, Box<dyn Component>>> {
        self.components.get_mut(&TypeId::of::<T>())
    }

    pub unsafe fn get_component<T : Component + 'static>(
        &self,
        entity_id: u32,
    ) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|comp_map| comp_map.get(&entity_id))
            .and_then(|component| component.as_any().downcast_ref::<T>())
    }

    pub fn get_component_mut<T : Component + 'static>(
        &mut self,
        entity_id: u32,
    ) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|comp_map| comp_map.get_mut(&entity_id))
            .and_then(|component| component.as_mut().as_any_mut().downcast_mut::<T>())
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
}
