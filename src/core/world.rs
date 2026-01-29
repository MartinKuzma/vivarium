use crate::core::Entity;
use crate::core::errors::CoreError;
use crate::core::messaging::{JSONObject, Message, MessageBus};
use crate::core::metrics::Metrics;
use crate::core::world_config::WorldCfg;
use std::rc::Rc;

use std::{cell::RefCell, collections::HashMap};

// Represents a simulation world containing entities, message bus, and metrics.
pub struct World {
    cfg: WorldCfg,
    msg_bus: MessageBus,
    state: Rc<RefCell<WorldState>>,
    metrics: Metrics,
    simulation_time: u64, //TODO: Replace with some shared clock
}

pub struct WorldState {
    entities: HashMap<String, RefCell<Entity>>,
}

pub struct WorldUpdateResult {
    pub delivered_messages: Vec<Message>,
}

impl World {
    pub fn new(cfg: &WorldCfg) -> Result<Self, CoreError> {
        cfg.validate()?;

        let mut state = Rc::new(RefCell::new(WorldState {
            entities : HashMap::new(),
        }));

        
        for entity_cfg in &cfg.entities {
            let mut entity = Entity::new(
                entity_cfg.id.clone(),
                entity_cfg.script_id.clone(),
                cfg.script_library.get(&entity_cfg.script_id).unwrap().clone(),
                entity_cfg.initial_state.clone(),
                state.clone(),
            )
            .map_err(|e| CoreError::EntityCreation {
                id: entity_cfg.id.clone(),
                message: format!("Failed to create entity: {}", e),
            })?;

            if let Some(state) = &entity_cfg.initial_state {
                entity
                    .get_lua_controller_mut()
                    .set_state(state.clone())
                    .map_err(|e| CoreError::EntityCreation {
                        id: entity_cfg.id.clone(),
                        message: format!("Failed to set initial state for entity: {}", e),
                    })?;
            }

            state.borrow_mut().entities.insert(entity_cfg.id.clone(), RefCell::new(entity));
        }

        Ok(World {
            cfg: cfg.clone(),
            simulation_time: 0,
            msg_bus: MessageBus::new(),
            state: state,
            metrics: Metrics::new(),
        })
    }

    pub fn new_from_snapshot(snapshot: crate::core::snapshot::WorldSnapshot) -> Result<Self, CoreError> {
        let mut world = World::new(&snapshot.configuration)?;

        world.simulation_time = snapshot.simulation_time;
        world.metrics = Metrics::new_from_snapshot(&snapshot.metrics);
        
        for message in &snapshot.pending_messages {
            world.msg_bus.schedule_message(
                &message.sender,
                message.receiver.clone(),
                message.kind.clone(),
                message.content.clone(),
                message.receive_step,
            );
        }

        Ok(world)
    }

    pub fn remove_entity(&mut self, id: &str) -> Option<RefCell<Entity>> {
        self.get_state_mut().entities.remove(id)
    }

    pub fn fetch_messages(&mut self) -> Vec<Message> {
        let mut messages = Vec::new();
        while let Some(msg) = self.msg_bus.pop_deliverable_message(self.simulation_time) {
            messages.push(msg);
        }

        messages
    }

    pub fn update(&mut self, delta: u64) -> Result<WorldUpdateResult, CoreError> {
        let mut update_result = WorldUpdateResult::new();

        // Update simulation time
        self.update_simulation_time(self.simulation_time + delta);
        self.deliver_messages(&mut update_result);

        let mut commands = Vec::new();

        for entity in self.get_state_ref().entities.values() {
            let entity_commands = entity.borrow_mut().update(self.simulation_time)?;
            commands.extend(entity_commands);
        }

        self.process_commands(commands);

        Ok(update_result)
    }

    fn process_commands(&mut self, commands: Vec<crate::core::messaging::Command>) {
        for command in commands {
            match command {
                crate::core::messaging::Command::SendMessage {
                    sender,
                    receiver,
                    kind,
                    content,
                    delay,
                } => {
                    self.msg_bus.schedule_message(
                        &sender,
                        receiver,
                        kind,
                        content,
                        self.simulation_time + delay,
                    );
                }
                crate::core::messaging::Command::RemoveEntity { id } => {
                    self.remove_entity(&id);
                }
                crate::core::messaging::Command::RecordMetric { name, value } => {
                    self.metrics.record_metric(self.simulation_time, &name, value);
                }
            }
        }
    }

    fn deliver_messages(&mut self, update_result: &mut WorldUpdateResult) {
        let messages = self.fetch_messages();
        for msg in messages {
            // Log delivered message
            update_result.delivered_messages.push(msg.clone());

            match msg.receiver {
                crate::core::messaging::MessageReceiver::Entity { ref id, .. } => {
                    if let Some(entity) = self.get_state_ref().entities.get(id) {
                        entity.borrow_mut().receive_message(msg);
                    }
                }
                crate::core::messaging::MessageReceiver::Radius2D { x, y, radius } => {
                    // TODO: Implement radius-based message delivery
                }
            }
        }
    }

    pub fn set_entity_state(&mut self, id: &str, state: JSONObject) -> Result<(), CoreError> {
        if let Some(entity) = self.get_state_ref().entities.get(id) {
            entity.borrow_mut().get_lua_controller_mut().set_state(state)
        } else {
            Err(CoreError::EntityNotFound { id: id.to_string() })
        }
    }

    pub fn get_entity_state(&self, id: &str) -> Option<JSONObject> {
        self.get_state_ref().get_entity_state(id)
    }

    fn update_simulation_time(&mut self, new_time: u64) {
        self.simulation_time = new_time;
    }

    pub fn get_state_ref(&self) -> std::cell::Ref<'_, WorldState> {
        self.state.borrow()
    }

    fn get_state_mut(&self) -> std::cell::RefMut<'_, WorldState> {
        self.state.borrow_mut()
    }

    pub fn get_metrics_ref(&self) -> &Metrics {
        &self.metrics
    }

    pub fn create_snapshot(&self) -> Result<crate::core::snapshot::WorldSnapshot, CoreError> {
        let mut world_config = self.cfg.clone();

        for (id, entity_cell) in &self.get_state_ref().entities {
            let entity = entity_cell.borrow();
            let lua_controller = entity.get_lua_controller();
            let state = lua_controller.get_state()?;

            world_config.upsert_entity(id, entity.get_script_id(), Some(state))?;
        }

        let mut messages = Vec::new();
        for msg in self.msg_bus.get_pending_messages_iter() {
            messages.push(msg.clone());
        }

        Ok(crate::core::snapshot::WorldSnapshot::new(
            world_config,
            self.simulation_time,
            messages,
            self.metrics.create_snapshot(),
        ))
    }

    pub fn get_entities_count(&self) -> usize {
        self.get_state_ref().entities.len()
    }

    pub fn get_simulation_time(&self) -> u64 {
        self.simulation_time   
    }

    pub fn get_pending_messages_count(&self) -> usize {
        self.msg_bus.get_pending_messages_count()
    }
}

impl WorldState {
    pub fn get_entities(&self) -> &HashMap<String, RefCell<Entity>> {
        &self.entities
    }

    pub fn filter_entities<F>(&self, filter_fn: F) -> Vec<String>
    where
        F: Fn(&(&std::string::String, &RefCell<Entity>)) -> bool,
    {
        self.entities
            .iter()
            .filter(filter_fn)
            .map(|(id, _)| id.clone())
            .collect()
    }


    pub fn get_entity_state(&self, id: &str) -> Option<JSONObject> {
        if let Some(entity) = self.entities.get(id) {
            match entity.borrow().get_lua_controller().get_state() {
                Ok(state) => Some(state),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

impl WorldUpdateResult {
    pub fn new() -> Self {
        WorldUpdateResult {
            delivered_messages: Vec::new(),
        }
    }
}
