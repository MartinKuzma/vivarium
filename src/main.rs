mod simulation;
mod agent;
mod message_bus;
mod ecs;

use crate::agent::LuaAgent;

fn main() {

    let mut sim = simulation::SimulationEngine::new();
    sim.add_agent(Box::new(LuaAgent::new(1, "Agent1".to_string())));
    sim.advance_for(10);

    println!("Hello, world!");
}
