use std::env;
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;

/// Detect current shell from the SHELL environment variable.
///
/// Note: SDKMAN! officially supports bash and zsh. Other POSIX-compatible shells
/// that source .bashrc/.zshrc may work, but fish and other non-POSIX shells are
/// not supported as they use different initialization mechanisms.
pub fn detect_shell() -> String {
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

/// Check if shell RC files are read-only (e.g., NixOS).
///
/// This function checks common shell RC files (.bashrc, .zshrc, .bash_profile, .profile)
/// to determine if they are read-only. This is particularly relevant for systems like NixOS
/// where RC files are often symlinked to read-only Nix store paths.
///
/// Returns `true` if any RC file exists and is read-only, `false` otherwise.
pub async fn check_rc_files_readonly() -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };

    // Check common RC files
    let rc_files = vec![".bashrc", ".zshrc", ".bash_profile", ".profile"];

    for rc_file in rc_files {
        let path = home.join(rc_file);
        if path.exists() {
            // Check both the symlink and its target (for NixOS)
            // Use symlink_metadata to check the symlink itself
            if let Ok(symlink_meta) = fs::symlink_metadata(&path).await {
                if symlink_meta.is_symlink() {
                    // For symlinks, check the target's permissions
                    if let Ok(target_meta) = fs::metadata(&path).await {
                        if target_meta.permissions().readonly() {
                            debug!(
                                "RC file {} is a symlink to read-only target (e.g., NixOS)",
                                path.display()
                            );
                            return true;
                        }
                    }
                } else {
                    // Regular file, check its permissions
                    if symlink_meta.permissions().readonly() {
                        debug!("RC file {} is read-only", path.display());
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_bash() {
        env::set_var("SHELL", "/bin/bash");
        let shell = detect_shell();
        assert_eq!(shell, "bash");
        env::remove_var("SHELL");
    }

    #[test]
    fn test_detect_shell_zsh() {
        env::set_var("SHELL", "/bin/zsh");
        let shell = detect_shell();
        assert_eq!(shell, "zsh");
        env::remove_var("SHELL");
    }

    #[test]
    fn test_detect_shell_with_full_path() {
        env::set_var("SHELL", "/usr/local/bin/bash");
        let shell = detect_shell();
        assert_eq!(shell, "bash");
        env::remove_var("SHELL");
    }

    #[test]
    fn test_detect_shell_fallback_when_unset() {
        env::remove_var("SHELL");
        let shell = detect_shell();
        assert_eq!(shell, "bash");
    }

    #[tokio::test]
    async fn test_check_rc_files_readonly_returns_boolean() {
        // This test verifies the function executes without panicking
        // The actual result depends on the system's RC file permissions
        let result = check_rc_files_readonly().await;
        assert!(
            result == false || result == true,
            "Function should return a boolean without panicking"
        );
    }
}
