use crate::error::SdkmanError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

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
