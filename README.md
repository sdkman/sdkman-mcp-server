# SDKMAN! MCP Server

> **⚠️ ALPHA STATUS**: This MCP server is currently in early alpha development with limited tooling available. The feature set will expand significantly over time. See the [roadmap](#roadmap) for planned features.

An official [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server for [SDKMAN!](https://sdkman.io), enabling AI assistants like Claude to manage Software Development Kits through natural language interactions.

## What is This?

The SDKMAN! MCP Server allows you to manage your development environment through AI assistants. Instead of switching to your terminal to run SDKMAN! commands, you can simply ask Claude:

- "What version of SDKMAN! do I have installed?"
- "Install Java 21 for me" *(coming soon)*
- "What Gradle versions are available?" *(coming soon)*

## Tech Stack

This project is built with:

- **[Rust](https://www.rust-lang.org/)** (Edition 2021) - Systems programming language for performance and reliability
- **[rmcp](https://github.com/rust-mcp-stack/rmcp)** v0.11 - Rust MCP SDK for building Model Context Protocol servers
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime for Rust
- **[serde](https://serde.rs/)** - Serialization framework
- **[tracing](https://github.com/tokio-rs/tracing)** - Structured logging

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Optional: SDKMAN! installed on your system

### Building from Source

```bash
# Clone the repository
git clone https://github.com/sdkman/sdkman-mcp-server.git
cd sdkman-mcp-server

# Build the project
cargo build --release

# The binary will be at target/release/sdkman-mcp-server
```

### Installing with Cargo

```bash
# Install directly from source
cargo install --path .

# Or once published to crates.io (future):
# cargo install sdkman-mcp-server
```

## Usage

### Running the Server

The MCP server communicates via stdio using the Model Context Protocol:

```bash
# Run the server directly
./target/release/sdkman-mcp-server

# Or if installed via cargo:
sdkman-mcp-server
```

### Configuring with Claude Desktop

Add the server to your Claude Desktop configuration file:

**macOS/Linux**: `~/Library/Application Support/Claude/claude_desktop_config.json`

**Windows**: `%APPDATA%/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "sdkman": {
      "command": "/path/to/sdkman-mcp-server"
    }
  }
}
```

Replace `/path/to/sdkman-mcp-server` with the actual path to your compiled binary.

### Configuring with Other MCP Clients

Any MCP-compatible client can use this server. Refer to your client's documentation for configuration details. The server uses stdio transport as defined in the MCP specification.

## Currently Supported Tools

The following MCP tools are currently available:

### 🔍 `get_sdkman_version`

Get the installed SDKMAN! script and native version numbers.

**Description**: Retrieves version information from your SDKMAN! installation by reading from `~/.sdkman/var/version` and `~/.sdkman/var/version_native`.

**Parameters**: None

**Example usage** (via Claude):
```
User: "What version of SDKMAN! do I have?"
Claude: [calls get_sdkman_version]
Claude: "SDKMAN! Versions: Script: 5.18.2; Native: 0.4.6"
```

**Error handling**:
- Returns error if SDKMAN! is not installed
- Provides checked file paths in error response for troubleshooting

### 🛠️ `install_sdkman`

Downloads and executes the official SDKMAN! installer script, creating the complete SDKMAN! environment with automatic shell configuration.

**Description**: Installs SDKMAN! on your system using the official installer from https://get.sdkman.io. This tool enables first-time users to set up SDKMAN! through natural language interaction with AI assistants.

**Parameters**:
- `update_rc_files` (optional, boolean, default: `true`): Whether to update shell RC files. Set to false to skip shell profile updates.

**Example usage** (via Claude):
```
User: "Install SDKMAN! for me"
Claude: [calls install_sdkman with default parameters]
Claude: "SDKMAN! installed successfully at /home/user/.sdkman. Shell configuration updated for bash. Please restart your terminal or run: source /home/user/.sdkman/bin/sdkman-init.sh"
```

**Features**:
- Automatic platform detection (supports Linux, macOS, WSL, Git Bash)
- Rejects native Windows (CMD/PowerShell) with helpful guidance
- Detects existing installations and skips if already installed
- Handles read-only RC files (e.g., NixOS)
- Respects `SDKMAN_DIR` environment variable
- Network retry logic with exponential backoff
- Installation verification

**Error handling**:
- Platform errors: Rejects native Windows with installation alternatives
- Network errors: Provides retry guidance and fallback options
- Permission errors: Clear instructions for resolution
- Already installed: Returns success with version information

## Roadmap

The following tools are planned for future releases (see [PRD](specs/PRD.md) for details):

### Phase 1: Discovery (Coming Soon)
- `list_candidates` - List all available SDK candidates
- `search_candidates` - Search for specific SDKs
- `list_versions` - List available versions for a candidate
- `get_default_version` - Get recommended version for a candidate

### Phase 2: Installation & Management (Partial)
- ✅ `install_sdkman` - Install SDKMAN! using the official installer
- `install_candidate` - Install a specific SDK version
- `uninstall_candidate` - Remove an SDK version
- `set_default_version` - Set the default version for an SDK

### Phase 3: Inspection & Information (Coming Soon)
- `get_installed_versions` - List locally installed SDK versions
- `get_current_version` - Get the currently active version
- `get_platform_info` - Get platform and configuration details
- `get_candidate_home` - Get path to installed SDK
- `get_sdkman_config` - Get SDKMAN! configuration settings

**Progress**: 2/15 tools implemented | **Target**: 15 total tools for v1.0 release

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with logging output
RUST_LOG=debug cargo test

# Run specific test
cargo test get_sdkman_version_integration_tests
```

### Enabling Debug Logging

The server uses the `tracing` framework for logging. Set the `RUST_LOG` environment variable to control log levels:

```bash
# Info level (default)
RUST_LOG=info sdkman-mcp-server

# Debug level (verbose)
RUST_LOG=debug sdkman-mcp-server

# Trace level (very verbose)
RUST_LOG=trace sdkman-mcp-server
```

Logs are written to stderr to avoid interfering with the stdio MCP transport.

### Project Structure

```
sdkman-mcp-server/
├── src/
│   ├── main.rs              # MCP server implementation
│   ├── lib.rs               # Library exports
│   ├── installation.rs      # SDKMAN! installation logic
│   ├── versions.rs          # Version detection logic
│   └── error.rs             # Error types and handling
├── tests/                   # Integration tests
├── specs/                   # Product specifications
├── Cargo.toml              # Dependencies and metadata
└── README.md               # This file
```

## Platform Support

Currently tested on:
- ✅ Linux (x86_64)
- ✅ macOS (Intel and Apple Silicon)
- 🔄 Windows (via WSL/Git Bash)

Native Windows support is planned but not yet available.

## Security

This server follows security best practices:

- ✅ Validates file paths to prevent directory traversal
- ✅ Uses HTTPS for all future API communications
- ✅ Verifies checksums for SDK downloads (when implemented)
- ✅ No credential storage
- ✅ Read-only operations in current alpha

Security issues should be reported via GitHub issues.

## License

See [LICENSE](LICENSE) file for details.

## Related Projects

- [SDKMAN!](https://github.com/sdkman/sdkman-cli) - The official SDKMAN! CLI
- [Model Context Protocol](https://modelcontextprotocol.io) - Protocol specification
- [rmcp](https://github.com/rust-mcp-stack/rmcp) - Rust MCP SDK

## Support

- 📖 [SDKMAN! Documentation](https://sdkman.io/usage)
- 💬 [GitHub Issues](https://github.com/sdkman/sdkman-mcp-server/issues)
- 🌐 [MCP Documentation](https://modelcontextprotocol.io)

---

**Status**: Alpha v0.0.1 | **Last Updated**: 2025-12-12
