use thiserror::Error;

/// Custom error codes for SDKMAN! MCP server
pub const SDKMAN_NOT_INSTALLED_CODE: i32 = -40001;
pub const INTERNAL_ERROR_CODE: i32 = -32603;
pub const UNSUPPORTED_PLATFORM_CODE: i32 = -1000;
pub const BASH_NOT_AVAILABLE_CODE: i32 = -1001;
pub const NETWORK_ERROR_CODE: i32 = -1002;
pub const PERMISSION_ERROR_CODE: i32 = -1003;

/// Errors that can occur in the SDKMAN! MCP server
#[derive(Error, Debug)]
pub enum SdkmanError {
    #[error("SDKMAN! not installed")]
    NotInstalled { checked_paths: Vec<String> },

    #[error("Unsupported platform: {details}")]
    UnsupportedPlatform { details: String, recovery: String },

    #[error("Bash shell not available: {details}")]
    BashNotAvailable { details: String, recovery: String },

    #[error("Failed to download SDKMAN! installer: {details}")]
    NetworkError { details: String, recovery: String },

    #[error("Permission denied: {details}")]
    PermissionError { details: String, recovery: String },

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
