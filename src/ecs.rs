use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

trait Component {
    fn update(&mut self, current_step: u32);
    fn get_component_type(&self) -> &str;
    fn entity_id(&self) -> u32;
}


// System trait defining behavior for systems that operate on entities
// System is responsible for updating specific aspects of entities based on their components
trait System {
    fn update(&mut self, current_step: u32, world: &mut WorldContext);
    fn get_system_type(&self) -> &str;
}

struct World {
    systems: Vec<Box<dyn System>>, // Systems managing entity behavior
    components : HashMap<String, HashMap<u32, Box<dyn Component>>>, // Components categorized by type and entity ID
}

pub enum Command {
    RemoveEntity(u32),
    AddComponent(u32, Box<dyn Component>),
    RemoveComponent(u32, String), // Component type
}

pub struct WorldContext<'a> {
    commands: Vec<Command>,
    components: RefCell<&'a HashMap<String, HashMap<u32, Box<dyn Component>>>>,
}

impl WorldContext<'_> {
    pub fn get_components_of_type(&self, component_type: &str) -> Option<&HashMap<u32, Box<dyn Component>>> {
        self.components.borrow().get(component_type)
    }

    pub fn remove_entity(&mut self, entity_id: u32) {
        self.commands.push(Command::RemoveEntity(entity_id));
    }

    pub fn add_component(&mut self, entity_id: u32, component: Box<dyn Component>) {
        self.commands.push(Command::AddComponent(entity_id, component));
    }

    pub fn remove_component(&mut self, entity_id: u32, component_type: &str) {
        self.commands.push(Command::RemoveComponent(entity_id, component_type.to_string()));
    }
}

impl World {
    pub fn new() -> Self {
        World {
            systems: Vec::new(),
            components: HashMap::new(),
        }
    }

    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    pub fn update(&mut self, current_step: u32) {
        let mut wctx = WorldContext {
            commands: Vec::new(),
            components: RefCell::new(&self.components),
        };

        for system in self.systems.iter_mut() {
            system.update(current_step, &mut wctx);
        }

        //TODO: Process commands
    }
}