use std::collections::HashMap;
use crate::agent::AgentBehavior;
use crate::message_bus::MessageBus;

pub struct SimulationEngine {
    current_step: u32,
    agents: HashMap<u32, Box<dyn AgentBehavior>>,
    message_bus: MessageBus,
}

impl SimulationEngine {
    pub fn new() -> Self {
        SimulationEngine {
            current_step: 0,
            agents: HashMap::new(),
            message_bus: MessageBus::new(),
        }
    }

    pub fn add_agent(&mut self, agent: Box<dyn AgentBehavior>) {
        self.agents.insert(agent.get_id(), agent);
    }

    fn tick(&mut self) {
        // Update each agent
        for agent in self.agents.values_mut() {
            agent.tick(self.current_step, &mut self.message_bus);
        }

        // Deliver messages scheduled for this step
        while let Some(message) = self.message_bus.get_deliverable_message(self.current_step) {
            print!("Delivering message to agent {}: {}\n", message.recipient_id, message.content);
            // if let Some(agent) = self.agents.get_mut(&message.recipient_id) {
            //     agent.on_message(&message.content);
            // }
        }
    }

    // Advance the simulation by a given number of steps
    pub fn advance_for(&mut self, steps: u32) {
        for _ in 0..steps {
            self.tick();
            self.current_step += 1;
        }
    }
}