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

impl SdkmanError {
    /// Convert to MCP error code
    pub fn error_code(&self) -> i32 {
        match self {
            SdkmanError::NotInstalled { .. } => SDKMAN_NOT_INSTALLED_CODE,
            SdkmanError::Internal(_) | SdkmanError::Io(_) => INTERNAL_ERROR_CODE,
        }
    }
}
