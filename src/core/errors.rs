use std::fmt;

#[derive(Debug)]
pub enum CoreError {
    EntityNotFound{id: String},
    EntityCreation{id: String, message: String},

    ScriptExecution{message: String},
    ScriptState{message: String},

    SerializationError(String),
    DeserializationError(String),
    SnapshotError(String),
 }

 impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::EntityNotFound { id } => write!(f, "Entity with ID '{}' not found", id),
            CoreError::EntityCreation { id, message } => write!(f, "Failed to create entity '{}': {}", id, message),
            CoreError::ScriptExecution { message } => write!(f, "Script execution error: {}", message),
            CoreError::ScriptState { message } => write!(f, "Script state error: {}", message),
            CoreError::SerializationError(message) => write!(f, "Serialization error: {}", message),
            CoreError::DeserializationError(message) => write!(f, "Deserialization error: {}", message),
            CoreError::SnapshotError(message) => write!(f, "Snapshot error: {}", message),
        }
    }
 }

 impl std::error::Error for CoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
 }