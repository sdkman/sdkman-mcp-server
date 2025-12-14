# SDKMAN! MCP Server

> **ALPHA STATUS**: This MCP server is currently in early alpha development with limited tooling available. The feature set will expand significantly over time. See the [roadmap](#roadmap) for planned features.

An official [Model Context Protocol (MCP)](https://modelcontextprotocol.io) server for [SDKMAN!](https://sdkman.io), enabling AI assistants like Claude to manage Software Development Kits through natural language interactions.

**Status**: Alpha v0.0.1 | **Progress**: 2/15 tools implemented

## Overview

The SDKMAN! MCP Server allows AI assistants to manage development environments without switching to the terminal. Ask Claude to install SDKs, check versions, or manage your development tools directly through conversation.

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (bundled with Rust)

### Build from Source

```bash
git clone https://github.com/sdkman/sdkman-mcp-server.git
cd sdkman-mcp-server
cargo build --release
```

The binary will be available at `target/release/sdkman-mcp-server`.

### Install with Cargo

```bash
cargo install --path .
```

## Configuration

### Claude Desktop

Add to your Claude Desktop configuration:

**Linux**: `~/.config/Claude/claude_desktop_config.json`

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

Replace `/path/to/sdkman-mcp-server` with the actual binary path.

### Other MCP Clients

The server uses stdio transport per the MCP specification. Refer to your client's documentation for configuration.

## Available Tools

### `install_sdkman`

Installs SDKMAN! using the official installer from https://get.sdkman.io.

**Parameters:**
- `update_rc_files` (optional, boolean, default: true) - Update shell RC files

**Features:**
- Automatic platform detection (Linux, macOS, WSL, Git Bash)
- Existing installation detection
- Handles read-only RC files (eg, NixOS)
- Network retry with exponential backoff
- Installation verification

### `get_sdkman_version`

Retrieves SDKMAN! script and native version numbers.

**Parameters:** None

**Returns:** Version information from `~/.sdkman/var/version` and `~/.sdkman/var/version_native`

## Roadmap

Based on the [Product Requirements Document](specs/PRD.md), the following features are planned for v1.0:

| Feature | Tool | Status |
|---------|------|--------|
| **F0: SDKMAN! Installation** | | |
| | `install_sdkman` | ✓ |
| **F1: Candidate Discovery** | | |
| | `list_candidates` | |
| | `search_candidates` | |
| | `list_versions` | |
| | `get_default_version` | |
| **F2: SDK Installation** | | |
| | `validate_version` | |
| | `install_candidate` | |
| **F3: SDK Removal** | | |
| | `uninstall_candidate` | |
| **F4: Version Management** | | |
| | `set_default_version` | |
| **F5: Installation Inspection** | | |
| | `get_installed_versions` | |
| | `get_current_version` | |
| | `get_platform_info` | |
| **F6: Utility Commands** | | |
| | `get_candidate_home` | |
| | `get_sdkman_version` | ✓ |
| | `get_sdkman_config` | |
| **MCP Resources** | | |
| | `sdkman://installed` | |
| | `sdkman://installed/{candidate}` | |
| | `sdkman://config` | |

**Completed:** 2/15 tools | **Target:** v1.0 with all 15 tools and 3 resources

## Development

### Running Tests

```bash
# All tests
cargo test

# With debug output
RUST_LOG=debug cargo test

# Specific test
cargo test get_sdkman_version_integration_tests
```

### Logging

Set `RUST_LOG` environment variable:

```bash
RUST_LOG=info sdkman-mcp-server   # Default
RUST_LOG=debug sdkman-mcp-server  # Verbose
RUST_LOG=trace sdkman-mcp-server  # Very verbose
```

Logs are written to stderr to avoid interfering with stdio transport.

### Project Structure

```
sdkman-mcp-server/
├── src/
│   ├── main.rs          # MCP server implementation
│   ├── lib.rs           # Library exports
│   ├── installation.rs  # SDKMAN! installation logic
│   ├── versions.rs      # Version detection
│   └── utils/           # Utility modules
├── tests/               # Integration tests
├── specs/               # Product specifications
└── Cargo.toml          # Dependencies and metadata
```

## Platform Support

| Platform | Support |
|----------|---------|
| Linux (x86_64, ARM64) | Supported |
| macOS (Intel, Apple Silicon) | Supported |
| Windows (WSL, Git Bash) | Supported |
| Windows (Native) | **Not supported** |

## Security

- Path validation prevents directory traversal
- HTTPS for all network communications
- SHA256 checksum verification for downloads
- No credential storage
- Fail-safe error handling

Report security issues via [GitHub Issues](https://github.com/sdkman/sdkman-mcp-server/issues).

## Technology Stack

- **Rust** (Edition 2021) - Systems programming language
- **[rmcp](https://github.com/rust-mcp-stack/rmcp)** v0.11 - Rust MCP SDK
- **[Tokio](https://tokio.rs/)** - Asynchronous runtime
- **[serde](https://serde.rs/)** - Serialization framework
- **[tracing](https://github.com/tokio-rs/tracing)** - Structured logging

## Related Projects

- [SDKMAN! CLI](https://github.com/sdkman/sdkman-cli) - Official SDKMAN! command-line interface
- [SDKMAN! Native CLI](https://github.com/sdkman/sdkman-cli-native) - Official SDKMAN! native 🦀 extensions
- [Model Context Protocol](https://modelcontextprotocol.io) - Protocol specification
- [rmcp](https://github.com/rust-mcp-stack/rmcp) - Rust MCP SDK

## Resources

- [SDKMAN! Documentation](https://sdkman.io/install)
- [GitHub Issues](https://github.com/sdkman/sdkman-mcp-server/issues)
- [MCP Documentation](https://modelcontextprotocol.io)

## License

See [LICENSE](LICENSE) file for details.
