use super::error::SdkmanError;
use std::env;
use std::process::Stdio;
use tokio::process::Command;

/// Platform information including OS and architecture
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformInfo {
    pub os: String,
    pub arch: String,
    pub triple: Option<String>,
}

impl PlatformInfo {
    /// Get the current platform information
    pub fn detect() -> Self {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();
        let triple = get_platform_triple();

        Self { os, arch, triple }
    }
}

/// Get the platform triple identifier
/// Returns one of the allowed Rust triple identifiers for SDKMAN!, or None if unsupported
fn get_platform_triple() -> Option<String> {
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    match (os, arch) {
        ("linux", "aarch64") => Some("aarch64-unknown-linux-gnu".to_string()),
        ("linux", "x86_64") => Some("x86_64-unknown-linux-gnu".to_string()),
        ("linux", "x86") => Some("i686-unknown-linux-gnu".to_string()),
        ("macos", "aarch64") => Some("aarch64-apple-darwin".to_string()),
        ("macos", "x86_64") => Some("x86_64-apple-darwin".to_string()),
        ("windows", "x86_64") => Some("x86_64-pc-windows-msvc".to_string()),
        _ => None,
    }
}

/// Check platform compatibility - only allow supported platform triples
pub async fn check_platform_compatibility() -> Result<(), SdkmanError> {
    let platform = PlatformInfo::detect();
    
    match &platform.triple {
        Some(triple) => {
            tracing::debug!(
                "Platform detected: OS={}, Architecture={}, Triple={}",
                platform.os,
                platform.arch,
                triple
            );
            Ok(())
        }
        None => {
            Err(SdkmanError::UnsupportedPlatform {
                details: format!("Platform '{}/{}' is not supported by SDKMAN!", platform.os, platform.arch),
                recovery: [
                    "SDKMAN! is only supported on the following platforms:",
                    "  • Linux aarch64",
                    "  • Linux x86_64",
                    "  • Linux i686",
                    "  • macOS aarch64 (Apple Silicon)",
                    "  • macOS x86_64 (Intel)",
                    "  • Windows x86_64 (WSL/Git Bash)",
                ]
                .join("\n"),
            })
        }
    }
}

/// Check if bash is available in the environment
pub async fn check_bash_available() -> Result<(), SdkmanError> {
    let has_bash = Command::new("bash")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
        .is_ok();

    if !has_bash {
        return Err(SdkmanError::BashNotAvailable {
            details: "SDKMAN! requires bash shell to be available".to_string(),
            recovery: [
                "Please ensure bash is installed and available in your PATH.",
                "On Windows, consider installing Git Bash or WSL to provide bash support.",
            ]
            .join("\n"),
        });
    }

    Ok(())
}
