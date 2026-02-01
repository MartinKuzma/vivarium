use std::fmt::Debug;

use crate::core::errors::CoreError;
use rmcp::model::ErrorData as McpError;


impl From<CoreError> for McpError {
    fn from(err: CoreError) -> Self {
        let code = match err {
            CoreError::WorldNotFound { .. } => rmcp::model::ErrorCode::INVALID_PARAMS,
            CoreError::EntityNotFound {.. } => rmcp::model::ErrorCode::INVALID_PARAMS,
            CoreError::WorldAlreadyExists {.. } => rmcp::model::ErrorCode::INVALID_PARAMS,
            _ => rmcp::model::ErrorCode::INTERNAL_ERROR,
        };

        McpError::new(
            code,
            format!("Core error: {}", err),
            None,
        )
    }
}