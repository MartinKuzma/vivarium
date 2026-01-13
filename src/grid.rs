use crate::ecs::{Component, System, WorldContext};
use std::cell::RefCell;

pub struct GridSystem {
    components: Vec<GridComponent>,    
}

pub struct GridComponent {
    pub pos_x: i32,
    pub pos_y: i32,
}

impl GridComponent {
    pub fn new(pos_x: i32, pos_y: i32) -> Self {
        GridComponent { pos_x, pos_y }
    }
}

impl Component for GridComponent {
    fn update(&mut self, current_step: u32) {
        println!(
            "GridComponent at position ({}, {}) updating at step {}",
            self.pos_x, self.pos_y, current_step
        );
    }

    fn entity_id(&self) -> u32 {
        0 // Placeholder, should return actual entity ID
    }

    fn as_any(&self) -> &(dyn std::any::Any + 'static) {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut (dyn std::any::Any + 'static) {
        self
    }
}

impl System for GridSystem {
    fn update(&mut self, current_step: u32, ctx: &mut WorldContext) {
        // Implement the logic to update the grid system
        println!("Updating GridSystem at step {}", current_step);
    }
}

impl GridSystem {
    pub fn new() -> Self {
        GridSystem {
            components: Vec::new(),
        }
    }

}