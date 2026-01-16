use std::{cell::RefCell, rc::Rc, time};

// use std::{cell::RefCell, rc::Rc};
// use ecs::Component;

use mlua::prelude::*;

mod simulator;
use simulator::World;

fn main() ->  Result<(), String>  {
    let mut world = simulator::World::new();

    let agent_script = String::from(r#"
        function update()
            print("Agent updating: " .. entity.id)
            entity.send_msg(1, "Greeting", "Hello from Agent 1")
        end

        function on_message(msg)
            print("Agent received message: " .. msg.content)
        end
    "#);

    world.create_entity(1, "Agent_1", agent_script)
        .map_err(|e| format!("Failed to create entity: {}", e))?;

    for _step in 0..5 {
        world.update_entities()?;
        world.process_commands();
        world.update_simulation_time(time::Duration::from_secs(1));
    }


    
    Ok(())
}
