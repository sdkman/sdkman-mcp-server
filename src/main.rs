mod error;
mod installation;
mod versions;

use error::{
    SdkmanError, INTERNAL_ERROR_CODE, NETWORK_ERROR_CODE, PERMISSION_ERROR_CODE,
    SDKMAN_NOT_INSTALLED_CODE, UNSUPPORTED_PLATFORM_CODE,
};
use installation::SdkmanInstallation;
use rmcp::{
    handler::server::tool::ToolRouter, model::*, tool, tool_handler, tool_router,
    transport::io::stdio, ErrorData as McpError, ServiceExt,
};
use versions::SdkmanVersion;
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

    #[tool(description = "Downloads and executes the official SDKMAN! installer script, creating the complete SDKMAN! environment with automatic shell configuration")]
    async fn install_sdkman(&self) -> Result<CallToolResult, McpError> {
        info!("Tool call: install_sdkman");

        // Default to updating RC files
        let update_rc_files = true;

        match SdkmanInstallation::install(update_rc_files, None).await {
            Ok(result) => {
                info!("SDKMAN! installation successful");
                let json_result = serde_json::to_string_pretty(&result).map_err(|e| McpError {
                    code: ErrorCode(INTERNAL_ERROR_CODE),
                    message: Cow::Owned(format!("Failed to serialize result: {}", e)),
                    data: None,
                })?;

                Ok(CallToolResult::success(vec![Content::text(json_result)]))
            }
            Err(e) => {
                error!("SDKMAN! installation failed: {}", e);
                match e {
                    SdkmanError::UnsupportedPlatform { details, recovery } => {
                        let error_data = serde_json::json!({
                            "details": details,
                            "recovery": recovery
                        });
                        Err(McpError {
                            code: ErrorCode(UNSUPPORTED_PLATFORM_CODE),
                            message: Cow::Borrowed("SDKMAN! installation not supported on native Windows"),
                            data: Some(error_data),
                        })
                    }
                    SdkmanError::NetworkError { details, recovery } => {
                        let error_data = serde_json::json!({
                            "details": details,
                            "recovery": recovery
                        });
                        Err(McpError {
                            code: ErrorCode(NETWORK_ERROR_CODE),
                            message: Cow::Borrowed("Failed to download SDKMAN! installer"),
                            data: Some(error_data),
                        })
                    }
                    SdkmanError::PermissionError { details, recovery } => {
                        let error_data = serde_json::json!({
                            "details": details,
                            "recovery": recovery
                        });
                        Err(McpError {
                            code: ErrorCode(PERMISSION_ERROR_CODE),
                            message: Cow::Borrowed("Permission denied"),
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
