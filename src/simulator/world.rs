use crate::simulator::messaging::{Message, MessageBus};
use std::{cell::RefCell, collections::HashMap, time};

pub struct World {
    msg_bus: RefCell<MessageBus>,
    entities: HashMap<u32, RefCell<crate::simulator::entity::Entity>>,
    simulation_time: RefCell<time::Instant>,
}

impl World {
    pub fn new() -> Self {
        World {
            msg_bus: RefCell::new(MessageBus::new()),
            entities: HashMap::new(),
            simulation_time: RefCell::new(time::Instant::now()), // Initialize to current time
        }
    }

    pub fn add_entity(&mut self, entity: crate::simulator::entity::Entity) {
        self.entities.insert(entity.get_id(), RefCell::new(entity));
    }

    pub fn fetch_message(&self) -> Option<Message> {
        self.msg_bus
            .borrow_mut()
            .get_deliverable_message(*self.simulation_time.borrow())
    }

    pub fn update_entities(&self) -> Result<(), String> {
        while let Some(msg) = self.fetch_message() {
            match msg.receiver {
                crate::simulator::messaging::MessageReceiver::Entity { id, .. } => {
                    if let Some(entity) = self.entities.get(&id) {
                        entity.borrow_mut().receive_message(&msg, &self);
                    }
                }
                _ => (),
            }
        }

        for entity in self.entities.values() {
            entity.borrow_mut().update()?;
        }

        Ok(())
    }

    pub fn process_commands(&mut self) {
        // Placeholder for processing world-level commands
    }

    pub fn update_simulation_time(&self, delta: time::Duration) {
        let mut sim_time = self.simulation_time.borrow_mut();
        *sim_time += delta;
    }

    pub fn send_msg(&self, entity_id: u32, kind: String, content: String) {
        self.schedule_msg(entity_id, kind, content, time::Duration::from_secs(0));
    }

    pub fn schedule_msg(
        &self,
        entity_id: u32,
        kind: String,
        content: String,
        delay: time::Duration,
    ) {
        //self.msg_bus.borrow_mut().schedule_message(message);
        self.msg_bus.borrow_mut().schedule_message(
            crate::simulator::messaging::MessageReceiver::Entity { id: entity_id },
            kind,
            crate::simulator::messaging::MessageContent::Text(content),
            time::Instant::now(),
            delay,
        );
    }
}
