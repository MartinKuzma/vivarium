mod core;
mod mcp;
use rmcp::{ServiceExt, transport::stdio};
use std::sync::{Arc};

use crate::mcp::VivariumToolServer;

#[tokio::main]
async fn main() ->  Result<(), String>  {
    let world_registry = Arc::new(crate::core::registry::Registry::new());

    let service = VivariumToolServer::new(world_registry.clone()).serve(stdio()).await
        .map_err(|e| format!("Server error: {}", e))?;

    if let Err(e) = service.waiting().await {
        return Err(format!("Service error: {}", e));
    }

    Ok(())
}
