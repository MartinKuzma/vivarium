use crate::simulator::messaging::{Message, MessageBus};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap, time};

pub struct World {
    // msg_bus: RefCell<MessageBus>,
    // entities: HashMap<u32, RefCell<crate::simulator::entity::Entity>>,
    // simulation_time: time::Instant,
    state: Rc<RefCell<WorldState>>,
}


pub struct WorldState {
    msg_bus: RefCell<MessageBus>,
    simulation_time: time::Instant,
    entities: HashMap<u32, RefCell<crate::simulator::entity::Entity>>,
}

impl World {
    pub fn new() -> Self {
        World {
            state: Rc::new(RefCell::new(WorldState {
                msg_bus: RefCell::new(MessageBus::new()),
                entities: HashMap::new(),
                simulation_time: time::Instant::now(), // Initialize to current time
            })),
        }
    }

    pub fn create_entity(
        &mut self,
        id: u32,
        name: &str,
        script: String,
    ) -> Result<(), mlua::Error> {
        let entity = crate::simulator::entity::Entity::new(id, name, script, self.state.clone())?;
        
        self.get_state_mut()
            .entities
            .insert(id, RefCell::new(entity));
        Ok(())
    }

    pub fn fetch_message(&self) -> Option<Message> {
        let current_time = self.get_state_ref().simulation_time;

        self.get_state_ref()
            .msg_bus
            .borrow_mut()
            .get_deliverable_message(current_time)
    }

    pub fn update_entities(&self) -> Result<(), String> {
        while let Some(msg) = self.fetch_message() {
            match msg.receiver {
                crate::simulator::messaging::MessageReceiver::Entity { id, .. } => {
                    if let Some(entity) = self.get_state_ref().entities.get(&id) {
                        entity.borrow_mut().receive_message(&msg);
                    }
                }
                _ => (),
            }
        }

        for entity in self.get_state_ref().entities.values() {
            entity.borrow_mut().update()?;
        }

        Ok(())
    }

    pub fn process_commands(&mut self) {
        // Placeholder for processing world-level commands
    }

    pub fn update_simulation_time(&mut self, delta: time::Duration) {
        self.get_state_mut().simulation_time += delta;
    }

    fn get_state_ref(&self) -> std::cell::Ref<'_, WorldState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> std::cell::RefMut<'_, WorldState> {
        self.state.borrow_mut()
    }
}

impl WorldState {
    pub fn schedule_msg(
        &self,
        entity_id: u32,
        kind: String,
        content: String,
        delay: time::Duration,
    ) {
        let current_time = self.simulation_time;

        self.msg_bus.borrow_mut().schedule_message(
            crate::simulator::messaging::MessageReceiver::Entity { id: entity_id },
            kind,
            crate::simulator::messaging::MessageContent::Text(content),
            current_time,
            delay,
        );
    }
}
