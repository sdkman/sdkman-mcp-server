use tokio::fs;
use tracing::debug;

/// Check if shell RC files are read-only (e.g., NixOS).
///
/// This function checks common shell RC files (.bashrc, .zshrc, .bash_profile, .profile)
/// to determine if they are read-only. This is particularly relevant for systems like NixOS
/// where RC files are often symlinked to read-only Nix store paths.
///
/// Returns `true` if any RC file exists and is read-only, `false` otherwise. Never panics.
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
