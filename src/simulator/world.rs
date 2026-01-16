use crate::simulator::messaging::{Message, MessageBus};
use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap, time};

pub struct World {
    msg_bus: Rc<RefCell<MessageBus>>,
    // entities: HashMap<u32, RefCell<crate::simulator::entity::Entity>>,
    // simulation_time: time::Instant,
    state: Rc<RefCell<WorldState>>,
}

pub struct WorldState {
    simulation_time: time::Instant,
    entities: HashMap<u32, RefCell<crate::simulator::entity::Entity>>,
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
        id: u32,
        name: &str,
        script: String,
    ) -> Result<(), mlua::Error> {
        let entity = crate::simulator::entity::Entity::new(
            id,
            name,
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
    // pub fn schedule_msg(
    //     &self,
    //     entity_id: u32,
    //     kind: String,
    //     content: String,
    //     delay: time::Duration,
    // ) {
    //     let current_time = self.simulation_time;

    //     self.msg_bus.borrow_mut().schedule_message(
    //         crate::simulator::messaging::MessageReceiver::Entity { id: entity_id },
    //         kind,
    //         crate::simulator::messaging::MessageContent::Text(content),
    //         current_time,
    //         delay,
    //     );
    // }
}
