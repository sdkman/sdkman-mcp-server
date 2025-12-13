# F0: SDKMAN! Installation

This feature enables first-time users to install SDKMAN! through the MCP server using the official installer. Users without an existing SDKMAN! installation can set up the complete environment (CLI + directory structure) through natural language interaction with AI assistants, eliminating manual installation steps and ensuring proper shell integration.

## Requirements

- Downloads official installer from https://get.sdkman.io via HTTPS
- Executes installer script with appropriate flags for non-interactive mode
- Verifies successful installation by checking for `~/.sdkman` directory and CLI scripts
- Provides clear feedback during installation process
- Handles installation errors gracefully with actionable messages
- Completes in < 60 seconds (network dependent)
- Works across all supported platforms (Linux, macOS, Git Bash, WSL)
- Forces a **hard stop** if the process is running in a Windows CMD shell
- Detects and skips installation if SDKMAN! already exists
- Automatically configures shell profiles (bash/zsh) via official installer

## Rules

- rules/rust.md
- rules/mcp-best-practices.md
- rules/domain-driven-design.md

## MCP Primitives

### Tools

- **install_sdkman**: Downloads and executes the official SDKMAN! installer script, creating the complete SDKMAN! environment with automatic shell configuration

## Tool Schemas

### install_sdkman

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "update_rc_files": {
      "type": "boolean",
      "description": "Whether to update shell RC files (default: true). Set to false to skip shell profile updates.",
      "default": true
    }
  }
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "installed": {
      "type": "boolean",
      "description": "Whether installation was successful"
    },
    "sdkman_dir": {
      "type": "string",
      "description": "Path to SDKMAN! installation directory (~/.sdkman or $SDKMAN_DIR)"
    },
    "message": {
      "type": "string",
      "description": "Human-readable installation result message"
    },
    "shell_restart_required": {
      "type": "boolean",
      "description": "Whether user needs to restart terminal or source init script"
    },
    "rc_files_updated": {
      "type": "boolean",
      "description": "Whether shell RC files (bashrc, zshrc, etc.) were updated during installation"
    }
  }
}
```

## Error Handling

### Error Codes

- **-32603** (Internal Error): When installer download fails, execution fails, or verification fails
- **-32602** (Invalid Params): When parameters don't match schema
- **Custom -1000** (Unsupported Platform): When running on native Windows (CMD/PowerShell) - only WSL and Git Bash are supported
- **Custom -1001** (Already Installed): When SDKMAN! is already installed (not an error, returns success with message)
- **Custom -1002** (Network Error): When installer download fails due to network issues
- **Custom -1003** (Permission Error): When insufficient permissions to create directories or execute installer

### Error Response Format

```json
{
  "code": -1002,
  "message": "Failed to download SDKMAN! installer",
  "data": {
    "details": "Network connection timeout after 30s",
    "recovery": "Check your internet connection and try again. Visit https://sdkman.io/install for manual installation."
  }
}
```

## Domain

```rust
// Installation state representation
#[derive(Debug, Clone)]
pub struct SdkmanInstallation {
    pub dir: PathBuf,           // ~/.sdkman or $SDKMAN_DIR
    pub is_installed: bool,     // Whether SDKMAN! is installed
    pub version: Option<String>, // Script version if installed
}

impl SdkmanInstallation {
    /// Detect existing SDKMAN! installation
    pub fn detect() -> Result<Self, Error> {
        let dir = get_sdkman_dir();
        let is_installed = dir.join("bin/sdkman-init.sh").exists();
        let version = if is_installed {
            Some(read_version_from_metadata(&dir)?)
        } else {
            None
        };
        
        Ok(Self { dir, is_installed, version })
    }
    
    /// Install SDKMAN! using official installer
    pub async fn install(update_rc_files: bool) -> Result<InstallationResult, Error> {
        // 1. Detect platform and reject native Windows (CMD/PowerShell)
        // 2. Check if already installed
        // 3. Download installer from https://get.sdkman.io
        // 4. Execute with appropriate flags
        // 5. Verify installation succeeded
        // 6. Return result with paths and instructions
    }
}

#[derive(Debug)]
pub struct InstallationResult {
    pub installed: bool,
    pub sdkman_dir: PathBuf,
    pub message: String,
    pub shell_restart_required: bool,
}
```

## Transport Considerations

- **stdio**: Primary transport, installer runs as child process with output capture
- Installation script output should be streamed or buffered for user feedback
- Long-running operation (up to 60s), client must handle timeout appropriately

## Security Considerations

- HTTPS only for installer download (enforced by curl, no HTTP fallback)
- Official installer maintained by SDKMAN! team at https://get.sdkman.io
- User should be informed before executing installer (good AI practice)
- Installer script piped directly to bash (not saved to disk first, follows official pattern)
- No credentials required or stored
- Installation runs with user permissions with **no sudo/root allowed**
- Validate SDKMAN_DIR environment variable if set to prevent path traversal

## Extra Considerations

- **Platform Detection**: MUST detect operating system before attempting installation
  - **Note**: This is minimal platform validation for installation only (can we run bash?)
  - Comprehensive platform detection is provided by F5's `get_platform_info` tool
  - Both can share underlying platform detection utilities in the implementation
- **Native Windows Rejection**: MUST reject installation attempts on native Windows (CMD/PowerShell) with clear error directing users to WSL or Git Bash
- Supported environments: Linux, macOS, Git Bash for Windows, WSL (Windows Subsystem for Linux)
- SDKMAN_DIR environment variable must be respected if already set
- Installation creates `~/.sdkman` by default if SDKMAN_DIR not set
- Shell profile updates are automatic via official installer (bash/zsh)
- Post-installation, user must restart terminal or source `~/.sdkman/bin/sdkman-init.sh`
- Partial installations should be detected (directory exists but incomplete)
- Installation is idempotent - safe to call multiple times
- Network failures should provide retry guidance
- Platform-specific handling delegated to official installer (Linux, macOS, Git Bash, WSL)
- Keep platform detection at **it's barest minimum**: we only care about Windows detection (CMD/Posh) now, will expand this later when needed
- When detecting an existing installation, report **both** script and native versions
- Before installation, **first check if the rc files are readonly** (NixOS): and use the appropriate installation method with `rc_update` flag
- Inform the user if the rc files are not updated due to readonly permissions
- Instruct the user how to remedy when rc files are readonly
- Provide a way to override the rc file updating and apply correct `rc_update` flag on get.sdkman.io so that we don't cause failures!
- Make the default install hook url overridable for testing

## Testing Considerations

### Unit Tests
- Test SDKMAN! detection logic (installed vs not installed)
- Test path resolution (SDKMAN_DIR vs default ~/.sdkman)
- Test URL construction for installer download
- Test installation state verification logic

### Integration Tests
- Test full installation flow on clean system (CI matrix)
- Test detection of existing installation (skip case)
- Test with SDKMAN_DIR environment variable set
- Test installation with update_rc_files=false|true
- Test partial installation detection and handling
- Mock installer script execution for fast tests
- Override install hook url with mock installer script location
- Mock install hooks should create most essential directories

## Implementation Notes

**For Rust:**
- **Platform Detection**: Use `std::env::consts::OS` to detect operating system at runtime
  - Reject if `OS == "windows"` AND not running in WSL/Git Bash (check for bash via `which bash` or `$SHELL` env var)
  - Accept: `OS == "linux"`, `OS == "macos"`, or Windows with bash available
- Use `reqwest` for HTTPS download of installer script
- Use `tokio::process::Command` for executing bash with installer script
- Capture stdout/stderr for progress feedback
- Timeout protection (60s max)
- Verify installation by checking filesystem:
  - `~/.sdkman/bin/sdkman-init.sh` exists
  - `~/.sdkman/candidates/` directory exists
  - `~/.sdkman/var/version` file exists
- Environment variable handling: `std::env::var("SDKMAN_DIR")`
- Use `dirs` crate for cross-platform home directory resolution

**Error Handling:**
- **FIRST**: Check platform compatibility and fail fast on native Windows
- Distinguish between "already installed" (success) and "installation failed" (error)
- Provide recovery instructions for all error cases
- Retry logic for transient network failures (3 attempts with exponential backoff)

## Specification by Example

### Example: Installing SDKMAN! (First Time)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "install_sdkman",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"installed\": true,\n  \"sdkman_dir\": \"/home/alice/.sdkman\",\n  \"message\": \"SDKMAN! installed successfully at /home/alice/.sdkman. Shell configuration updated for bash. Please restart your terminal or run: source /home/alice/.sdkman/bin/sdkman-init.sh\",\n  \"shell_restart_required\": true\n}"
      }
    ]
  }
}
```

### Example: Already Installed

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "install_sdkman",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"installed\": true,\n  \"sdkman_dir\": \"/home/alice/.sdkman\",\n  \"message\": \"SDKMAN! is already installed at /home/alice/.sdkman (version 5.18.2)\",\n  \"shell_restart_required\": false\n}"
      }
    ]
  }
}
```

### Example: Installation Without RC Updates

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "install_sdkman",
    "arguments": {
      "update_rc_files": false
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "{\n  \"installed\": true,\n  \"sdkman_dir\": \"/home/alice/.sdkman\",\n  \"message\": \"SDKMAN! installed successfully at /home/alice/.sdkman. Shell RC files were not modified. Add this to your shell profile manually: source /home/alice/.sdkman/bin/sdkman-init.sh\",\n  \"shell_restart_required\": false\n}"
      }
    ]
  }
}
```

### Example: Network Error

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "error": {
    "code": -1002,
    "message": "Failed to download SDKMAN! installer",
    "data": {
      "details": "Connection timeout after 30s (attempted 3 times)",
      "recovery": "Check your internet connection and try again. Visit https://sdkman.io/install for manual installation or check service status at https://sdkman.io/status"
    }
  }
}
```

### Example: Unsupported Platform (Native Windows)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "install_sdkman",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "error": {
    "code": -1000,
    "message": "SDKMAN! installation not supported on native Windows",
    "data": {
      "details": "SDKMAN! requires a Unix-like environment with bash shell",
      "recovery": "Please use one of these supported environments:\n• Windows Subsystem for Linux (WSL) - Recommended: https://docs.microsoft.com/en-us/windows/wsl/install\n• Git Bash for Windows: https://gitforwindows.org/\n\nAlternatively, use SDKMAN! on Linux or macOS."
    }
  }
}
```

## Usage Example

```typescript
// Example AI assistant interaction flow
const client = new MCPClient({
  serverName: "sdkman-mcp-server",
  transport: "stdio"
});

await client.connect();

// Check if SDKMAN! is installed (via get_platform_info or detect installation)
const platformInfo = await client.callTool({
  name: "get_platform_info",
  arguments: {}
});

// If not installed, install it
if (!platformInfo.sdkman_installed) {
  const result = await client.callTool({
    name: "install_sdkman",
    arguments: {}
  });
  
  console.log(result.message);
  // "SDKMAN! installed successfully at /home/user/.sdkman. 
  //  Please restart your terminal or run: source ~/.sdkman/bin/sdkman-init.sh"
}

// After installation, proceed with SDK management
const candidates = await client.callTool({
  name: "list_candidates",
  arguments: {}
});
```

## Verification

- [ ] Tool exposes correct JSON schema for install_sdkman
- [ ] **Rejects native Windows (CMD/PowerShell) with error code -1000 and actionable guidance**
- [ ] Detects existing SDKMAN! installation correctly
- [ ] Downloads installer from https://get.sdkman.io via HTTPS
- [ ] Executes installer script with appropriate flags
- [ ] Verifies installation by checking directory structure
- [ ] Returns proper error for network failures with retry count
- [ ] Returns proper error for permission failures
- [ ] Handles SDKMAN_DIR environment variable correctly
- [ ] Completes installation in < 60 seconds (network dependent)
- [ ] Works on Linux x64 (tested in CI)
- [ ] Works on macOS Intel and ARM64 (tested in CI)
- [ ] Works on Windows Git Bash (tested in CI)
- [ ] Works on Windows WSL (tested in CI)
- [ ] Provides actionable error messages with recovery steps for all error cases
- [ ] Integration test: install → verify → list_candidates flow
- [ ] Installation is idempotent (can run multiple times safely)
- [ ] Shell configuration updated correctly (when update_rc_files=true)
