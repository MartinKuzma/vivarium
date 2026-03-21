use rmcp::{ServiceExt, transport::stdio};
use vivarium::mcp::{ProjectStore, VivariumToolServer};

#[tokio::main]
async fn main() ->  Result<(), String>  {
    let world_registry = ProjectStore::new();

    let tool_server =  VivariumToolServer::new(world_registry);
    let service = tool_server.serve(stdio()).await
        .map_err(|e| format!("Server error: {}", e))?;
    
    if let Err(e) = service.waiting().await {
        return Err(format!("Service error: {}", e));
    }

    Ok(())
}
