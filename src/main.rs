use std::{cell::RefCell, rc::Rc, time};

// use std::{cell::RefCell, rc::Rc};
// use ecs::Component;

use mlua::prelude::*;

mod simulator;
use simulator::World;

fn main() ->  Result<(), String>  {
    let world = Rc::new(RefCell::new(simulator::World::new()));

    let agent_script = String::from(r#"
        function update()
            print("Agent updating...")
            entity.send_msg(1, "Greeting", "Hello from Agent 1")
        end

        function on_message(msg)
            print("Agent received message: " .. msg.content)
        end
    "#);

    let entity = simulator::Entity::new(1, "Agent_1", agent_script, world.clone())
        .map_err(|e| format!("Failed to create entity: {}", e))?;

    world.borrow_mut().add_entity(entity);

    world.borrow().update_entities()?;
    world.borrow_mut().process_commands();
    world.borrow_mut().update_simulation_time(time::Duration::from_secs(1));


    
    Ok(())
}
