use thiserror::Error;

/// Custom error codes for SDKMAN! MCP server
pub const SDKMAN_NOT_INSTALLED_CODE: i32 = -40001;
pub const INTERNAL_ERROR_CODE: i32 = -32603;

/// Errors that can occur in the SDKMAN! MCP server
#[derive(Error, Debug)]
pub enum SdkmanError {
    #[error("SDKMAN! not installed")]
    NotInstalled { checked_paths: Vec<String> },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
