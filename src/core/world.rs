use serde::de;

use crate::core::Entity;
use crate::core::messaging::{Message, MessageBus};
use crate::core::metrics::Metrics;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

pub struct World {
    msg_bus: MessageBus,
    state: Rc<RefCell<WorldState>>,
    metrics: Metrics,
    simulation_time: u64, //TODO: Replace with some shared clock
}

pub struct WorldState {
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
            msg_bus: MessageBus::new(),
            state: Rc::new(RefCell::new(WorldState {
                entities: HashMap::new(),
            })),
            metrics: Metrics::new(start_time),
        }
    }

    pub fn create_entity(&mut self, id: String, script: String) -> Result<(), mlua::Error> {
        let entity = crate::core::entity::Entity::new(id.clone(), script, self.state.clone())?;

        self.get_state_mut()
            .entities
            .insert(id, RefCell::new(entity));
        Ok(())
    }

    pub fn remove_entity(&mut self, id: &String) -> Option<RefCell<Entity>> {
        self.get_state_mut().entities.remove(id)
    }

    pub fn fetch_messages(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        while let Some(msg) = self.msg_bus.get_deliverable_message() {
            messages.push(msg);
        }

        messages
    }

    pub fn update(&mut self, delta: u64) -> Result<WorldUpdateResult, String> {
        let mut update_result = WorldUpdateResult {
            delivered_messages: Vec::new(),
        };

        // Update simulation time
        self.update_simulation_time(self.simulation_time + delta);
        self.deliver_messages(&mut update_result);

        let mut commands = Vec::new();

        for entity in self.get_state_ref().entities.values() {
            let entity_commands = entity.borrow_mut().update(self.simulation_time)?;
            commands.extend(entity_commands);
        }

        self.process_commands(commands);

        Ok(update_result)
    }

    fn process_commands(&mut self, commands: Vec<crate::core::messaging::Command>) {
        for command in commands {
            match command {
                crate::core::messaging::Command::SendMessage {
                    sender,
                    receiver,
                    kind,
                    content,
                    delay,
                } => {
                    self.msg_bus
                        .schedule_message(&sender, receiver, kind, content, delay);
                }
                crate::core::messaging::Command::RemoveEntity { id } => {
                    self.remove_entity(&id);
                }
                crate::core::messaging::Command::RecordMetric { name, value } => {
                    self.metrics.record_metric(&name, value);
                }
            }
        }
    }

    fn deliver_messages(&mut self, update_result: &mut WorldUpdateResult) {
        let messages = self.fetch_messages();
        for msg in messages {
            // Log delivered message
            update_result.delivered_messages.push(msg.clone());

            match msg.receiver {
                crate::core::messaging::MessageReceiver::Entity { ref id, .. } => {
                    if let Some(entity) = self.get_state_ref().entities.get(id) {
                        entity.borrow_mut().receive_message(msg);
                    }
                }
                crate::core::messaging::MessageReceiver::Radius2D { x, y, radius } => {
                    // TODO: Implement radius-based message delivery
                }
            }
        }
    }

    pub fn set_entity_state(&mut self, id: &String, state: &str) -> Result<(), String> {
        if let Some(entity) = self.get_state_ref().entities.get(id) {
            entity.borrow_mut().get_lua_controller_mut().set_state(state)
        } else {
            Err(format!("Entity with ID '{}' not found", id))
        }
    }

    fn update_simulation_time(&mut self, new_time: u64) {
        self.simulation_time = new_time;
        self.msg_bus.update_time(new_time);
        self.metrics.update_time(new_time);
    }

    pub fn get_state_ref(&self) -> std::cell::Ref<'_, WorldState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> std::cell::RefMut<'_, WorldState> {
        self.state.borrow_mut()
    }

    pub fn get_metrics_ref(&self) -> &Metrics {
        &self.metrics
    }
}

impl WorldState {
    pub fn get_entities(&self) -> &HashMap<String, RefCell<Entity>> {
        &self.entities
    }
}
