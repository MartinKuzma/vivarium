use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    EntityNotFound { id: String },
    EntityCreation { id: String, message: String },

    ScriptExecution { message: String },
    ScriptState { message: String },

    WorldAlreadyExists,
    WorldNotFound { name: String },

    DeserializationError(String),
    SnapshotError(String),

}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::EntityNotFound { id } => write!(f, "Entity with ID '{}' not found", id),
            CoreError::EntityCreation { id, message } => {
                write!(f, "Failed to create entity '{}': {}", id, message)
            }
            CoreError::ScriptExecution { message } => write!(f, "Script execution error: {}", message),
            CoreError::ScriptState { message } => write!(f, "Script state error: {}", message),                
            CoreError::DeserializationError(message) => write!(f, "Deserialization error: {}", message),
            CoreError::SnapshotError(message) => write!(f, "Snapshot error: {}", message),
            CoreError::WorldAlreadyExists => write!(f, "World already exists"),
            CoreError::WorldNotFound { name } => write!(f, "World '{}' not found", name),
        }
    }
}

impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
