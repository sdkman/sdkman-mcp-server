# Corrective Actions for F0: SDKMAN! Installation

Consider the following rules during execution of the tasks:
- rules/rust.md
- rules/mcp-best-practices.md
- rules/domain-driven-design.md

## Tasks

### Task 1: Add Support for Both Script and Native SDKMAN! Versions

- [x] Track both script and native versions in SdkmanInstallation struct and related code

**Prompt**: Update the SDKMAN! version tracking to support both script and native versions. Currently, the code only tracks the script version from `var/version`, but SDKMAN! also has a native version at `var/version_native`. 

Changes required:
1. Create a new `SdkmanVersions` struct with `script_version: Option<String>` and `native_version: Option<String>` fields
2. Update [`SdkmanInstallation`](src/installation.rs:19) struct to use `versions: Option<SdkmanVersions>` instead of `version: Option<String>`
3. Update [`read_version_from_metadata()`](src/installation.rs:158) to read both `var/version` and `var/version_native` files and return a `SdkmanVersions` struct
4. Update the installation detection message formatting to display both versions when available
5. Update integration tests in [`test_detect_existing_installation()`](tests/install_sdkman_integration_tests.rs:28) and [`test_detect_no_installation()`](tests/install_sdkman_integration_tests.rs:53) to assert both script and native versions
6. Update mock installation helper [`create_mock_sdkman_installation()`](tests/install_sdkman_integration_tests.rs:12) to create both version files for realistic testing

The native version file may not always exist, so handle it gracefully. When displaying version info, show "script: X.Y.Z, native: A.B.C" format when both are present, or just the available version if only one exists.

**Files affected**:
- `src/installation.rs`
- `tests/install_sdkman_integration_tests.rs`

### Task 2: Add RC Files Update Tracking to InstallationResult

- [ ] Add field to track whether RC files were updated during installation

**Prompt**: Update the [`InstallationResult`](src/installation.rs:28) struct to include a field that tracks whether RC files (bashrc, zshrc, etc.) were updated during installation. 

Changes required:
1. Add a new field `rc_files_updated: bool` to the `InstallationResult` struct
2. Update the [`SdkmanInstallation::install()`](src/installation.rs:60) method to set this field based on whether RC files were actually updated (consider the `should_update_rc` variable)
3. Update the tool schema documentation in [`prompts/02-f0-sdkman-installation_drive.md`](prompts/02-f0-sdkman-installation_drive.md:48) to include this new field in the output schema
4. Update the "Specification by Example" section in the drive document to show the new field in all response examples
5. Ensure the field is properly serialized in the JSON output

This will give users clear feedback about whether they need to manually update their RC files or if it was done automatically.

**Files affected**:
- `src/installation.rs`
- `prompts/02-f0-sdkman-installation_drive.md`

### Task 3: Refactor Helper Functions to Common Library Modules

- [ ] Move reusable helper functions from installation.rs to shared lib modules

**Prompt**: Refactor the installation-specific helper functions into reusable library modules for better code organization and potential reuse across features.

Changes required:
1. Create a new module `src/sdkman_helpers.rs` and add `get_sdkman_dir()` function (currently at line 141 of [`src/installation.rs`](src/installation.rs:141))
2. Create a new module `src/platform.rs` and move [`check_platform_compatibility()`](src/installation.rs:179) function there
3. Create a new module `src/shell.rs` and move [`detect_shell()`](src/installation.rs:233) function there
4. Add appropriate `pub mod` declarations in [`src/lib.rs`](src/lib.rs:1) to expose these new modules
5. Update imports in [`src/installation.rs`](src/installation.rs:1) to use the functions from their new locations
6. Move the associated unit tests:
   - Move [`test_get_sdkman_dir_default()`](src/installation.rs:399), [`test_get_sdkman_dir_from_env()`](src/installation.rs:407), and [`test_get_sdkman_dir_rejects_traversal()`](src/installation.rs:417) to the tests section of `src/sdkman_helpers.rs`
   - Move [`test_detect_shell()`](src/installation.rs:426) to the tests section of `src/shell.rs`
7. Ensure all tests still pass after the refactoring

Make sure to keep the same function signatures and behavior to avoid breaking existing code. The goal is only to improve code organization, not to change functionality.

**Files affected**:
- `src/installation.rs`
- `src/lib.rs`
- `src/sdkman_helpers.rs` (new file)
- `src/platform.rs` (new file)
- `src/shell.rs` (new file)

### Task 4: Add Architecture Detection for ARM64 vs Intel Platforms

- [ ] Enhance platform compatibility checks to differentiate between ARM64 and Intel architectures

**Prompt**: Extend the platform compatibility checks to detect and differentiate between ARM64 and Intel (x86_64) architectures on both Linux and macOS platforms.

Changes required:
1. Update the platform compatibility module (from Task 3, or in [`src/installation.rs`](src/installation.rs:179) if Task 3 isn't done yet) to detect CPU architecture
2. Use Rust's `std::env::consts::ARCH` to determine the architecture (will be "aarch64" for ARM64 or "x86_64" for Intel)
3. Add a new struct `PlatformInfo { os: String, arch: String }` to represent complete platform information
4. Update platform compatibility function to return or log architecture information
5. Add integration tests in [`tests/install_sdkman_integration_tests.rs`](tests/install_sdkman_integration_tests.rs:75):
   - Enhance [`test_platform_compatibility_linux()`](tests/install_sdkman_integration_tests.rs:77) to verify architecture detection on Linux
   - Enhance [`test_platform_compatibility_macos()`](tests/install_sdkman_integration_tests.rs:92) to verify architecture detection on macOS
6. Consider adding conditional compilation attributes (`#[cfg(target_arch = "aarch64")]` and `#[cfg(target_arch = "x86_64")]`) to run architecture-specific test assertions

This lays the groundwork for potential future features that might need to behave differently based on architecture (e.g., downloading architecture-specific binaries).

**Files affected**:
- `src/installation.rs` (or `src/platform.rs` if Task 3 is completed first)
- `tests/install_sdkman_integration_tests.rs`

## Execution Plan Workflow

The following workflow applies when executing this TODO list:
- Execute only the **SPECIFIED TASK**
- Implement the task in **THE SIMPLEST WAY POSSIBLE**
- Run the tests, format and perform static analysis on the code:
    - `cargo fmt`
    - `cargo test`
    - `cargo clippy -- -D warnings`
- **Ask me to review the task once you have completed and then WAIT FOR ME**
- Mark the TODO item as complete with [x]
- Commit the change to Git when I've approved and/or amended the code
- **STOP and await further instructions**
