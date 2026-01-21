use crate::simulator::messaging::{Message, MessageBus};
use std::ops::DerefMut;
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap, time};
use crate::simulator::Entity;

pub struct World {
    msg_bus: Rc<RefCell<MessageBus>>,
    state: Rc<RefCell<WorldState>>,
}

pub struct WorldState {
    simulation_time: time::Instant,
    entities: HashMap<String, RefCell<Entity>>,
}

impl World {
    pub fn new() -> Self {
        World {
            msg_bus: Rc::new(RefCell::new(MessageBus::new())),
            state: Rc::new(RefCell::new(WorldState {
                entities: HashMap::new(),
                simulation_time: time::Instant::now(), // Initialize to current time
            })),
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
        )?;

        self.get_state_mut()
            .entities
            .insert(id, RefCell::new(entity));
        Ok(())
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

    pub fn update(&mut self, delta: time::Duration) -> Result<(), String> {
        let messages = self.fetch_messages();
        for msg in messages {
            match msg.receiver {
                crate::simulator::messaging::MessageReceiver::Entity { ref id, .. } => {
                    if let Some(entity) = self.get_state_ref().entities.get(id) {
                        entity.borrow_mut().receive_message(msg);
                    }
                }
                crate::simulator::messaging::MessageReceiver::Radius2D { x, y, radius }  => {
                    // TODO: Implement radius-based message delivery
                }
                _ => (),
            }
        }

        for entity in self.get_state_ref().entities.values() {
            entity.borrow_mut().update()?;
        }

        self.get_state_mut().simulation_time += delta;
        self.msg_bus.borrow_mut().update_time(self.get_state_ref().simulation_time);
        Ok(())
    }

    fn get_state_ref(&self) -> std::cell::Ref<'_, WorldState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> std::cell::RefMut<'_, WorldState> {
        self.state.borrow_mut()
    }
}

impl WorldState {
    pub fn get_entities(&self) -> &HashMap<String, RefCell<Entity>> {
        &self.entities
    }
}