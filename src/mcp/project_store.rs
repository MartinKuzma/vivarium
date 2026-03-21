use std::collections::HashMap;
use crate::core::World;
use std::sync::{RwLock, Arc};
use crate::core::errors::CoreError;
use crate::core::persistence::project::ProjectContext;

struct LoadedProject {
    world: Arc<RwLock<World>>,
    project_context: ProjectContext,
}

// Registry for managing multiple simulations.
pub struct ProjectStore {
    projects: RwLock<HashMap<String, LoadedProject>>,
}

impl ProjectStore {
    pub fn new() -> Self {
        ProjectStore {
            projects: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_project(
        &self,
        name: String,
        world: World,
        project_context: ProjectContext,
        replace_if_exists: bool,
    ) -> Result<(), CoreError> {
        let mut projects = self.projects.write().unwrap();

        if projects.contains_key(&name) && !replace_if_exists {
            return Err(CoreError::WorldAlreadyExists);
        }

        projects.insert(
            name,
            LoadedProject {
                world: Arc::new(RwLock::new(world)),
                project_context,
            },
        );
        Ok(())
    }

    pub fn replace_world(&self, name: &str, world: World) -> Result<(), CoreError> {
        let mut projects = self.projects.write().unwrap();
        let Some(loaded_project) = projects.get_mut(name) else {
            return Err(CoreError::WorldNotFound {
                name: name.to_string(),
            });
        };

        loaded_project.world = Arc::new(RwLock::new(world));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Arc<RwLock<World>>, CoreError> {
        match self.projects.read().unwrap().get(name) {
            Some(loaded_project) => Ok(loaded_project.world.clone()),
            None => Err(CoreError::WorldNotFound { name: name.to_string() }),
        }
    }

    pub fn delete(&self, name: &str) -> Result<(), CoreError> {
        if self.projects.write().unwrap().remove(name).is_none() {
            return Err(CoreError::WorldNotFound { name: name.to_string() });
        }
        Ok(())
    }

    pub fn get_project_context(&self, name: &str) -> Result<ProjectContext, CoreError> {
        self.projects
            .read()
            .unwrap()
            .get(name)
            .map(|loaded_project| loaded_project.project_context.clone())
            .ok_or_else(|| CoreError::WorldNotFound {
                name: name.to_string(),
            })
    }

    pub fn list(&self) -> Vec<String> {
        self.projects.read().unwrap().keys().cloned().collect()
    }
}
