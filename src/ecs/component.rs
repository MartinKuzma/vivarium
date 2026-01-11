use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::cell::RefCell;


// Component trait defining behavior for components attached to entities
pub trait Component {
    fn update(&mut self, current_step: u32);
    fn entity_id(&self) -> u32;
}
