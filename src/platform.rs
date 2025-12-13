use crate::error::SdkmanError;
use std::process::Stdio;
use tokio::process::Command;

/// Check platform compatibility - reject native Windows
pub async fn check_platform_compatibility() -> Result<(), SdkmanError> {
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
                details: "SDKMAN! requires bash shell to be available".to_string(),
                recovery: [
                    "Please ensure bash is installed and available in your PATH.",
                    "",
                    "Supported environments:",
                    "  • Linux",
                    "  • macOS",
                    "  • Windows Subsystem for Linux (WSL)",
                    "  • Git Bash for Windows",
                ]
                .join("\n"),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_platform_compatibility() {
        // This test validates that the function executes without panicking
        // On Unix systems (Linux/macOS), it should succeed
        // On Windows with bash (WSL/Git Bash), it should succeed
        // On Windows without bash, it should return UnsupportedPlatform error
        let result = check_platform_compatibility().await;

        // On Unix platforms, this should always succeed
        #[cfg(not(target_os = "windows"))]
        assert!(
            result.is_ok(),
            "Platform compatibility check should succeed on Unix systems"
        );

        // On Windows, the result depends on whether bash is available
        // We can't make a definitive assertion here as it depends on the environment
        #[cfg(target_os = "windows")]
        {
            // Just ensure it returns either Ok or the expected error type
            if let Err(e) = result {
                assert!(matches!(e, SdkmanError::UnsupportedPlatform { .. }));
            }
        }
    }
}
