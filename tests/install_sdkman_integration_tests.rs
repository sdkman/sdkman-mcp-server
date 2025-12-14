use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to set up a temporary SDKMAN_DIR for testing
fn setup_temp_sdkman_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper to create a mock SDKMAN installation structure
fn create_mock_sdkman_installation(dir: &PathBuf) {
    fs::create_dir_all(dir.join("bin")).expect("Failed to create bin dir");
    fs::create_dir_all(dir.join("candidates")).expect("Failed to create candidates dir");
    fs::create_dir_all(dir.join("var")).expect("Failed to create var dir");

    fs::write(
        dir.join("bin/sdkman-init.sh"),
        "#!/bin/bash\n# Mock SDKMAN init script\n",
    )
    .expect("Failed to write init script");

    fs::write(dir.join("var/version"), "5.18.2").expect("Failed to write version file");

    fs::write(dir.join("var/version_native"), "0.4.6")
        .expect("Failed to write native version file");
}

#[tokio::test]
async fn test_detect_existing_installation() {
    use sdkman_mcp_server::installation::SdkmanInstallation;

    let temp_dir = setup_temp_sdkman_dir();
    let sdkman_dir = temp_dir.path().to_path_buf();

    // Create mock installation
    create_mock_sdkman_installation(&sdkman_dir);

    // Set SDKMAN_DIR environment variable
    env::set_var("SDKMAN_DIR", sdkman_dir.to_str().unwrap());

    let installation = SdkmanInstallation::detect()
        .await
        .expect("Failed to detect installation");

    assert!(installation.is_installed);

    // Clean up
    env::remove_var("SDKMAN_DIR");
}

#[tokio::test]
async fn test_detect_no_installation() {
    let temp_dir = setup_temp_sdkman_dir();
    let sdkman_dir = temp_dir.path().to_path_buf();

    // Set SDKMAN_DIR but don't create installation
    env::set_var("SDKMAN_DIR", sdkman_dir.to_str().unwrap());

    use sdkman_mcp_server::installation::SdkmanInstallation;

    let installation = SdkmanInstallation::detect()
        .await
        .expect("Failed to detect (non)installation");

    assert!(!installation.is_installed);

    // Clean up
    env::remove_var("SDKMAN_DIR");
}

