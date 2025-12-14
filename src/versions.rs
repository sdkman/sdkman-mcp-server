use crate::utils::error::SdkmanError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::debug;

/// SDKMAN! version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkmanVersion {
    pub script_version: String,
    pub native_version: String,
}

/// Get the SDKMAN! installation directory respecting SDKMAN_DIR environment variable
fn get_sdkman_dir() -> Result<PathBuf, SdkmanError> {
    if let Ok(sdkman_dir) = std::env::var("SDKMAN_DIR") {
        debug!("Using SDKMAN_DIR from environment: {}", sdkman_dir);
        Ok(PathBuf::from(sdkman_dir))
    } else {
        let home_dir = std::env::var("HOME")
            .map_err(|e| SdkmanError::Internal(format!("Failed to get HOME directory: {}", e)))?;
        let default_path = PathBuf::from(home_dir).join(".sdkman");
        debug!("Using default SDKMAN! directory: {:?}", default_path);
        Ok(default_path)
    }
}

impl SdkmanVersion {
    /// Read version information from SDKMAN! filesystem
    pub fn read_from_filesystem() -> Result<Self, SdkmanError> {
        let sdkman_dir = get_sdkman_dir()?;

        let script_version_path = sdkman_dir.join("var/version");
        let native_version_path = sdkman_dir.join("var/version_native");

        debug!("Reading SDKMAN! version from filesystem");
        debug!("SDKMAN! directory: {:?}", sdkman_dir);
        debug!("Script version path: {:?}", script_version_path);
        debug!("Native version path: {:?}", native_version_path);

        // Validate paths to prevent directory traversal
        if !script_version_path.starts_with(&sdkman_dir)
            || !native_version_path.starts_with(&sdkman_dir)
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
    use serial_test::serial;
    use std::env;

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

    #[test]
    #[serial]
    fn test_get_sdkman_dir_with_env_var() {
        // Set SDKMAN_DIR environment variable
        let custom_path = "/custom/sdkman/path";
        env::set_var("SDKMAN_DIR", custom_path);

        let result = get_sdkman_dir();

        // Clean up
        env::remove_var("SDKMAN_DIR");

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PathBuf::from(custom_path));
    }

    #[test]
    #[serial]
    fn test_get_sdkman_dir_default_fallback() {
        // Ensure SDKMAN_DIR is not set
        env::remove_var("SDKMAN_DIR");

        let result = get_sdkman_dir();

        assert!(result.is_ok());
        let path = result.unwrap();

        // Should end with .sdkman
        assert!(path.to_string_lossy().ends_with(".sdkman"));
    }
}
