use std::{cell::RefCell, rc::Rc, time};

mod simulator;

fn main() ->  Result<(), String>  {
    let mut world = simulator::World::new();

    let agent_script = String::from(r#"
        function update(msgs)
            for _, msg in ipairs(msgs) do
                print("Agent received message: " .. msg.content)
            end
            print("Agent updating: " .. entity.id)
            entity.send_msg("Agent_2", "Greeting", "Hello from Agent 1", 5)
        end
    "#);

    let agent_script2 = String::from(r#"
        function update(msgs)
            for _, msg in ipairs(msgs) do
                print("Agent received message: " .. msg.content)
            end

            for _, entity_id in ipairs(entity.list_entities()) do
                print("Known entity: " .. entity_id)
            end
            print("Agent updating: " .. entity.id)
            entity.send_msg("Agent_1", "Greeting", "Hello from Agent 2", 0)

        end
    "#);

    world.create_entity("Agent_1".to_string(), agent_script)
        .map_err(|e| format!("Failed to create entity: {}", e))?;

    world.create_entity("Agent_2".to_string(), agent_script2)
        .map_err(|e| format!("Failed to create entity: {}", e))?;

    for _step in 0..15 {
        world.update(time::Duration::from_secs(1))?;
    }
    
    Ok(())
}
