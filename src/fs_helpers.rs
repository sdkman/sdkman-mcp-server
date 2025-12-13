use std::env;
use std::path::PathBuf;
use tracing::warn;

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

    // Default to $HOME/.sdkman
    dirs::home_dir()
        .expect("Unable to determine home directory")
        .join(".sdkman")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Use a mutex to ensure tests don't run in parallel and interfere with each other
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_get_sdkman_dir_default() {
        let _lock = TEST_MUTEX.lock().unwrap();

        // Save current value
        let saved = env::var("SDKMAN_DIR").ok();

        // Test with SDKMAN_DIR unset
        env::remove_var("SDKMAN_DIR");
        let dir = get_sdkman_dir();
        let home = dirs::home_dir().expect("Unable to determine home directory");
        let expected = home.join(".sdkman");
        assert_eq!(
            dir, expected,
            "Directory should be home/.sdkman when SDKMAN_DIR is not set"
        );
        assert!(dir.ends_with(".sdkman"));

        // Restore original value
        if let Some(val) = saved {
            env::set_var("SDKMAN_DIR", val);
        }
    }

    #[test]
    fn test_get_sdkman_dir_from_env() {
        let _lock = TEST_MUTEX.lock().unwrap();

        // Save current value
        let saved = env::var("SDKMAN_DIR").ok();

        let test_dir = "/tmp/test-sdkman";
        env::set_var("SDKMAN_DIR", test_dir);
        let dir = get_sdkman_dir();
        assert_eq!(dir.display().to_string(), test_dir);

        // Restore original value
        match saved {
            Some(val) => env::set_var("SDKMAN_DIR", val),
            None => env::remove_var("SDKMAN_DIR"),
        }
    }

    #[test]
    fn test_get_sdkman_dir_rejects_traversal() {
        let _lock = TEST_MUTEX.lock().unwrap();

        // Save current value
        let saved = env::var("SDKMAN_DIR").ok();

        env::set_var("SDKMAN_DIR", "/tmp/../etc/passwd");
        let dir = get_sdkman_dir();
        let home = dirs::home_dir().expect("Unable to determine home directory");
        assert_eq!(dir, home.join(".sdkman"));

        // Restore original value
        match saved {
            Some(val) => env::set_var("SDKMAN_DIR", val),
            None => env::remove_var("SDKMAN_DIR"),
        }
    }
}
