use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::cell::RefCell;


// Component trait defining behavior for components attached to entities
pub trait Component : Any {
    fn update(&mut self, current_step: u32);
    fn entity_id(&self) -> u32;

    fn as_any(&self) -> &(dyn Any + 'static);
    
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static);
}
