use crate::utils::error::SdkmanError;
use crate::utils::fs_helpers::get_sdkman_dir;
use crate::utils::platform::{check_bash_available, check_platform_compatibility};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tracing::{debug, info};

const DEFAULT_INSTALLER_URL: &str = "https://get.sdkman.io";
const INSTALLER_TIMEOUT_SECS: u64 = 60;
const DOWNLOAD_TIMEOUT_SECS: u64 = 30;
const MAX_RETRIES: u32 = 3;
const RETRY_DELAY_MS: u64 = 1000;

/// Installation state representation
#[derive(Debug, Clone)]
pub struct SdkmanInstallation {
    pub dir: PathBuf,
    pub is_installed: bool,
}

/// Result of installation operation
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallationResult {
    pub installed: bool,
    pub sdkman_dir: String,
    pub message: String,
    pub shell_restart_required: bool,
}

impl SdkmanInstallation {
    /// Detect existing SDKMAN! installation
    pub async fn detect() -> Result<Self, SdkmanError> {
        let dir = get_sdkman_dir();
        let init_script = dir.join("bin/sdkman-init.sh");
        let is_installed = init_script.exists();

        Ok(Self { dir, is_installed })
    }

    /// Install SDKMAN! using official installer
    pub async fn install(
        update_rc_files: bool,
        installer_url: Option<String>,
    ) -> Result<InstallationResult, SdkmanError> {
        info!("Starting SDKMAN! installation process");

        // 1. Check platform compatibility (validate platform triple)
        check_platform_compatibility().await?;

        // 2. Check bash availability
        check_bash_available().await?;

        // 3. Check if already installed
        let installation = Self::detect().await?;
        if installation.is_installed {
            info!("SDKMAN! already installed at {:?}", installation.dir);
            return Ok(InstallationResult {
                installed: true,
                sdkman_dir: installation.dir.display().to_string(),
                message: format!("SDKMAN! is already installed at {}", installation.dir.display()),
                shell_restart_required: false,
            });
        }

        // 4. Download installer from https://get.sdkman.io
        let install_url = installer_url.unwrap_or_else(|| DEFAULT_INSTALLER_URL.to_string());
        let install_url_with_rcupdate = format!("{}?rcupdate={}", install_url, update_rc_files);
        let installer_script = download_installer(&install_url_with_rcupdate).await?;

        // 5. Execute installer
        execute_installer(&installer_script, update_rc_files).await?;

        // 6. Verify installation succeeded
        verify_installation(&installation.dir).await?;

        // 7. Return result with paths and instructions
        let message = if update_rc_files {
            format!(
                "SDKMAN! installed successfully at {}. Shell configuration updated. Please restart your terminal or run: source {}/bin/sdkman-init.sh",
                installation.dir.display(),
                installation.dir.display()
            )
        } else {
            format!(
                "SDKMAN! installed successfully at {}. Shell RC files were not modified. Add this to your shell profile manually: [ -f {}/bin/sdkman-init.sh ] && source {}/bin/sdkman-init.sh",
                installation.dir.display(),
                installation.dir.display(),
                installation.dir.display()
            )
        };

        info!("SDKMAN! installation completed successfully");

        Ok(InstallationResult {
            installed: true,
            sdkman_dir: installation.dir.display().to_string(),
            message,
            shell_restart_required: update_rc_files,
        })
    }
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
async fn execute_installer(script: &str, _update_rc_files: bool) -> Result<(), SdkmanError> {
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

    // Note: RC file update control is handled via rcupdate query parameter in the download URL
    // See: https://sdkman.io/install/#install-without-modifying-shell-config

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
