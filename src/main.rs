mod core;
mod mcp;
use rmcp::{ServiceExt, transport::stdio};

use crate::mcp::SimulationToolServer;

#[tokio::main]
async fn main() ->  Result<(), String>  {
    let service = SimulationToolServer::new().serve(stdio()).await
        .map_err(|e| format!("Server error: {}", e))?;

    if let Err(e) = service.waiting().await {
        return Err(format!("Service error: {}", e));
    }

    Ok(())
}
