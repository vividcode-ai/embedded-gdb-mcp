use tracing::{info, error};
use tracing_subscriber::{EnvFilter, fmt};
use rmcp::{ServiceExt, transport::stdio};

use embedded_gdb_mcp::EmbeddedGdbToolHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    info!("Starting GDB MCP Server");

    let service = EmbeddedGdbToolHandler::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            error!("Serving error: {:?}", e);
        })?;

    info!("GDB MCP server running on stdio");

    // Handle shutdown gracefully
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        info!("Shutting down GDB MCP server...");
        // Sessions will be cleaned up on drop
        std::process::exit(0);
    });

    service.waiting().await?;

    info!("GDB MCP server stopped");
    Ok(())
}
