use std::rc::Rc;
use std::cell::RefCell;
use crate::message_bus::{Message, MessageBus};

pub trait AgentBehavior {
    fn get_id(&self) -> u32;
    fn get_name(&self) -> &str;

    fn tick(&mut self, current_step: u32, message_bus: &mut MessageBus);
    fn on_message(&mut self, message: Message);
}

// pub trait AgentComponent {
//     fn update(&mut self, current_step: u32);
//     fn get_component_type(&self) -> &str;
// }

pub struct LuaAgent {
    id: u32,
    name: String,
}

impl LuaAgent {
    pub fn new(id: u32, name: String) -> Self {
        LuaAgent { id, name }
    }
}

impl AgentBehavior for LuaAgent {
    fn get_id(&self) -> u32 {
        self.id
    }


    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn tick(&mut self, current_step: u32, message_bus: &mut MessageBus) {
        print!("Agent {} ticking at step {}\n", self.name, current_step);
    }
    
    fn on_message(&mut self, message: Message) {
        print!("Agent {} received message: {}\n", self.name, message.content);
    }
}
