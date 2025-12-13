use crate::error::SdkmanError;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};

const DEFAULT_INSTALLER_URL: &str = "https://get.sdkman.io";
const INSTALLER_TIMEOUT_SECS: u64 = 60;
const DOWNLOAD_TIMEOUT_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

/// SDKMAN! version information
#[derive(Debug, Clone)]
pub struct SdkmanVersions {
    pub script_version: String,
    pub native_version: Option<String>,
}

impl SdkmanVersions {
    /// Format versions for display
    pub fn display(&self) -> String {
        match &self.native_version {
            Some(native) => format!("script: {}, native: {}", self.script_version, native),
            None => format!("script: {}", self.script_version),
        }
    }
}

/// Installation state representation
#[derive(Debug, Clone)]
pub struct SdkmanInstallation {
    pub dir: PathBuf,
    pub is_installed: bool,
    pub versions: Option<SdkmanVersions>,
}

/// Result of installation operation
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationResult {
    pub installed: bool,
    pub sdkman_dir: String,
    pub message: String,
    pub shell_restart_required: bool,
    //TODO: this struct should contain a field that
    //      tracks if the rc files were updated
    //      eg. bashrc, zshrc
}

impl SdkmanInstallation {
    /// Detect existing SDKMAN! installation
    pub async fn detect() -> Result<Self, SdkmanError> {
        let dir = get_sdkman_dir();
        let init_script = dir.join("bin/sdkman-init.sh");
        let is_installed = init_script.exists();

        let versions = if is_installed {
            read_version_from_metadata(&dir).await.ok()
        } else {
            None
        };

        Ok(Self {
            dir,
            is_installed,
            versions,
        })
    }

    /// Install SDKMAN! using official installer
    pub async fn install(
        update_rc_files: bool,
        installer_url: Option<String>,
    ) -> Result<InstallationResult, SdkmanError> {
        info!("Starting SDKMAN! installation process");

        // 1. Detect platform and reject native Windows (CMD/PowerShell)
        check_platform_compatibility().await?;

        // 2. Check if already installed
        let installation = Self::detect().await?;
        if installation.is_installed {
            let version_str = installation
                .versions
                .as_ref()
                .map(|v| v.display())
                .unwrap_or_else(|| "unknown".to_string());
            info!("SDKMAN! already installed at {:?}", installation.dir);
            return Ok(InstallationResult {
                installed: true,
                sdkman_dir: installation.dir.display().to_string(),
                message: format!(
                    "SDKMAN! is already installed at {} (version: {})",
                    installation.dir.display(),
                    version_str
                ),
                shell_restart_required: false,
            });
        }

        // 3. Check if RC files are writable
        let rc_files_readonly = check_rc_files_readonly().await;
        let should_update_rc = update_rc_files && !rc_files_readonly;

        if rc_files_readonly && update_rc_files {
            warn!("Shell RC files are read-only, will skip RC file updates");
        }

        // 4. Download installer from https://get.sdkman.io
        let install_url = installer_url.unwrap_or_else(|| DEFAULT_INSTALLER_URL.to_string());
        let installer_script = download_installer(&install_url).await?;

        // 5. Execute with appropriate flags
        execute_installer(&installer_script, should_update_rc).await?;

        // 6. Verify installation succeeded
        verify_installation(&installation.dir).await?;

        // 7. Return result with paths and instructions
        let message = if should_update_rc {
            let shell = detect_shell();
            format!(
                "SDKMAN! installed successfully at {}. Shell configuration updated for {}. Please restart your terminal or run: source {}/bin/sdkman-init.sh",
                installation.dir.display(),
                shell,
                installation.dir.display()
            )
        } else if rc_files_readonly {
            format!(
                "SDKMAN! installed successfully at {}. Shell RC files are read-only (e.g., NixOS). Add this to your shell profile manually: source {}/bin/sdkman-init.sh",
                installation.dir.display(),
                installation.dir.display()
            )
        } else {
            format!(
                "SDKMAN! installed successfully at {}. Shell RC files were not modified. Add this to your shell profile manually: source {}/bin/sdkman-init.sh",
                installation.dir.display(),
                installation.dir.display()
            )
        };

        info!("SDKMAN! installation completed successfully");

        Ok(InstallationResult {
            installed: true,
            sdkman_dir: installation.dir.display().to_string(),
            message,
            shell_restart_required: should_update_rc,
        })
    }
}

//TODO: consider moving this to a common `lib`` module
//      as this is reusable code
/// Get SDKMAN! directory from environment or default
pub fn get_sdkman_dir() -> PathBuf {
    if let Ok(sdkman_dir) = env::var("SDKMAN_DIR") {
        // Validate to prevent path traversal
        let path = PathBuf::from(&sdkman_dir);
        if path.is_absolute() && !sdkman_dir.contains("..") {
            return path;
        }
        warn!("Invalid SDKMAN_DIR environment variable, using default");
    }

    // Default to ~/.sdkman
    dirs::home_dir()
        .expect("Unable to determine home directory")
        .join(".sdkman")
}

/// Read version from SDKMAN! metadata
async fn read_version_from_metadata(dir: &Path) -> Result<SdkmanVersions, SdkmanError> {
    let script_version_file = dir.join("var/version");
    let native_version_file = dir.join("var/version_native");

    // Script version is required
    let script_version = if script_version_file.exists() {
        fs::read_to_string(&script_version_file)
            .await
            .map(|content| content.trim().to_string())
            .map_err(|e| {
                SdkmanError::Internal(format!("Failed to read script version file: {}", e))
            })?
    } else {
        return Err(SdkmanError::Internal(
            "Script version file not found".to_string(),
        ));
    };

    // Native version is optional
    let native_version = if native_version_file.exists() {
        match fs::read_to_string(&native_version_file).await {
            Ok(content) => Some(content.trim().to_string()),
            Err(e) => {
                debug!("Failed to read native version file: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok(SdkmanVersions {
        script_version,
        native_version,
    })
}

//TODO: consider moving this to a common platform helper in the lib module
/// Check platform compatibility - reject native Windows
async fn check_platform_compatibility() -> Result<(), SdkmanError> {
    // Check if we're on Windows OS
    if cfg!(target_os = "windows") {
        // Check if we have bash available (Git Bash or WSL)
        let has_bash = Command::new("bash")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .is_ok();

        if !has_bash {
            return Err(SdkmanError::UnsupportedPlatform {
                details: "SDKMAN! requires a Unix-like environment with bash shell".to_string(),
                recovery: "Please use one of these supported environments:\n• Windows Subsystem for Linux (WSL) - Recommended: https://docs.microsoft.com/en-us/windows/wsl/install\n• Git Bash for Windows: https://gitforwindows.org/\n\nAlternatively, use SDKMAN! on Linux or macOS.".to_string(),
            });
        }
    }

    Ok(())
}

/// Check if shell RC files are read-only (e.g., NixOS)
async fn check_rc_files_readonly() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    // Check common RC files
    let rc_files = vec![".bashrc", ".zshrc", ".bash_profile", ".profile"];

    for rc_file in rc_files {
        let path = home.join(rc_file);
        if path.exists() {
            // Try to check if file is writable
            match fs::metadata(&path).await {
                Ok(metadata) => {
                    if metadata.permissions().readonly() {
                        debug!("RC file {} is read-only", path.display());
                        return true;
                    }
                }
                Err(_) => continue,
            }
        }
    }

    false
}

//TODO: move to shell ops helper in lib module
/// Detect current shell
fn detect_shell() -> String {
    env::var("SHELL")
        .ok()
        .and_then(|s| {
            PathBuf::from(&s)
                .file_name()
                .and_then(|n| n.to_str())
                .map(String::from)
        })
        .unwrap_or_else(|| "bash".to_string())
}

/// Download installer script with retry logic
async fn download_installer(url: &str) -> Result<String, SdkmanError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .map_err(|e| SdkmanError::Internal(format!("Failed to create HTTP client: {}", e)))?;

    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        debug!("Download attempt {} of {}", attempt, MAX_RETRIES);

        match client.get(url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(script) => {
                            info!("Successfully downloaded installer script");
                            return Ok(script);
                        }
                        Err(e) => {
                            last_error = Some(format!("Failed to read response body: {}", e));
                        }
                    }
                } else {
                    last_error = Some(format!("HTTP error: {}", response.status()));
                }
            }
            Err(e) => {
                last_error = Some(e.to_string());
            }
        }

        if attempt < MAX_RETRIES {
            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * attempt as u64)).await;
        }
    }

    let details = last_error.unwrap_or_else(|| "Unknown error".to_string());
    Err(SdkmanError::NetworkError {
        details: format!("Connection failed after {} attempts: {}", MAX_RETRIES, details),
        recovery: "Check your internet connection and try again. Visit https://sdkman.io/install for manual installation or check service status at https://sdkman.io/status".to_string(),
    })
}

/// Execute installer script
async fn execute_installer(script: &str, update_rc_files: bool) -> Result<(), SdkmanError> {
    info!("Executing SDKMAN! installer script");

    // Prepare environment variables
    let mut cmd = Command::new("bash");
    cmd.arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Set SDKMAN_DIR if specified
    if let Ok(sdkman_dir) = env::var("SDKMAN_DIR") {
        cmd.env("SDKMAN_DIR", sdkman_dir);
    }

    // Control RC file updates
    if !update_rc_files {
        cmd.env("SDKMAN_SKIP_RC", "true");
    }

    let mut child = cmd.spawn().map_err(|e| SdkmanError::PermissionError {
        details: format!("Failed to spawn bash process: {}", e),
        recovery: "Ensure bash is installed and you have permission to execute it.".to_string(),
    })?;

    // Write script to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(script.as_bytes()).await.map_err(|e| {
            SdkmanError::Internal(format!("Failed to write installer script to stdin: {}", e))
        })?;
    }

    // Wait for process with timeout
    let output = tokio::time::timeout(
        Duration::from_secs(INSTALLER_TIMEOUT_SECS),
        child.wait_with_output(),
    )
    .await
    .map_err(|_| {
        SdkmanError::Internal("Installer execution timed out after 60 seconds".to_string())
    })?
    .map_err(|e| SdkmanError::Internal(format!("Failed to execute installer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for permission errors
        if stderr.contains("Permission denied") || stdout.contains("Permission denied") {
            return Err(SdkmanError::PermissionError {
                details: "Insufficient permissions to install SDKMAN!".to_string(),
                recovery:
                    "Ensure you have write permissions to your home directory. Do not use sudo."
                        .to_string(),
            });
        }

        return Err(SdkmanError::Internal(format!(
            "Installer failed with exit code {:?}\nStdout: {}\nStderr: {}",
            output.status.code(),
            stdout,
            stderr
        )));
    }

    debug!("Installer script executed successfully");
    Ok(())
}

/// Verify installation completed successfully
async fn verify_installation(dir: &Path) -> Result<(), SdkmanError> {
    info!("Verifying SDKMAN! installation");

    let init_script = dir.join("bin/sdkman-init.sh");
    let candidates_dir = dir.join("candidates");
    let version_file = dir.join("var/version");

    if !init_script.exists() {
        return Err(SdkmanError::Internal(
            "Installation verification failed: sdkman-init.sh not found".to_string(),
        ));
    }

    if !candidates_dir.exists() {
        return Err(SdkmanError::Internal(
            "Installation verification failed: candidates directory not found".to_string(),
        ));
    }

    if !version_file.exists() {
        return Err(SdkmanError::Internal(
            "Installation verification failed: version file not found".to_string(),
        ));
    }

    info!("Installation verification successful");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    //TODO: move with associated sdkman dir helper
    #[test]
    fn test_get_sdkman_dir_default() {
        env::remove_var("SDKMAN_DIR");
        let dir = get_sdkman_dir();
        assert!(dir.ends_with(".sdkman"));
    }

    //TODO: move with associated sdkman dir helper
    #[test]
    fn test_get_sdkman_dir_from_env() {
        let test_dir = "/tmp/test-sdkman";
        env::set_var("SDKMAN_DIR", test_dir);
        let dir = get_sdkman_dir();
        assert_eq!(dir.display().to_string(), test_dir);
        env::remove_var("SDKMAN_DIR");
    }

    //TODO: move with associated sdkman dir helper
    #[test]
    fn test_get_sdkman_dir_rejects_traversal() {
        env::set_var("SDKMAN_DIR", "/tmp/../etc/passwd");
        let dir = get_sdkman_dir();
        assert!(dir.ends_with(".sdkman")); // Should fall back to default
        env::remove_var("SDKMAN_DIR");
    }

    //TODO: move alongside associated shell helper
    #[test]
    fn test_detect_shell() {
        let shell = detect_shell();
        assert!(!shell.is_empty());
    }
}

