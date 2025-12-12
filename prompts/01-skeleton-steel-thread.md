# SDKMAN version - steel thread

Build the absolute minimum viable skeleton for the official SDKMAN! MCP (Model Context Protocol) server in Rust. This steel thread demonstrates ONE complete end-to-end flow: a simple `get_sdkman_version` tool that reads version information from a SDKMAN! installation.

This is a walking skeleton - the simplest possible implementation that exercises the entire system from MCP protocol handling to SDKMAN! filesystem interaction and back.

## Requirements

- Initialize Cargo project with minimal dependencies
- Set up basic project structure
- Implement MCP server with stdio transport using **rmcp** (official Rust MCP SDK)
- Register ONE tool: `get_sdkman_version` (fully implemented, not a stub)
- Set up basic logging with tracing
- Server starts, responds to MCP protocol messages, and shuts down gracefully
- The one tool actually reads from `~/.sdkman/var/version` and `~/.sdkman/var/version_native` and returns real data

## PRD

- specs/PRD.md

## Rules

- rules/domain-driven-design.md
- rules/mcp-best-practices.md
- rules/rust.md

## MCP Primitives

### Tools (Just One)

- **get_sdkman_version**: Get SDKMAN! script and native version numbers by reading from filesystem

### Resources

None for this steel thread

### Prompts

None for this steel thread

## Server Metadata

```json
{
  "name": "sdkman-mcp-server",
  "version": "0.0.1",
  "description": "Official MCP server for SDKMAN!",
  "capabilities": {
    "tools": {}
  }
}
```

## Tool Schema

### get_sdkman_version

**Input Schema:**
```json
{
  "type": "object",
  "properties": {}
}
```

**Output Schema:**
```json
{
  "type": "object",
  "properties": {
    "script_version": { "type": "string" },
    "native_version": { "type": "string" }
  }
}
```


## Error Handling

For this steel thread, keep error handling minimal:

- **-32603** (Internal Error): Server encountered unexpected error
- **-40001** (SDKMAN Not Installed): SDKMAN! not found on system (version files don't exist)

### Error Response Format

```json
{
  "code": -40001,
  "message": "SDKMAN! not installed",
  "data": {
    "checked_paths": ["~/.sdkman/var/version", "~/.sdkman/var/version_native"]
  }
}
```

## Domain

Minimal domain types for this steel thread:

```rust
/// SDKMAN! version information
pub struct SdkmanVersion {
    pub script_version: String,
    pub native_version: String,
}
```

## Transport Considerations

- **stdio**: Primary transport for Claude Desktop and CLI tools
- Server reads JSON-RPC requests from stdin
- Server writes JSON-RPC responses to stdout
- Logging goes to stderr to avoid protocol interference
- Binary is invoked directly by MCP clients

## Security Considerations

For this steel thread:
- Read-only filesystem access (just reading two version files)
- Path validation to prevent traversal attacks
- Proper error handling without exposing system internals
- No network calls
- No shell command execution

## Extra Considerations

- **CRITICAL**: Use `rmcp` (official SDK from modelcontextprotocol/rust-sdk), NOT `rust-mcp-sdk` (outdated community fork)
- Use `tracing` for structured logging (not println!)
- Error types use `thiserror`
- Serialize/deserialize with `serde` and `serde_json`
- Tool responses return proper MCP content blocks (text)
- Server handles SIGTERM/SIGINT for graceful shutdown
- All filesystem operations use Rust's standard library (`std::fs`, `std::path`)

## Testing Considerations

### Unit Tests
- Test reading version files from filesystem
- Test handling of missing files (SDKMAN! not installed)
- Test handling of malformed version files

### Integration Tests
- Test MCP protocol handshake
- Test `get_sdkman_version` tool invocation
- Test server initialization and shutdown

### End-to-End Tests
- Manual testing with MCP Inspector tool
- Test with Claude Desktop (if available)

## Implementation Notes

**For Rust:**
- Use `rmcp` (official MCP SDK) with `#[tool]` and `#[tool_router]` macros
- Error handling with `Result<T, Error>` and thiserror
- Async runtime: Tokio
- Serialization: serde and serde_json
- Logging: tracing with tracing-subscriber

**Project Structure (Minimal):**
```
sdkman-mcp-server/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, server initialization, tool implementation
│   └── error.rs             # Error types
└── tests/
    └── get_sdkman_version_integration_tests.rs  # Version integration test
```

**Key Dependencies (Cargo.toml):**
```toml
[dependencies]
rmcp = "0.11"  # Check crates.io for latest version
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.8"
```

**Example Tool Implementation:**
```rust
use rmcp::{ErrorData as McpError, model::*, tool_router};

#[derive(Clone)]
pub struct SdkmanServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SdkmanServer {
    fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }

    #[tool(description = "Get SDKMAN! script and native version numbers")]
    async fn get_sdkman_version(&self) -> Result<CallToolResult, McpError> {
        let version = SdkmanVersion::read_from_filesystem()?;
        Ok(CallToolResult::success(vec![Content::text(version.format())]))
    }
}
```

## Specification by Example

### Example: Calling get_sdkman_version (Success)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_sdkman_version",
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
        "text": "SDKMAN! Versions: Script: 5.18.2; Native: 0.4.6"
      }
    ]
  }
}
```

### Example: Calling get_sdkman_version (SDKMAN! Not Installed)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "get_sdkman_version",
    "arguments": {}
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -40001,
    "message": "SDKMAN! not installed",
    "data": {
      "checked_paths": ["~/.sdkman/var/version", "~/.sdkman/var/version_native"]
    }
  }
}
```

## Usage Example

For this steel thread, the server should be testable with MCP Inspector or manual stdio interaction:

```bash
# Build the server
cargo build --release

# Run with MCP Inspector
mcp-inspector ./target/release/sdkman-mcp-server

# Or test manually with JSON-RPC
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list"}' | ./target/release/sdkman-mcp-server

# Expected: List containing just the get_sdkman_version tool

# Call the tool
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"get_sdkman_version","arguments":{}}}' | ./target/release/sdkman-mcp-server

# Expected: Actual version numbers from your SDKMAN! installation
```

## Verification

- [ ] Cargo project compiles without errors
- [ ] Server starts and listens on stdio
- [ ] Server responds to `initialize` handshake
- [ ] Server responds to `tools/list` with the one tool
- [ ] `get_sdkman_version` reads actual files and returns real data
- [ ] `get_sdkman_version` returns error when SDKMAN! not installed
- [ ] Logging goes to stderr (not stdout)
- [ ] Server handles SIGTERM gracefully
- [ ] Basic integration test passes
- [ ] No warnings from `cargo clippy`
- [ ] Code is formatted with `cargo fmt`
