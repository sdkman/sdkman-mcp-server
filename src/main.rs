mod error;
mod sdkman_version;

use error::{SdkmanError, INTERNAL_ERROR_CODE, SDKMAN_NOT_INSTALLED_CODE};
use rmcp::{
    handler::server::tool::ToolRouter, model::*, tool, tool_handler, tool_router,
    transport::io::stdio, ErrorData as McpError, ServiceExt,
};
use sdkman_version::SdkmanVersion;
use std::borrow::Cow;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[derive(Clone)]
pub struct SdkmanServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SdkmanServer {
    fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get SDKMAN! script and native version numbers")]
    async fn get_sdkman_version(&self) -> Result<CallToolResult, McpError> {
        info!("Tool call: get_sdkman_version");

        match SdkmanVersion::read_from_filesystem() {
            Ok(version) => {
                info!("Successfully retrieved SDKMAN! version");
                Ok(CallToolResult::success(vec![Content::text(
                    version.format(),
                )]))
            }
            Err(e) => {
                error!("Failed to get SDKMAN! version: {}", e);
                match e {
                    SdkmanError::NotInstalled { checked_paths } => {
                        let error_data = serde_json::json!({
                            "checked_paths": checked_paths
                        });
                        Err(McpError {
                            code: ErrorCode(SDKMAN_NOT_INSTALLED_CODE),
                            message: Cow::Borrowed("SDKMAN! not installed"),
                            data: Some(error_data),
                        })
                    }
                    _ => Err(McpError {
                        code: ErrorCode(INTERNAL_ERROR_CODE),
                        message: Cow::Owned(format!("Internal error: {}", e)),
                        data: None,
                    }),
                }
            }
        }
    }
}

#[tool_handler]
impl rmcp::ServerHandler for SdkmanServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to stderr (so it doesn't interfere with stdio transport)
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    info!("Starting SDKMAN! MCP Server v0.0.1");

    // Create and run the server with stdio transport
    let service = SdkmanServer::new().serve(stdio()).await.inspect_err(|e| {
        error!("Error starting server: {}", e);
    })?;

    info!("Server initialized and running");

    // Wait for the service to complete
    service.waiting().await?;

    info!("Server shutdown complete");

    Ok(())
}
