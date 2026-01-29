mod core;
mod mcp;
use rmcp::{ServiceExt, transport::stdio};
use crate::mcp::VivariumToolServer;

#[tokio::main]
async fn main() ->  Result<(), String>  {
    let world_registry = crate::core::registry::Registry::new();

    let service = VivariumToolServer::new(world_registry).serve(stdio()).await
        .map_err(|e| format!("Server error: {}", e))?;

    if let Err(e) = service.waiting().await {
        return Err(format!("Service error: {}", e));
    }

    Ok(())
}
