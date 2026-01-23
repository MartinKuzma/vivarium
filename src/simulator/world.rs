use serde::de;

use crate::simulator::messaging::{Message, MessageBus};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};
use crate::simulator::Entity;
use crate::simulator::metrics::Metrics;

#[derive(Clone)]
pub struct World {
    msg_bus: Rc<RefCell<MessageBus>>,
    state: Rc<RefCell<WorldState>>,
    metrics: Rc<RefCell<Metrics>>,
    simulation_time: u64, //TODO: Replace with some shared clock
}

pub struct WorldState {
    simulation_time: u64,
    entities: HashMap<String, RefCell<Entity>>,
}

pub struct WorldUpdateResult {
    pub delivered_messages: Vec<Message>,
}

impl World {
    pub fn new() -> Self {
        let start_time = 0;

        World {
            simulation_time: 0,
            msg_bus: Rc::new(RefCell::new(MessageBus::new())),
            state: Rc::new(RefCell::new(WorldState {
                entities: HashMap::new(),
                simulation_time: 0,
            })),
            metrics: Rc::new(RefCell::new(Metrics::new(start_time))),
        }
    }

    pub fn create_entity(
        &mut self,
        id: String,
        script: String,
    ) -> Result<(), mlua::Error> {
        let entity = crate::simulator::entity::Entity::new(
            id.clone(),
            script,
            self.msg_bus.clone(),
            self.state.clone(),
            self.metrics.clone(),
        )?;

        self.get_state_mut()
            .entities
            .insert(id, RefCell::new(entity));
        Ok(())
    }

    pub fn remove_entity(&mut self, id: &String) -> Option<RefCell<Entity>> {
        self.get_state_mut().entities.remove(id)
    }

    pub fn fetch_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        while let Some(msg) = self
            .msg_bus
            .borrow_mut()
            .get_deliverable_message()
        {
            messages.push(msg);
        }

        messages
    }

    pub fn update(&mut self, delta: u64) -> Result<WorldUpdateResult, String> {
        let mut update_result = WorldUpdateResult {
            delivered_messages: Vec::new(),
        };

        // Update simulation time
        self.simulation_time += delta;
        self.msg_bus.borrow_mut().update_time(self.simulation_time);
        self.metrics.borrow_mut().update_time(self.simulation_time);
        self.state.borrow_mut().update_time(self.simulation_time);

        let messages = self.fetch_messages();
        for msg in messages {
            // Log delivered message
            update_result.delivered_messages.push(msg.clone());

            match msg.receiver {
                crate::simulator::messaging::MessageReceiver::Entity { ref id, .. } => {
                    if let Some(entity) = self.get_state_ref().entities.get(id) {
                        entity.borrow_mut().receive_message(msg);
                    }
                }
                crate::simulator::messaging::MessageReceiver::Radius2D { x, y, radius }  => {
                    // TODO: Implement radius-based message delivery
                }
            }
        }

        for entity in self.get_state_ref().entities.values() {
            entity.borrow_mut().update(self.simulation_time)?;
        }

        
        Ok(update_result)
    }

    pub fn get_state_ref(&self) -> std::cell::Ref<'_, WorldState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> std::cell::RefMut<'_, WorldState> {
        self.state.borrow_mut()
    }

    pub fn get_metrics_ref(&self) -> std::cell::Ref<'_, Metrics> {
        self.metrics.borrow()
    }
}


impl WorldState {
    pub fn get_entities(&self) -> &HashMap<String, RefCell<Entity>> {
        &self.entities
    }

    pub fn update_time(&mut self, new_time: u64) {
        self.simulation_time = new_time;
    }
}