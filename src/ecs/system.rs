use super::world::WorldContext;

// System trait defining behavior for systems that operate on entities
// System is responsible for updating specific aspects of entities based on their components
pub trait System {
    fn update(&mut self, current_step: u32, ctx: &mut WorldContext);    
}
