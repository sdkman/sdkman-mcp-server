# PRODUCT REQUIREMENTS DOCUMENT (PRD)
## SDKMAN! MCP Server

**Version**: 1.0
**Status**: Draft
**Last Updated**: 2025-12-11
**Author**: SDKMAN! Team

---

## 1. Product Overview

### 1.1 Vision
Enable developers to manage Software Development Kits through natural language interactions with AI assistants, making SDK management as simple as asking Claude "install the latest Java LTS version" or "what Gradle versions are available?"

### 1.2 Product Name
**SDKMAN! MCP Server** - Official Model Context Protocol server for SDKMAN!

### 1.3 Target Users

**Primary Users**:
- **Individual Developers**: Using Claude Desktop or AI-powered IDEs to manage their local development environment
- **Development Teams**: Standardizing SDK versions across team members through AI-assisted workflows
- **DevOps Engineers**: Automating SDK management in development and CI/CD environments

**User Personas**:

1. **Alice - Full-Stack Developer**
   - Uses multiple JVM languages (Java, Kotlin, Scala)
   - Switches between projects with different SDK requirements
   - Wants quick, friction-free SDK management
   - Currently uses SDKMAN! CLI but prefers natural language with Claude

2. **Bob - DevOps Engineer**
   - Manages development environments for entire team
   - Automates SDK installations in Docker containers and CI pipelines
   - Needs programmatic, reliable SDK management
   - Values consistency and auditability

3. **Carol - New Developer**
   - Just learned about SDKMAN! and overwhelmed by CLI options
   - Prefers asking questions and getting guidance
   - Needs hand-holding for SDK selection and installation
   - Benefits from AI explaining what each SDK is for

### 1.4 Problem Statement

**Current State**:
- Developers must context-switch from their AI assistant to terminal to use SDKMAN! CLI
- SDK management commands must be memorized or looked up
- No way for AI assistants to directly manage SDKs on behalf of users
- Complex installations (Java on macOS) require understanding of platform quirks

**Desired State**:
- Developers can ask Claude to manage SDKs without leaving their conversation
- AI can discover, install, and configure SDKs through natural language requests
- Complex SDK installations are handled automatically
- SDK state is visible to AI for contextual recommendations

### 1.5 Success Criteria

**User Experience**:
- - User can install an SDK in < 30 seconds from initial request to Claude
- - Zero prerequisite knowledge required (AI handles everything)
- - Error messages are actionable (user knows what to do next)
- - Compatible with existing SDKMAN! installations (no migration needed)

**Technical**:
- - Installation success rate > 95% (excluding network failures)
- - API response time P90 < 3 seconds
- - Memory usage < 50MB when idle
- - Test coverage > 80%

**Adoption**:
- - Accepted by SDKMAN! team as official MCP server
- - 100+ GitHub stars in first 3 months
- - Featured in SDKMAN! documentation
- - Positive community feedback (> 90% positive sentiment)

---

## 2. Feature Requirements

### 2.0 MCP Tools Summary (15 Total)

**Setup & Discovery** (6 tools):
1. `install_sdkman` - Install SDKMAN! using official installer (F0)
2. `list_candidates` - List all SDK candidates (F1)
3. `search_candidates` - Search candidates (F1)
4. `list_versions` - List versions for a candidate (F1)
5. `get_default_version` - Get recommended version (F1)
6. `validate_version` - Check version availability (F2)

**Installation & Management** (3 tools):
7. `install_candidate` - Install an SDK version (F2)
8. `uninstall_candidate` - Remove an SDK version (F3)
9. `set_default_version` - Set default version (F4)

**Information & Inspection** (6 tools):
10. `get_installed_versions` - List installed versions (F5)
11. `get_current_version` - Get current default version (F5, maps to `sdk current`)
12. `get_platform_info` - Platform and SDKMAN! status (F5)
13. `get_candidate_home` - Get absolute path to SDK (F6, maps to `sdk home`)
14. `get_sdkman_version` - Get SDKMAN! versions (F6, maps to `sdk version`)
15. `get_sdkman_config` - Get configuration settings (F6, maps to `sdk config`)

**MCP Resources** (3 total):
- `sdkman://installed` - All installed SDKs (JSON)
- `sdkman://installed/{candidate}` - Specific candidate details
- `sdkman://config` - Configuration and platform info

**Not Included in v1.0**:
- `sdk env` commands (init, install, clear) - Requires shell session manipulation, marked as P2 feature
- `sdk update` - SDKMAN! CLI self-update (not needed for MCP server)
- `sdk selfupdate` - Same as above
- `sdk flush` - Cache management (handled automatically by API client)
- `sdk offline` - Offline mode (explicitly out of scope)

---

### 2.1 P0 Features (Must-Have for v1.0)

#### F0: SDKMAN! Installation
**Description**: Users without SDKMAN! can install it via the MCP server using the official installer

**MCP Tools**:
- `install_sdkman` - Download and execute the official SDKMAN! installer script

**User Stories**:
- As a new user, I want to set up SDKMAN! through Claude without manual installation
- As a developer, I want the MCP server to handle SDKMAN! installation automatically
- As a user, I want the official, tested installation process

**Acceptance Criteria**:
- Downloads official installer from https://get.sdkman.io
- Executes installer script with appropriate flags
- Verifies successful installation (checks for `~/.sdkman` directory and CLI scripts)
- Provides feedback during installation process
- Handles installation errors gracefully with actionable messages
- Completes in < 60 seconds (network dependent)
- Works across all supported platforms (Linux, macOS, Git Bash, WSL)
- Detects and skips installation if SDKMAN! already exists

**Implementation**:
```bash
# The MCP server will execute:
curl -s "https://get.sdkman.io" | bash

# Or for non-interactive mode:
curl -s "https://get.sdkman.io?rcupdate=false" | bash
```

**Security Considerations**:
- HTTPS only (enforced by curl)
- Official installer is maintained by SDKMAN! team
- User confirms before executing installer script
- Installer script is piped directly (not saved to disk first)

**Notes**:
- Uses official installer for maximum compatibility
- Automatically configures shell profiles (bash/zsh)
- Creates complete SDKMAN! environment (CLI + directory structure)
- After installation, all other MCP tools work seamlessly

---

#### F1: Candidate Discovery
**Description**: Users can discover and search for available SDK candidates

**MCP Tools**:
- `list_candidates` - List all SDK candidates with descriptions
- `search_candidates` - Search by name or description
- `list_versions` - List available versions for a candidate
- `get_default_version` - Get recommended version

**User Stories**:
- As a developer, I want to see all available SDKs so I can discover what's available
- As a developer, I want to search for SDKs by name so I can quickly find what I need
- As a developer, I want to see all Java versions so I can choose the right one
- As a developer, I want to know the recommended version so I don't have to research

**Acceptance Criteria**:
- Lists return results in < 2 seconds (90th percentile)
- Search is case-insensitive and matches name/description
- Version lists show installed/current/available status
- Results are cached for 30 minutes to minimize API calls

---

#### F2: SDK Installation
**Description**: Users can install specific SDK versions

**MCP Tools**:
- `install_candidate` - Install a specific version
- `validate_version` - Verify version exists before install

**User Stories**:
- As a developer, I want to install Java 21 so I can use the latest LTS
- As a developer, I want to install Kotlin 1.9.20 for my project
- As a developer, I want installation to set the new version as default automatically
- As a developer, I want to be warned if a version doesn't exist before attempting install

**Acceptance Criteria**:
- Downloads verify SHA256 checksums (100% of the time)
- Installation completes in < 5 minutes for typical SDKs (< 2GB)
- Failed installations clean up temp files automatically
- Hooks execute only for Java (macOS) and JMC
- Installation sets symlink as default by default (configurable)
- Progress is logged for visibility

---

#### F3: SDK Removal
**Description**: Users can uninstall SDK versions they no longer need

**MCP Tools**:
- `uninstall_candidate` - Remove a specific version

**User Stories**:
- As a developer, I want to remove old Java versions to free up disk space
- As a developer, I want to be warned if I'm removing the current version
- As a developer, I want cleanup to be complete (no leftover files)

**Acceptance Criteria**:
- Uninstallation completes in < 10 seconds
- Warns if removing the current/default version
- Removes all files associated with the version
- Updates state to reflect removal
- Fails safely if version is in use

---

#### F4: Version Management
**Description**: Users can manage which version is the default

**MCP Tools**:
- `set_default_version` - Set an installed version as default

**User Stories**:
- As a developer, I want to switch my default Java version for all new terminals
- As a developer, I want to verify my current default before switching

**Acceptance Criteria**:
- Updates `current` symlink to point to new version
- Verifies version is installed before switching
- Completes in < 1 second
- Compatible with SDKMAN! CLI symlink format

---

#### F5: Installation Inspection
**Description**: Users can see what's currently installed

**MCP Tools**:
- `get_installed_versions` - List locally installed versions
- `get_current_version` - Get the active default version
- `get_platform_info` - Platform and configuration details

**MCP Resources**:
- `sdkman://installed` - All installed SDKs (JSON)
- `sdkman://installed/{candidate}` - Specific candidate details
- `sdkman://config` - Configuration and platform info (includes installation status)

**User Stories**:
- As a developer, I want to see what Java versions I have installed
- As a developer, I want to know which version is currently default
- As a developer, I want to see my platform details for debugging
- As a new user, I want to know if SDKMAN! is installed and ready to use

**Acceptance Criteria**:
- Reads filesystem directly (no separate database)
- Returns results in < 500ms
- Shows installed path for each version
- Indicates which version is current (default)
- `get_platform_info` includes SDKMAN! installation status (installed, not installed, partial)

---

#### F6: Utility Commands
**Description**: Additional utility commands for inspecting SDKMAN! state and configuration

**MCP Tools**:
- `get_candidate_home` - Get absolute path to an installed SDK version
- `get_sdkman_version` - Get SDKMAN! script and native version numbers
- `get_sdkman_config` - Get current SDKMAN! configuration settings

**User Stories**:
- As a developer, I want to know the exact path to a Java installation for configuring my IDE
- As a developer, I want to check which version of SDKMAN! I'm running
- As a developer, I want to see my SDKMAN! configuration settings (auto-answer, color mode, etc.)

**Acceptance Criteria for `get_candidate_home`**:
- Takes candidate name and version as parameters
- Returns absolute filesystem path (e.g., `/home/user/.sdkman/candidates/java/21.0.1-tem`)
- Returns error if candidate/version not installed
- Completes in < 100ms

**Acceptance Criteria for `get_sdkman_version`**:
- Returns both script version and native version
- Reads from SDKMAN! installation metadata
- Returns error if SDKMAN! not installed
- Completes in < 100ms

**Acceptance Criteria for `get_sdkman_config`**:
- Returns all configuration settings as key-value pairs
- Reads from `~/.sdkman/etc/config`
- Includes: sdkman_auto_answer, sdkman_auto_selfupdate, sdkman_insecure_ssl, sdkman_curl_connect_timeout, sdkman_curl_max_time, sdkman_beta_channel, sdkman_debug_mode, sdkman_colour_enable, sdkman_auto_env, sdkman_auto_complete
- Returns sensible defaults if config file doesn't exist
- Completes in < 100ms

**Note on `sdk env`**:
- `.sdkmanrc` support is marked as P2 (future enhancement)
- Not included in v1.0 as it requires shell session manipulation
- May be added in v2.0 with project configuration support

---

### 2.2 P1 Features (Nice-to-Have for v1.0, Required for v1.1)

#### F7: Batch Operations
**Description**: Install or remove multiple SDKs in one operation

**User Stories**:
- As a developer, I want to install "Java 21, Kotlin 1.9.20, Gradle 8.5" in one request
- As a DevOps engineer, I want to set up a complete environment with one command

**Acceptance Criteria**:
- Supports comma-separated or array input
- Executes sequentially with per-candidate locking
- Reports success/failure for each candidate
- Rolls back on critical failures (optional)

---

#### F8: Update Notifications
**Description**: Notify users when newer versions are available

**User Stories**:
- As a developer, I want to know when a new Java LTS is available
- As a developer, I want to see if my Gradle version is outdated

**Acceptance Criteria**:
- Compares installed versions to latest available
- Shows major/minor/patch version differences
- Respects user preferences (LTS only, stable only, etc.)

---

#### F9: Installation Progress
**Description**: Real-time progress updates during installation

**User Stories**:
- As a developer, I want to see download progress for large SDKs
- As a developer, I want to know what step is currently executing

**Acceptance Criteria**:
- Reports download percentage
- Shows current step (downloading, verifying, extracting, etc.)
- Updates at least every 5 seconds
- Works within MCP protocol constraints

---

### 2.3 P2 Features (Future Enhancements)

#### F10: Project Configuration (`.sdkmanrc` / `sdk env`)
**Description**: Auto-detect and install SDKs based on project files

**User Stories**:
- As a developer, I want SDKs installed automatically when I open a new project
- As a developer, I want to share SDK requirements with my team via `.sdkmanrc`

**Implementation**: Detect and parse `.sdkmanrc`, `build.gradle`, `pom.xml`, etc.

---

#### F11: Custom Repositories
**Description**: Support for custom SDKMAN! repositories

**User Stories**:
- As an enterprise user, I want to use our internal SDK mirror
- As a developer, I want to test beta versions from custom sources

**Implementation**: Configurable API endpoints

---

#### F12: Analytics & Recommendations
**Description**: Track usage and suggest optimizations

**User Stories**:
- As a developer, I want to see which SDKs I use most
- As a developer, I want recommendations for unused SDKs to remove

**Implementation**: Local analytics, privacy-preserving

---

## 3. Non-Functional Requirements

### 3.1 Performance

| Metric | Target | Measurement |
|--------|--------|-------------|
| List candidates | < 2s (P90) | API + cache |
| List versions | < 3s (P90) | API + cache |
| Install SDK (typical) | < 5 min | End-to-end |
| Uninstall SDK | < 10s | File operations |
| Set default version | < 1s | Symlink update |
| Idle memory usage | < 50MB | RSS |
| Active installation | < 200MB | Peak RSS |

### 3.2 Security

**Requirements**:
1. **Download Integrity**: MUST verify SHA256 checksums for 100% of downloads
2. **HTTPS Only**: MUST use HTTPS for all API calls (no HTTP fallback)
3. **Hook Safety**: MUST only execute hooks from official SDKMAN! API
4. **Path Validation**: MUST validate all paths during extraction to prevent traversal
5. **No Credential Storage**: MUST NOT store passwords or tokens
6. **Fail Closed**: MUST fail installation on security check failure (no skip option)

**Threat Model**:
- Man-in-the-middle attacks: Mitigated by HTTPS + checksum verification
- Malicious archives: Mitigated by SHA256 verification
- Hook injection: Mitigated by using only official API hooks
- Path traversal: Mitigated by path validation during extraction

### 3.3 Reliability

**Requirements**:
1. **Installation Success**: > 95% success rate (excluding network failures)
2. **Crash-Free**: > 99.5% of sessions complete without crashes
3. **Data Safety**: Failed installations MUST clean up temp files
4. **Idempotency**: Installing same version twice MUST be safe
5. **State Consistency**: State MUST always reflect filesystem reality

**Error Handling**:
- Network failures: Retry with exponential backoff (3 attempts)
- Disk space: Pre-check before download
- Concurrent installs: Per-candidate locking
- Invalid checksums: Fail immediately, remove partial download

### 3.4 Compatibility

**Requirements**:
1. **SDKMAN! Interoperability**: MUST work with existing `~/.sdkman` installations
2. **Directory Structure**: MUST maintain standard SDKMAN! layout
3. **Symlink Format**: MUST use compatible `current` symlink format
4. **Platform Support**:
   - Linux x64, ARM64 (native)
   - macOS x64 (Intel), ARM64 (Apple Silicon)
   - Windows via Git Bash
   - Windows via WSL

**Migration**: No migration needed - reads existing SDKMAN! installations

### 3.5 Usability

**Requirements**:
1. **Error Messages**: MUST be actionable (tell user what to do)
2. **Installation Time**: MUST complete within user attention span (< 5 min typical)
3. **Zero Config**: MUST work out-of-box (respects `SDKMAN_DIR` if set)
4. **Documentation**: MUST have quick-start guide and troubleshooting

---

## 4. User Stories & Workflows

### 4.0 First-Time Setup Workflow

**Story**: David is a new developer who wants to use SDKMAN! through Claude

```
David: "I want to use SDKMAN! to manage my Java installations"
Claude: [calls get_platform_info to check for existing installation]
Claude: "I see you don't have SDKMAN! installed yet. Would you like me to set it up for you?"

David: "Yes, please"
Claude: [calls install_sdkman]
Claude: "Installing SDKMAN! using the official installer...
  * Downloaded installer from https://get.sdkman.io
  * Executing installation script...
  * SDKMAN! installed successfully at ~/.sdkman
  * Shell configuration updated (restart your terminal or run: source ~/.sdkman/bin/sdkman-init.sh)

SDKMAN! is now ready! What would you like to install first?"

David: "Install Java 21"
Claude: [calls install_candidate]
Claude: "* Java 21 installed successfully
  * Set as default version
  Java 21 is now available at ~/.sdkman/candidates/java/current/bin/java"
```

### 4.1 Discovery Workflow

**Story**: Alice wants to find the latest Java LTS

```
Alice: "What Java versions are available?"
Claude: [calls list_versions for java]
Claude: "Here are the available Java versions:
  - 21.0.1 (LTS, current default)
  - 17.0.9 (LTS)
  - 11.0.21 (LTS)
  - 22-ea (Early Access)
  ...
You currently have Java 11.0.21 installed. Would you like to upgrade to 21?"

Alice: "Yes, install Java 21"
Claude: [calls install_candidate with java, 21]
Claude: "Installing Java 21... * Downloaded and verified (SHA256 checksum OK)
  * Installed to ~/.sdkman/candidates/java/21.0.1-tem
  * Set as default version
Java 21 is now ready to use!"
```

### 4.2 Installation Workflow

**Story**: Bob needs to set up Kotlin for a new project

```
Bob: "Install Kotlin 1.9.20"
Claude: [calls validate_version]
Claude: [calls install_candidate]
Claude: "* Kotlin 1.9.20 installed successfully
  Path: ~/.sdkman/candidates/kotlin/1.9.20
  This is now your default Kotlin version."

Bob: "What Kotlin versions do I have installed?"
Claude: [calls get_installed_versions for kotlin]
Claude: "You have:
  - 1.9.20 (current)
  - 1.8.22"
```

### 4.3 Version Management Workflow

**Story**: Carol needs to switch Java versions for different projects

```
Carol: "What Java versions do I have?"
Claude: [calls get_installed_versions for java]
Claude: "You have:
  - 21.0.1-tem (current)
  - 17.0.9-tem
  - 11.0.21-tem"

Carol: "Set Java 17 as default"
Claude: [calls set_default_version]
Claude: "* Java 17.0.9-tem is now the default version
  New terminal sessions will use Java 17."

Carol: "Where is Java 17 installed on my system?"
Claude: [calls get_candidate_home for java 17.0.9-tem]
Claude: "Java 17.0.9-tem is installed at:
  /home/carol/.sdkman/candidates/java/17.0.9-tem

You can configure your IDE to use this path for the JDK."
```

### 4.4 Cleanup Workflow

**Story**: Alice wants to free up disk space

```
Alice: "What's taking up space in my SDK directory?"
Claude: [calls get_installed_versions for all candidates]
Claude: "You have these SDKs installed:
  - Java: 3 versions (21, 17, 11) ≈ 1.2GB
  - Kotlin: 2 versions (1.9.20, 1.8.22) ≈ 500MB
  - Gradle: 4 versions (8.5, 8.4, 8.3, 7.6) ≈ 800MB"

Alice: "Remove Gradle 7.6 and 8.3"
Claude: [calls uninstall_candidate for each]
Claude: "* Removed Gradle 7.6
  * Removed Gradle 8.3
  Freed ≈ 400MB of disk space"
```

### 4.5 Error Handling Workflow

**Story**: Installation fails due to network issues

```
Bob: "Install Java 21"
Claude: [calls install_candidate]
Claude: "Download failed (connection timeout). Retrying... (attempt 2/3)
  Download failed again. Retrying... (attempt 3/3)

  x Installation failed: Unable to download Java 21 after 3 attempts.

  Troubleshooting:
  - Check your internet connection
  - Try again in a few minutes
  - Visit https://sdkman.io/status to check service status

  Temporary files have been cleaned up automatically."
```

---

## 5. Out of Scope (Non-Goals)

### 5.1 Explicitly NOT Supported in v1.0

1. **Temporary Version Switching (`sdk use`)**
   - Reason: Shell session-scoped, not applicable to MCP context
   - Alternative: Use `set_default_version` for persistent switching

2. **Offline Mode**
   - Reason: Requires maintaining local candidate database
   - Alternative: Users should use official SDKMAN! CLI for offline scenarios

3. **Broadcast Messages**
   - Reason: Not applicable to MCP context (no interactive terminal)
   - Alternative: Could be added as notification resource in future

4. **Native Windows Support**
   - Reason: Complexity of Windows symlinks, directory junctions
   - Alternative: Full support via Git Bash and WSL

5. **Custom Candidate Development**
   - Reason: SDKMAN! server-side feature, not client concern
   - Alternative: Use official SDKMAN! processes for candidate submission

6. **Manual Shell Integration (for MCP-only installs)**
   - Reason: Official SDKMAN! installer handles shell integration automatically
   - Note: If user uses `install_sdkman` tool, shell integration is automatic via official installer

7. **Interactive Installation Prompts**
   - Reason: MCP is not an interactive terminal
   - Alternative: Use sensible defaults (e.g., always set as default unless specified)

### 5.2 Future Considerations (Maybe v2.0+)

1. **Project-based SDK Configuration** (`.sdkmanrc` support)
2. **Custom Repository Mirrors** (enterprise use case)
3. **SDK Usage Analytics** (privacy-preserving, local only)
4. **Automated Updates** (background update checking)
5. **Multi-user Support** (system-wide installations)

---

## 6. Success Metrics & KPIs

### 6.1 Technical KPIs

**Reliability**:
- Installation success rate: > 95% (target: 98%)
- API call success rate: > 99%
- Crash-free sessions: > 99.5%
- Checksum verification: 100% (no exceptions)

**Performance**:
- P50 API response time: < 1s
- P90 API response time: < 3s
- P99 API response time: < 5s
- Installation time (Kotlin 1.9.20): < 60s
- Installation time (Java 21): < 3 minutes

**Quality**:
- Test coverage: > 80%
- Zero critical security vulnerabilities
- Zero data loss incidents
- < 5 bugs per 1000 installations

### 6.2 User Experience KPIs

**Adoption**:
- 100+ GitHub stars in 3 months
- 500+ installations in 6 months
- 10+ community contributors
- Featured in SDKMAN! official docs

**Satisfaction**:
- > 90% positive user feedback
- < 10% uninstall rate
- Active community engagement (issues, PRs, discussions)
- Low support burden (< 5% of users need help)

**Usage Patterns**:
- Most common operations: list_versions, install_candidate
- Average session length: < 2 minutes
- Repeat usage rate: > 50% (users come back)

### 6.3 Business KPIs

**Ecosystem Growth**:
- Drives SDKMAN! adoption (more MCP users = more SDKMAN! users)
- Increases AI assistant usage for development workflows
- Demonstrates MCP value for developer tools

**Community Impact**:
- Becomes reference implementation for build tool MCP servers
- Inspires similar tools (Maven MCP, npm MCP, etc.)
- Strengthens Rust MCP ecosystem

---

## 7. Testing Strategy

### 7.1 Test Focus Candidates

**Primary Test Candidates**:
1. **Java** - Complex installation (hooks on macOS), multiple vendors, most used
2. **Kotlin** - Simple installation, no hooks, good baseline test

**Why These Two**:
- Java tests complex scenarios (hooks, platform-specific, vendors)
- Kotlin tests simple scenarios (universal binary, no hooks)
- Together they cover 80% of installation patterns
- Both are popular and well-maintained

### 7.2 Platform Testing Matrix

**CI Matrix** (GitHub Actions):

| Platform | Architecture | Test Candidates | Hook Execution |
|----------|--------------|-----------------|----------------|
| Ubuntu 22.04 | x64 | Java, Kotlin | No hooks |
| Ubuntu 22.04 | ARM64 (emulated) | Kotlin | No hooks |
| macOS 13 (Intel) | x64 | Java, Kotlin | Java hooks |
| macOS 14 (M1) | ARM64 | Java, Kotlin | Java hooks |
| Windows Server 2022 | x64 (Git Bash) | Kotlin | No hooks |
| Windows Server 2022 | x64 (WSL) | Java, Kotlin | No hooks |

**Test Types**:
- **Unit Tests**: Platform detection, API parsing, checksum verification
- **Integration Tests**: Full installation flow, state management
- **E2E Tests**: MCP protocol, tool invocations, resources

### 7.3 Test Scenarios

**Happy Path**:
1. - Install SDKMAN! (first-time setup)
2. - List all candidates
3. - List Java versions
4. - Install Java 21
5. - Install Kotlin 1.9.20
6. - Set Java 17 as default
7. - Uninstall old version
8. - Get installed versions

**Error Scenarios**:
1. x Install non-existent version (should fail gracefully)
2. x Network failure during download (should retry)
3. x Disk space exhausted (should fail with clear message)
4. x Checksum mismatch (should fail immediately)
5. x Concurrent installation of same candidate (should lock)

**Edge Cases**:
1. No `~/.sdkman` directory (first-time user, should offer to run official installer)
2. Empty `~/.sdkman` directory (partial install, should offer to reinstall)
3. Existing SDKMAN! CLI installation (compatibility, should detect and skip installer)
4. Corrupted symlink (should repair)
5. Missing `current` symlink (should create)
6. SDKMAN_DIR points to non-existent location (should error with guidance)
7. Network failure during installer download (should retry with guidance)

---

## 8. Technical Constraints

### 8.1 Dependencies

**Hard Dependencies** (cannot change):
- Rust 1.70+ (for async/await stabilization)
- SDKMAN! backend APIs (broker, candidates, hooks)
- Unix-like filesystem (for symlinks)

**Soft Dependencies** (configurable):
- `rust-mcp-sdk` 0.2+ (can switch to official SDK if needed)
- Tokio (can use other async runtimes)
- reqwest (can use hyper directly)

### 8.2 API Limitations

**SDKMAN! API Constraints**:
- No authentication (public APIs)
- No rate limiting (currently)
- Text-based responses for some endpoints (need parsing)
- No official OpenAPI spec (manual implementation)

**Mitigation**:
- Implement client-side rate limiting (good citizen)
- Cache aggressively where appropriate
- Robust parsing with fallbacks
- Version API calls to detect changes

### 8.3 Platform Limitations

**Git Bash**:
- Limited symlink support (may require copying)
- Different path conventions (may need conversion)
- Slower file operations than native Linux

**WSL**:
- File system performance (Windows drives are slower)
- Path interop between Windows and WSL
- Potential permission issues

**Mitigation**:
- Detect environment and adjust strategy
- Provide clear error messages for platform-specific issues
- Test thoroughly in CI matrix

---

## 9. Risks & Mitigation

### 9.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| SDKMAN! API changes without notice | Medium | High | Version API calls, comprehensive error handling, engage with SDKMAN! team |
| Platform-specific bugs in production | High | Medium | Comprehensive CI matrix, early beta testing, user feedback channel |
| Hook execution failures | Medium | Medium | Selective execution (Java/JMC only), timeout protection, fallback to no-hook install |
| rust-mcp-sdk breaking changes | Low | High | Pin versions in Cargo.toml, maintain fork if necessary |
| Checksum mismatches (CDN issues) | Low | Medium | Immediate failure with clear error, manual retry option |

### 9.2 Product Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Low user adoption | Medium | High | Marketing, SDKMAN! official endorsement, excellent docs |
| SDKMAN! team doesn't endorse | Low | High | Early collaboration, follow best practices, community feedback |
| Users prefer CLI over MCP | Medium | Medium | Focus on AI-first workflows, demonstrate value, iterate on UX |
| Competition from other tools | Low | Low | First-mover advantage, official status, quality focus |

### 9.3 Resource Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Timeline delays (> 10 weeks) | Medium | Low | Phased approach, MVP focus, can ship with subset of features |
| Maintenance burden | Medium | Medium | Good architecture, comprehensive tests, community contributors |
| Platform testing limitations | Medium | Medium | GitHub Actions provide all platforms, fallback to manual testing |

---

## 10. Release Plan

### 10.1 Alpha (Internal Testing)

**Timeline**: End of Phase 4 (Week 6)

**Features**:
- Discovery tools working
- Basic installation (Kotlin)
- State management

**Audience**:
- Project team only
- Selected SDKMAN! contributors

**Success Criteria**:
- Can install Kotlin successfully on Linux
- No critical bugs in discovery
- Installation completes in < 2 minutes

### 10.2 Beta (Public Testing)

**Timeline**: End of Phase 7 (Week 9)

**Features**:
- All 15 tools implemented (including SDKMAN! installation and utility commands)
- Java installation with hooks
- CI matrix passing

**Audience**:
- SDKMAN! community
- MCP early adopters
- Rust developers

**Success Criteria**:
- > 50 beta testers
- > 80% report positive experience
- < 10 critical bugs discovered

**Distribution**:
- GitHub releases (pre-release)
- Cargo install (--pre flag)
- Docker image (optional)

### 10.3 v1.0 (General Availability)

**Timeline**: End of Phase 8 (Week 10)

**Features**:
- All P0 features complete (15 tools, 3 resources)
- Documentation finalized
- CI/CD pipeline established

**Audience**:
- General public
- Featured in SDKMAN! website
- Promoted in MCP community

**Success Criteria**:
- Zero critical bugs
- Test coverage > 80%
- Passes all platform tests
- SDKMAN! team endorsement

**Distribution**:
- GitHub releases
- Cargo install
- Pre-built binaries (Linux, macOS, Windows)
- Docker image
- Homebrew formula (optional)

---

## 11. Maintenance & Support

### 11.1 Support Channels

**User Support**:
- GitHub Issues (bug reports, feature requests)
- GitHub Discussions (questions, community support)
- SDKMAN! Slack (integration with existing community)

**Response SLAs**:
- Critical bugs: < 24 hours (initial response)
- Normal bugs: < 72 hours
- Feature requests: < 1 week
- Questions: < 48 hours (community-driven)

### 11.2 Update Cadence

**Patch Releases** (x.y.Z):
- Frequency: As needed for critical bugs
- Examples: Security fixes, crash fixes, data loss prevention

**Minor Releases** (x.Y.0):
- Frequency: Monthly (if features available)
- Examples: New tools, performance improvements, new platforms

**Major Releases** (X.0.0):
- Frequency: Yearly (or when breaking changes needed)
- Examples: Protocol changes, architecture refactor, major features

### 11.3 Backward Compatibility

**Promises**:
- MCP tool signatures remain stable within major version
- Configuration format remains stable within major version
- State directory structure remains compatible with SDKMAN! CLI

**Breaking Changes**:
- Require major version bump
- Documented in CHANGELOG
- Migration guide provided
- Deprecation warnings in advance (1 minor version minimum)

---

## Appendix A: Glossary

- **SDK**: Software Development Kit (e.g., Java, Kotlin, Gradle)
- **MCP**: Model Context Protocol (enables AI assistants to use tools)
- **SDKMAN!**: The Software Development Kit Manager (https://sdkman.io)
- **Candidate**: An SDK managed by SDKMAN!
- **Version**: A specific release of a candidate (e.g., Java 21.0.1)
- **Current/Default**: The version symlinked as `current` (used by default)
- **Hook**: Bash script executed during installation for special handling
- **Platform ID**: SDKMAN! identifier for OS/architecture (e.g., linux-x64)

## Appendix B: References

- SDKMAN! Website: https://sdkman.io
- SDKMAN! Usage: https://sdkman.io/usage/
- Model Context Protocol: https://modelcontextprotocol.io
- rust-mcp-sdk: https://github.com/rust-mcp-stack/rust-mcp-sdk
- SDKMAN! Backend APIs:
  - Broker: https://github.com/sdkman/sdkman-broker-2
  - Candidates: https://github.com/sdkman/sdkman-candidates
  - Hooks: https://github.com/sdkman/sdkman-hooks

---

## Document Control

**Approval History**:
- Draft v1.0 - 2025-12-11 - Initial draft
- Awaiting approval from SDKMAN! team

**Change Log**:
- 2025-12-11: Initial document creation with comprehensive PRD
