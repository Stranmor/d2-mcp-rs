use d2_mcp::D2McpServer;
use rmcp::{ServiceExt, transport::stdio};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    D2McpServer.serve(stdio()).await?.waiting().await?;
    Ok(())
}
