mod error;

use error::{SdkmanError, INTERNAL_ERROR_CODE, SDKMAN_NOT_INSTALLED_CODE};
use rmcp::{
    handler::server::tool::ToolRouter, model::*, tool, tool_handler, tool_router,
    transport::io::stdio, ErrorData as McpError, ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// SDKMAN! version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkmanVersion {
    pub script_version: String,
    pub native_version: String,
}

impl SdkmanVersion {
    /// Read version information from SDKMAN! filesystem
    pub fn read_from_filesystem() -> Result<Self, SdkmanError> {
        let home_dir = std::env::var("HOME")
            .map_err(|e| SdkmanError::Internal(format!("Failed to get HOME directory: {}", e)))?;

        let script_version_path = PathBuf::from(&home_dir).join(".sdkman/var/version");
        let native_version_path = PathBuf::from(&home_dir).join(".sdkman/var/version_native");

        debug!("Reading SDKMAN! version from filesystem");
        debug!("Script version path: {:?}", script_version_path);
        debug!("Native version path: {:?}", native_version_path);

        // Validate paths to prevent directory traversal
        if !script_version_path.starts_with(&home_dir)
            || !native_version_path.starts_with(&home_dir)
        {
            return Err(SdkmanError::Internal("Invalid path detected".to_string()));
        }

        // Check if both files exist
        if !script_version_path.exists() || !native_version_path.exists() {
            let checked_paths = vec![
                script_version_path.to_string_lossy().to_string(),
                native_version_path.to_string_lossy().to_string(),
            ];
            debug!("SDKMAN! not found. Checked paths: {:?}", checked_paths);
            return Err(SdkmanError::NotInstalled { checked_paths });
        }

        // Read version files
        let script_version = std::fs::read_to_string(&script_version_path)?
            .trim()
            .to_string();
        let native_version = std::fs::read_to_string(&native_version_path)?
            .trim()
            .to_string();

        debug!(
            "Read versions - Script: {}, Native: {}",
            script_version, native_version
        );

        Ok(SdkmanVersion {
            script_version,
            native_version,
        })
    }

    /// Format version information as text
    pub fn format(&self) -> String {
        format!(
            "SDKMAN! Versions: Script: {}; Native: {}",
            self.script_version, self.native_version
        )
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_format() {
        let version = SdkmanVersion {
            script_version: "5.18.2".to_string(),
            native_version: "0.4.6".to_string(),
        };

        assert_eq!(
            version.format(),
            "SDKMAN! Versions: Script: 5.18.2; Native: 0.4.6"
        );
    }
}
