# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-04

## Test Frameworks

### TypeScript/Client

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Vitest | Inline in `vite.config.ts` | Ready |
| Integration | Vitest | Inline in `vite.config.ts` | Ready |
| Component | Vitest + React Testing Library (planned) | TBD | Not started |
| E2E | Not selected | TBD | Not started |

### Rust/Server

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)]` inline | In use |
| Integration | Rust built-in | `tests/` directory | In use |
| E2E | Not selected | TBD | Not started |

### Rust/Monitor

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)]` inline | In use |
| Integration | Rust built-in | `tests/` directory with `serial_test` crate | In use (Phase 11+) |
| CLI | Subprocess-based binary execution | `tests/` with `std::process::Command` | In use (Phase 12) |
| E2E | Not selected | TBD | Not started |

### GitHub Actions

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Composite Action | YAML-based action composition | `.github/actions/vibetea-monitor/action.yml` | In use (Phase 6) |
| Workflow Integration | Example workflows with monitoring | `.github/workflows/ci-with-monitor.yml` | Available |

### Running Tests

#### TypeScript/Client

| Command | Purpose |
|---------|---------|
| `npm test` | Run all unit tests once |
| `npm run test:watch` | Run tests in watch mode (re-run on file changes) |
| `npm run typecheck` | Run TypeScript type checking |
| `npm run format:check` | Check code formatting without fixing |
| `npm run lint` | Run ESLint |

#### Rust/Server and Monitor

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests in the workspace |
| `cargo test --lib` | Run library unit tests only |
| `cargo test --test '*'` | Run integration tests only |
| `cargo test -- --nocapture` | Run tests with println output |
| `cargo test -- --test-threads=1` | Run tests sequentially (prevents env var interference) |
| `cargo test -p vibetea-monitor privacy` | Run privacy module tests |
| `cargo test --test privacy_test` | Run privacy integration tests |
| `cargo test --test env_key_test` | Run environment key loading tests (Phase 11, 21 tests) |
| `cargo test --test key_export_test` | Run export-key CLI tests (Phase 12, 12 tests) |
| `cargo test -p vibetea-monitor crypto` | Run crypto module tests |
| `cargo test -p vibetea-monitor sender` | Run sender module tests |
| `cargo test -p vibetea-monitor project_tracker` | Run project tracker module tests (Phase 11, 69 tests) |

**Important**: Monitor tests run with `--test-threads=1` in CI to prevent environment variable interference:

```bash
cargo test --package vibetea-monitor -- --test-threads=1
```

## Test Organization

### TypeScript/Client

```
client/
├── src/
│   ├── __tests__/              # Test directory
│   │   ├── App.test.tsx        # App component tests
│   │   ├── events.test.ts      # Event type tests
│   │   └── formatting.test.ts  # Formatting utility tests
│   ├── components/
│   ├── hooks/
│   ├── utils/
│   └── types/
```

**Test organization strategy**: Tests co-located with source via `__tests__/` directory at feature level.

### Rust/Server and Monitor

```
server/src/
├── main.rs
├── lib.rs
├── config.rs              # Configuration with tests at EOF
├── auth.rs                # Auth with 30+ tests at EOF
├── error.rs               # Error types with some tests
├── types.rs               # Type definitions
├── routes.rs              # Route handlers
├── broadcast.rs           # Broadcasting logic
└── rate_limit.rs          # Rate limiting

monitor/src/
├── main.rs
├── lib.rs
├── config.rs              # Configuration with tests
├── crypto.rs              # Cryptography (Phase 6) with tests
├── sender.rs              # HTTP sender (Phase 6) with tests
├── privacy.rs             # Privacy pipeline (Phase 5) with tests
├── parser.rs              # JSONL parsing
└── trackers/
    ├── agent_tracker.rs   # Agent spawn detection (Phase 4) with 28+ tests
    ├── skill_tracker.rs   # Skill invocation tracking (Phase 5) with 20+ tests
    ├── todo_tracker.rs    # Todo list monitoring (Phase 6) with 79 tests
    ├── stats_tracker.rs   # Token usage tracking (Phase 8-10) with 40+ tests
    └── project_tracker.rs # Project session tracking (Phase 11) with 69 tests
```

### Rust/Monitor Directory Structure

```
monitor/
├── src/
│   ├── config.rs               # Config module with inline tests
│   ├── error.rs                # Error module with inline tests
│   ├── types.rs                # Types module with inline tests
│   ├── watcher.rs              # File watching implementation
│   ├── parser.rs               # JSONL parser implementation
│   ├── privacy.rs              # Privacy pipeline with 38 inline unit tests
│   ├── crypto.rs               # Ed25519 crypto operations with 14 inline unit tests
│   ├── sender.rs               # HTTP sender with 8 inline unit tests
│   ├── lib.rs                  # Library entrypoint
│   ├── main.rs                 # Binary entrypoint (CLI)
│   └── trackers/
│       ├── agent_tracker.rs    # Agent spawn detection (Phase 4) with 28+ inline tests
│       ├── skill_tracker.rs    # Skill invocation tracking (Phase 5) with 20+ inline tests
│       ├── todo_tracker.rs     # Todo list monitoring (Phase 6) with 79 inline tests
│       ├── stats_tracker.rs    # Token usage tracking (Phase 8-10) with 40+ inline tests
│       └── project_tracker.rs  # Project session tracking (Phase 11) with 69 inline tests
└── tests/
    ├── env_key_test.rs         # 21 integration tests for env var key loading (Phase 11)
    ├── privacy_test.rs         # 17 integration tests for privacy compliance
    ├── sender_recovery_test.rs # Integration tests for error recovery
    └── key_export_test.rs      # 12 integration tests for export-key subcommand (Phase 12, 698 lines)
```

### GitHub Actions Composite Action Structure (Phase 6)

```
.github/
└── actions/
    └── vibetea-monitor/
        └── action.yml          # Composite action for monitor deployment
```

The action (`action.yml`) is a reusable GitHub Actions artifact that encapsulates:
- Binary download from releases
- Environment variable configuration
- Monitor startup and graceful shutdown
- Output parameters for downstream steps

### Test File Location Strategy

**TypeScript**: Co-located tests in `__tests__/` directory

| Source File | Test File |
|-------------|-----------|
| `src/types/events.ts` | `src/__tests__/events.test.ts` |
| `src/hooks/useEventStore.ts` | `src/__tests__/useEventStore.test.ts` (planned) |
| `src/hooks/useWebSocket.ts` | `src/__tests__/useWebSocket.test.ts` (planned) |
| `src/hooks/useSessionTimeouts.ts` | `src/__tests__/useSessionTimeouts.test.ts` (planned) |
| `src/components/ConnectionStatus.tsx` | `src/__tests__/ConnectionStatus.test.tsx` (planned) |
| `src/components/TokenForm.tsx` | `src/__tests__/TokenForm.test.tsx` (planned) |
| `src/components/EventStream.tsx` | `src/__tests__/EventStream.test.tsx` (planned) |
| `src/components/Heatmap.tsx` | `src/__tests__/Heatmap.test.tsx` (planned) |
| `src/components/SessionOverview.tsx` | `src/__tests__/SessionOverview.test.tsx` (planned) |
| `src/utils/formatting.ts` | `src/__tests__/formatting.test.ts` |
| `src/App.tsx` | `src/__tests__/App.test.tsx` (planned) |

**Rust**:
- Unit tests inline in same module (`#[cfg(test)] mod tests`)
- Integration tests in separate files in `tests/` directory with `_test.rs` suffix

**GitHub Actions**:
- Composite action stored in `.github/actions/{action-name}/action.yml`
- Example workflows stored in `.github/workflows/{workflow-name}.yml`

## Test Patterns

### TypeScript Unit Tests

**Pattern: Describe/It structure with Arrange/Act/Assert**

```typescript
import { describe, it, expect } from 'vitest';
import type { VibeteaEvent, EventType } from '../types/events';

describe('Event Types', () => {
  it('should create a valid session event', () => {
    // Arrange
    const event: VibeteaEvent<'session'> = {
      id: 'evt_test123456789012345',
      source: 'test-source',
      timestamp: new Date().toISOString(),
      type: 'session',
      payload: {
        sessionId: '123e4567-e89b-12d3-a456-426614174000',
        action: 'started',
        project: 'test-project',
      },
    };

    // Act
    // (no action needed for validation tests)

    // Assert
    expect(event.type).toBe('session');
    expect(event.payload.action).toBe('started');
  });
});
```

### Inline Module Unit Tests (Rust - Phase 11)

New pattern for tracker modules with comprehensive unit test coverage:

#### File: `monitor/src/trackers/project_tracker.rs` (1,822 lines, 69 tests)

Key features of project_tracker tests:

1. **Comprehensive coverage**: 69 unit tests covering:
   - Path slug parsing (8 tests) - handles standard paths, nested dirs, edge cases
   - Summary event detection (18 tests) - various JSON formats and edge cases
   - Event creation (4 tests) - active/inactive sessions, empty strings, unicode
   - Session ID extraction (8 tests) - valid/invalid UUIDs, wrong format
   - Error types (2 tests) - Display impl, Debug impl
   - Configuration (2 tests) - defaults, cloning
   - Tracker creation (3 tests) - directory existence, valid paths
   - File watching (15 tests) - new files, modifications, ignoring non-jsonl, invalid UUIDs
   - Manual scanning (3 tests) - scan_projects method, multiple projects, hidden directories

2. **Test organization**: Tests grouped by feature with section markers
   ```rust
   #[cfg(test)]
   mod tests {
       // ===================================================================
       // T230: Unit test for project path slug parsing
       // ===================================================================

       #[test]
       fn parse_project_slug_standard_path() { ... }
   }
   ```

3. **Async test support**: Uses `#[tokio::test]` for async file watching tests
   ```rust
   #[tokio::test]
   async fn test_tracker_detects_new_session_file() {
       let temp_dir = TempDir::new().expect("Failed to create temp dir");
       // Test with file system isolation
   }
   ```

4. **Test isolation**: Uses `tempfile::TempDir` for file system isolation
   ```rust
   use tempfile::TempDir;

   let temp_dir = TempDir::new()?;
   // TempDir automatically cleaned up when dropped
   ```

5. **Constants for test data**:
   ```rust
   const ACTIVE_SESSION: &str = r#"{"type": "user", "message": "hello"}
   {"type": "assistant", "message": "hi there"}
   "#;

   const COMPLETED_SESSION: &str = r#"{"type": "user", "message": "hello"}
   {"type": "assistant", "message": "hi there"}
   {"type": "summary", "summary": "Session completed"}
   "#;
   ```

### Integration Tests - Environment Variable Handling (Rust - Phase 11)

New integration test pattern for environment variable loading with proper test isolation:

#### File: `monitor/tests/env_key_test.rs` (21 tests)

```rust
//! Integration tests for environment variable key loading.
//!
//! These tests verify FR-001 (load Ed25519 private key from `VIBETEA_PRIVATE_KEY` env var),
//! FR-002 (env var takes precedence over file), FR-004 (clear error messages),
//! FR-005 (whitespace trimming), FR-021 (standard Base64 RFC 4648),
//! FR-022 (validate 32-byte key length), and FR-027/FR-028 (round-trip verification).
//!
//! # Important Notes
//!
//! These tests modify environment variables and MUST be run with `--test-threads=1`
//! or use the `serial_test` crate to prevent interference between tests.

use base64::prelude::*;
use ed25519_dalek::Verifier;
use serial_test::serial;
use std::env;
use tempfile::TempDir;
use vibetea_monitor::crypto::{Crypto, KeySource};

// =============================================================================
// Test Helpers
// =============================================================================

/// Environment variable name for the private key.
const ENV_VAR_NAME: &str = "VIBETEA_PRIVATE_KEY";

/// RAII guard that saves and restores an environment variable.
struct EnvGuard {
    name: String,
    original: Option<String>,
}

impl EnvGuard {
    fn new(name: &str) -> Self {
        let original = env::var(name).ok();
        Self {
            name: name.to_string(),
            original,
        }
    }

    fn set(&self, value: &str) {
        env::set_var(&self.name, value);
    }

    fn remove(&self) {
        env::remove_var(&self.name);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(val) => env::set_var(&self.name, val),
            None => env::remove_var(&self.name),
        }
    }
}

// =============================================================================
// FR-001: Load Ed25519 private key seed from VIBETEA_PRIVATE_KEY env var
// =============================================================================

/// Verifies that a valid base64-encoded 32-byte seed can be loaded from
/// the `VIBETEA_PRIVATE_KEY` environment variable.
///
/// FR-001: Load Ed25519 private key seed from `VIBETEA_PRIVATE_KEY` env var
/// as base64-encoded string.
#[test]
#[serial]
fn load_valid_base64_key_from_env() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    // Set env var and test loading...
}
```

Key patterns in integration tests:

1. **RAII Pattern**: `EnvGuard` automatically restores environment variables on drop
2. **#[serial] Macro**: Ensures sequential test execution to prevent interference
3. **Module-level organization**: Tests grouped by feature requirement (FR-###)
4. **Clear documentation**: Each test documents which requirements it verifies
5. **Test helpers**: Centralized helper functions at top of file

### Integration Tests - CLI Command Execution (Rust - Phase 12)

New CLI testing pattern for subprocess-based command testing:

#### File: `monitor/tests/key_export_test.rs` (12 tests, 698 lines)

```rust
//! Integration tests for the `export-key` subcommand.
//!
//! These tests verify the following requirements:
//!
//! - **FR-003**: Monitor MUST provide `export-key` subcommand to output ONLY the base64-encoded
//!   private key followed by a single newline (no additional text), enabling direct piping to
//!   clipboard or secret management tools.
//!
//! - **FR-023**: All diagnostic and error messages from `export-key` MUST go to stderr; only the
//!   key itself goes to stdout.
//!
//! - **FR-026**: Exit codes: 0 for success, 1 for configuration error (invalid env var, missing
//!   key), 2 for runtime error.
//!
//! - **FR-027**: Integration tests MUST verify that a key exported with `export-key` can be loaded
//!   via `VIBETEA_PRIVATE_KEY`.
//!
//! - **FR-028**: Integration tests MUST verify round-trip: generate key, export, load from env var,
//!   verify signing produces valid signatures.
//!
//! # Important Notes
//!
//! These tests modify environment variables and MUST be run with `--test-threads=1`
//! or use the `serial_test` crate to prevent interference between tests.

use base64::prelude::*;
use ed25519_dalek::Verifier;
use serial_test::serial;
use std::env;
use std::process::Command;
use tempfile::TempDir;
use vibetea_monitor::crypto::Crypto;

// =============================================================================
// Test Helpers
// =============================================================================

/// Environment variable name for the private key.
const ENV_VAR_NAME: &str = "VIBETEA_PRIVATE_KEY";

/// Exit code for successful execution.
const EXIT_SUCCESS: i32 = 0;

/// Exit code for configuration errors (invalid env var, missing key).
const EXIT_CONFIG_ERROR: i32 = 1;

/// Exit code for runtime errors.
#[allow(dead_code)]
const EXIT_RUNTIME_ERROR: i32 = 2;

/// RAII guard that saves and restores an environment variable.
struct EnvGuard {
    name: String,
    original: Option<String>,
}

impl EnvGuard {
    fn new(name: &str) -> Self {
        let original = env::var(name).ok();
        Self {
            name: name.to_string(),
            original,
        }
    }

    fn set(&self, value: &str) {
        env::set_var(&self.name, value);
    }

    fn remove(&self) {
        env::remove_var(&self.name);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(val) => env::set_var(&self.name, val),
            None => env::remove_var(&self.name),
        }
    }
}

/// Builds and returns the path to the vibetea-monitor binary.
fn get_monitor_binary_path() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let target_dir = std::path::Path::new(manifest_dir)
        .parent()
        .expect("Should have parent directory")
        .join("target")
        .join("debug")
        .join("vibetea-monitor");
    target_dir.to_string_lossy().to_string()
}

/// Runs the vibetea-monitor export-key command with the given path.
fn run_export_key_command(key_path: &std::path::Path) -> std::process::Output {
    Command::new(get_monitor_binary_path())
        .arg("export-key")
        .arg("--path")
        .arg(key_path.to_string_lossy().as_ref())
        .output()
        .expect("Failed to execute vibetea-monitor binary")
}

// =============================================================================
// FR-027/FR-028: Round-trip verification with export-key command
// =============================================================================

/// Verifies the complete round-trip using the export-key command:
/// 1. Generate key with `Crypto::generate()`
/// 2. Save key to file
/// 3. Export via `export-key` command
/// 4. Load via `VIBETEA_PRIVATE_KEY` environment variable
/// 5. Sign and verify
///
/// **Covers:** FR-003, FR-027, FR-028
#[test]
#[serial]
fn roundtrip_generate_export_command_import_sign_verify() {
    let guard = EnvGuard::new(ENV_VAR_NAME);
    guard.remove();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Step 1 & 2: Generate and save keypair
    let original_crypto = Crypto::generate();
    original_crypto
        .save(temp_dir.path())
        .expect("Failed to save keypair");
    let original_pubkey = original_crypto.public_key_base64();

    // Step 3: Export via export-key command
    let output = run_export_key_command(temp_dir.path());

    // Step 4: Command should succeed with exit code 0
    assert!(
        output.status.success(),
        "export-key should exit with code 0, got: {:?}\nstderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    // Step 5: Verify exported key matches original seed and can be loaded
    let exported_key =
        String::from_utf8(output.stdout.clone()).expect("stdout should be valid UTF-8");
    let exported_key_trimmed = exported_key.trim();

    let original_seed = original_crypto.seed_base64();
    assert_eq!(
        exported_key_trimmed, original_seed,
        "Exported key should match the original seed"
    );

    guard.set(exported_key_trimmed);
    let result = Crypto::load_from_env();
    assert!(result.is_ok(), "Should load exported key from env var");

    let (loaded_crypto, _) = result.unwrap();
    let loaded_pubkey = loaded_crypto.public_key_base64();
    assert_eq!(original_pubkey, loaded_pubkey);
}

/// Verifies that signatures from the original key and the key loaded via
/// export-key are identical (Ed25519 is deterministic).
///
/// **Covers:** FR-027, FR-028
#[test]
#[serial]
fn roundtrip_export_command_signatures_are_identical() {
    // ... test code
}
```

Test organization in key_export_test.rs (12 tests total):

1. **Round-trip tests** (2 tests):
   - `roundtrip_generate_export_command_import_sign_verify` - Full cycle verification
   - `roundtrip_export_command_signatures_are_identical` - Deterministic signing

2. **Output format tests** (2 tests):
   - `export_key_output_format_base64_with_single_newline` - Exact format (base64 + \n)
   - `export_key_output_is_valid_base64_32_bytes` - Base64 validity and length

3. **Stream separation tests** (2 tests):
   - `export_key_diagnostics_go_to_stderr` - Diagnostics on stderr only
   - `export_key_error_messages_go_to_stderr` - Error messages on stderr only

4. **Error handling tests** (3 tests):
   - Missing key error with exit code 1
   - Error message clarity tests
   - Path argument validation

5. **Default path behavior** (1 test):
   - Uses default key directory when no --path provided

Key conventions in CLI testing:
- **Subprocess execution**: Tests spawn the actual binary using `Command::new()`
- **Exit code validation**: Tests verify expected exit codes (0 success, 1 config error, 2 runtime error)
- **Output stream separation**: Verify output goes to correct stream (stdout for data, stderr for diagnostics)
- **Format validation**: Tests verify output format exactly (e.g., base64 key + single newline)
- **Base64 validation**: Exported keys are verified to decode to exactly 32 bytes
- **Round-trip verification**: Keys exported via command can be loaded and used for signing

### Integration Tests - Privacy Compliance (Rust)

File: `monitor/tests/privacy_test.rs` (17 tests)

Tests verify privacy guarantees across different event types and sensitive tools:

```rust
//! Privacy compliance test suite for VibeTea Monitor.
//!
//! These tests validate Constitution I (Privacy by Design) by ensuring
//! no sensitive data is ever present in processed events.

use std::collections::HashSet;
use uuid::Uuid;
use vibetea_monitor::privacy::{extract_basename, PrivacyConfig, PrivacyPipeline};
use vibetea_monitor::types::{EventPayload, SessionAction, ToolStatus};

#[test]
fn no_full_paths_in_tool_events() {
    let pipeline = PrivacyPipeline::new(PrivacyConfig::new(None));

    let payload = EventPayload::Tool {
        session_id: Uuid::nil(),
        tool: "Read".to_string(),
        status: ToolStatus::Completed,
        context: Some("/home/user/projects/secret/src/auth.rs".to_string()),
        project: Some("my-project".to_string()),
    };

    const description = getEventDescription(event);
    expect(description).toBe('Activity pattern: 3 hours tracked');
  });

    if let EventPayload::Tool { context, .. } = &result {
        assert_eq!(context.as_deref(), Some("auth.rs"));
    }
}
```

### Integration Tests - Error Recovery (Rust)

File: `monitor/tests/sender_recovery_test.rs`

Tests verify sender handles errors gracefully and recovers:

```rust
//! Integration tests for sender recovery behavior.
//!
//! These tests verify that the sender correctly handles error scenarios
//! and recovers gracefully, particularly around oversized events.

#[tokio::test]
async fn test_oversized_event_does_not_block_normal_events() {
    let mock_server = MockServer::start().await;
    // Test setup...

    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);
}
```

### GitHub Actions Integration Testing (Phase 5+6)

#### Example Workflow Integration (Phase 5)

File: `.github/workflows/ci-with-monitor.yml` (Example workflow)

The example workflow demonstrates how to test the monitor in CI/CD environments:

```yaml
# .github/workflows/ci-with-monitor.yml
name: CI with VibeTea Monitoring

# Tests demonstrate:
# 1. Binary download from releases (or build from source)
# 2. Background process execution with environment variable configuration
# 3. Non-blocking operation (monitor failures don't fail workflow)
# 4. Graceful shutdown with SIGTERM (flushes buffered events)
# 5. Private key via GitHub secrets (VIBETEA_PRIVATE_KEY)
# 6. Server URL via GitHub secrets (VIBETEA_SERVER_URL)
# 7. Source ID formatting for traceability (github-owner/repo-run-id)

jobs:
  build-with-monitoring:
    runs-on: ubuntu-latest
    env:
      VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
      VIBETEA_SERVER_URL: ${{ secrets.VIBETEA_SERVER_URL }}
      VIBETEA_SOURCE_ID: "github-${{ github.repository }}-${{ github.run_id }}"

    steps:
      # Download binary or build from source
      - name: Download VibeTea Monitor
        run: curl -fsSL -o vibetea-monitor "..."

      # Start in background (non-blocking)
      - name: Start VibeTea Monitor
        run: |
          if [ -f vibetea-monitor ] && [ -n "$VIBETEA_PRIVATE_KEY" ]; then
            ./vibetea-monitor run &
            MONITOR_PID=$!
            echo "MONITOR_PID=$MONITOR_PID" >> $GITHUB_ENV
          fi

      # Run CI steps (events captured during this time)
      - name: Run tests
        run: cargo test --workspace -- --test-threads=1

      # Graceful shutdown (flush events, don't block)
      - name: Stop VibeTea Monitor
        if: always()
        run: |
          if [ -n "$MONITOR_PID" ]; then
            kill -TERM $MONITOR_PID 2>/dev/null || true
            sleep 2
          fi
```

Key testing patterns for GitHub Actions:
- **Binary delivery**: Tests can use pre-built releases or cargo build
- **Environment variable configuration**: All settings via env vars and secrets
- **Non-blocking operation**: Monitor never blocks workflow (wrapped in conditionals)
- **Graceful shutdown**: SIGTERM handling with event flush before exit
- **Source tracking**: Includes repository and run ID for traceability

#### Composite Action Testing (Phase 6)

File: `.github/actions/vibetea-monitor/action.yml` (Reusable composite action)

The composite action encapsulates monitor deployment as a reusable artifact:

```yaml
name: 'VibeTea Monitor'
description: 'Start VibeTea monitor to track Claude Code events'
author: 'aaronbassett'

# Input validation testing
inputs:
  server-url:
    description: 'URL of the VibeTea server'
    required: true
  private-key:
    description: 'Base64-encoded Ed25519 private key'
    required: true
  source-id:
    description: 'Custom source identifier'
    required: false
    default: ''
  version:
    description: 'Monitor version to download'
    required: false
    default: 'latest'
  shutdown-timeout:
    description: 'Timeout in seconds for graceful shutdown'
    required: false
    default: '5'

# Output availability for downstream steps
outputs:
  monitor-pid:
    description: 'Process ID of the running monitor'
    value: ${{ steps.start-monitor.outputs.pid }}
  monitor-started:
    description: 'Whether the monitor started successfully'
    value: ${{ steps.start-monitor.outputs.started }}

# Implementation using composite steps
runs:
  using: 'composite'
  steps:
    - name: Download VibeTea Monitor
      id: download
      shell: bash
      run: |
        # Download and validate binary

    - name: Start VibeTea Monitor
      id: start-monitor
      shell: bash
      env:
        VIBETEA_PRIVATE_KEY: ${{ inputs.private-key }}
        VIBETEA_SERVER_URL: ${{ inputs.server-url }}
      run: |
        # Start monitor with validation
```

**Testing the composite action:**

In any workflow that uses it:

```yaml
- uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main
  with:
    server-url: ${{ secrets.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    source-id: "pr-${{ github.event.pull_request.number }}"

# Verify action outputs
- name: Check monitor status
  run: |
    echo "Monitor started: ${{ steps.vibetea-monitor.outputs.monitor-started }}"
    echo "Monitor PID: ${{ steps.vibetea-monitor.outputs.monitor-pid }}"
```

Action testing validation:
- **Input handling**: Tests verify required inputs block and optional inputs have sensible defaults
- **Download resilience**: Tests verify graceful handling of download failures
- **Output availability**: Tests verify PID and status outputs are accessible in downstream steps
- **Non-blocking design**: Action should complete successfully even if monitor startup fails
- **Process validation**: Tests verify monitor process actually starts (not just binary downloaded)
- **Cleanup requirements**: Tests document recommended cleanup step for graceful shutdown

### Mocking Strategy

| Layer | Mock Strategy | Location |
|-------|---------------|----------|
| External APIs (TypeScript) | MSW (Mock Service Worker) | `tests/mocks/` (planned) |
| External APIs (Rust) | wiremock | `tests/` with `wiremock::MockServer` |
| Database (TypeScript) | In-memory or test database | TBD |
| Time | `vi.useFakeTimers()` (Vitest) | In test functions |
| Environment | `EnvGuard` RAII pattern (Rust) | Integration tests |
| Subprocess | Real binary execution | CLI integration tests |

### Test Data

#### Fixtures (TypeScript)

```typescript
// Pattern for test fixtures
export const testUser = {
  id: 'test-user-id',
  email: 'test@example.com',
  name: 'Test User',
};

export const testEvent: VibeteaEvent<'session'> = {
  id: 'evt_test123456789012345',
  source: 'test-source',
  timestamp: new Date().toISOString(),
  type: 'session',
  payload: {
    sessionId: '123e4567-e89b-12d3-a456-426614174000',
    action: 'started',
    project: 'test-project',
  },
};
```

#### Helpers (Rust)

```rust
/// Generates a valid 32-byte seed and returns it base64-encoded.
fn generate_valid_base64_seed() -> (String, [u8; 32]) {
    let mut seed = [0u8; 32];
    use rand::Rng;
    rand::rng().fill(&mut seed);
    let base64_seed = BASE64_STANDARD.encode(&seed);
    (base64_seed, seed)
}

/// Creates a test event with a small payload.
fn create_small_event() -> Event {
    Event::new(
        "test-monitor".to_string(),
        EventType::Tool,
        EventPayload::Tool {
            session_id: Uuid::new_v4(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("small.rs".to_string()),
            project: Some("test-project".to_string()),
        },
    )
}
```

## Coverage Requirements

| Metric | Target | Strategy |
|--------|--------|----------|
| Line coverage | 80%+ | Focus on public APIs and error paths |
| Branch coverage | 75%+ | Include both success and failure cases |
| Function coverage | 90%+ | Every exported function should be tested |

### Coverage Exclusions

Files/patterns excluded from coverage:

- `src/generated/` - Auto-generated code
- `*.config.ts` - Configuration files
- `src/types/` - Type definitions only (tested indirectly)
- `src/main.rs` - Binary entrypoint (tested via integration tests)

## Test Categories

### Smoke Tests

Critical path tests that must pass before deploy:

| Test | Purpose | Location |
|------|---------|----------|
| `events.test.ts` | Event type creation and validation | `client/src/__tests__/` |
| `privacy_test.rs` | Privacy compliance across all event types | `monitor/tests/privacy_test.rs` |
| `env_key_test.rs` | Environment key loading and validation | `monitor/tests/env_key_test.rs` |
| `key_export_test.rs` | Export-key subcommand functionality | `monitor/tests/key_export_test.rs` |

### Regression Tests

Tests for previously fixed bugs:

| Test | Issue | Description |
|------|-------|-------------|
| `unsafe_mode_test.rs` | N/A | Verify unsafe auth mode disables validation |

## Test Execution Summary

### Test Count by Package

| Package | Test Type | Count | Status |
|---------|-----------|-------|--------|
| monitor | Unit (inline) | 129+ | In use (includes project_tracker 69 tests) |
| monitor | Integration (env_key_test) | 21 | Phase 11 |
| monitor | Integration (privacy_test) | 17 | In use |
| monitor | Integration (key_export_test) | 12 | Phase 12 |
| monitor | Integration (sender_recovery_test) | N/A | In use |
| server | Unit (inline) | 40+ | In use |
| server | Integration | 1+ | In use |
| client | Unit (Vitest) | 33+ | In use |

### Total Test Coverage (Phase 12)

- **Rust/Monitor**: 179+ tests across unit and integration (includes Phase 11 project_tracker 69 tests)
- **Rust/Server**: 40+ unit tests
- **TypeScript/Client**: 33+ tests
- **Grand Total**: 252+ tests

## CI Integration

### Test Pipeline (from `.github/workflows/ci.yml`)

```yaml
# Rust tests with sequential execution for env var safety
- name: Run tests
  run: cargo test --package ${{ matrix.crate }} -- --test-threads=1

# TypeScript tests
- name: Run tests
  run: pnpm test
```

### GitHub Actions Example Workflow

The example workflow (`.github/workflows/ci-with-monitor.yml`) demonstrates:

1. **Monitor binary deployment** - Download from releases or build locally
2. **Background execution** - Non-blocking operation with PID tracking
3. **Event capture during CI** - Events captured while running tests/builds
4. **Graceful shutdown** - SIGTERM handling for clean shutdown
5. **Secret configuration** - Private key and server URL via GitHub secrets
6. **Error resilience** - Monitor failures don't fail workflows
7. **Source tracking** - Unique identifiers for traceability

This serves as a reference implementation for teams wanting to integrate VibeTea monitoring into their CI pipelines.

### GitHub Actions Composite Action Integration (Phase 6)

The reusable composite action (`.github/actions/vibetea-monitor/action.yml`) simplifies monitor deployment:

```yaml
# Usage in any workflow
- uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main
  with:
    server-url: ${{ secrets.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}

# Optional: Custom source ID for tracking
- uses: aaronbassett/VibeTea/.github/actions/vibetea-monitor@main
  with:
    server-url: ${{ secrets.VIBETEA_SERVER_URL }}
    private-key: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    source-id: "github-${{ github.repository }}-${{ github.run_id }}"

# Graceful shutdown
- name: Stop VibeTea Monitor
  if: always()
  run: |
    if [ -n "$VIBETEA_MONITOR_PID" ]; then
      kill -TERM $VIBETEA_MONITOR_PID 2>/dev/null || true
      sleep 5
    fi
```

Key advantages of the composite action:
- **Reusability**: Action can be used across multiple repositories
- **Encapsulation**: Binary download and startup logic hidden from consumers
- **Output parameters**: PID and status available to downstream steps
- **Non-blocking**: Failures don't interrupt workflow (warnings only)
- **Versioning**: Pin to specific version or use `@main` for latest

### Required Checks

| Check | Blocking | Notes |
|-------|----------|-------|
| Unit tests pass | Yes | Must pass before merge |
| Integration tests pass | Yes | Must pass before merge |
| CLI tests pass | Yes | export-key subcommand (Phase 12) |
| Coverage threshold met | No | Informational only |
| Type checking passes | Yes | TypeScript strict mode |
| Linting passes | Yes | ESLint + Clippy |
| Formatting passes | Yes | Prettier + rustfmt |

## Test Execution Priority

Tests are organized by execution priority in CI:

1. **Linting & Formatting** (Fast, 1 min)
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `pnpm lint`
   - `pnpm format:check`

2. **Type Checking** (Medium, 2 min)
   - `cargo check`
   - `pnpm typecheck`

3. **Unit Tests** (Medium, 3-5 min)
   - Rust: `cargo test --lib`
   - TypeScript: `pnpm test`

4. **Integration Tests** (Slower, 5-10 min)
   - Rust: `cargo test --test '*'`
   - Includes privacy, crypto, sender, export-key tests

5. **Build Release** (Slow, 10+ min)
   - `cargo build --release`

## Testing Best Practices

### Rust

1. **Use descriptive test names**: Names should describe the behavior being tested
2. **Test one thing**: Each test should verify a single behavior
3. **Use #[serial] for env vars**: Prevents test interference
4. **Document requirements**: Link tests to specifications (FR-###)
5. **Test error messages**: Verify errors are clear and actionable
6. **Use RAII patterns**: Automatically clean up resources
7. **Group related tests**: Use module organization with clear comments
8. **Test subprocess interactions**: For CLI, spawn actual binary with Command::new()
9. **Validate output streams**: Ensure correct stream usage (stdout vs stderr)
10. **Verify exit codes**: Expect specific exit codes for different scenarios

### TypeScript

1. **Use Arrange-Act-Assert**: Clear test structure
2. **Test edge cases**: Empty inputs, null, invalid types
3. **Use type guards**: Verify types during tests
4. **Mock external dependencies**: Use MSW for API mocking
5. **Test error paths**: Not just happy paths
6. **Use fixtures**: Centralize test data
7. **Keep tests focused**: One assertion or related assertions per test

### GitHub Actions (YAML)

1. **Test action inputs**: Verify required inputs are enforced, optional inputs have defaults
2. **Test output parameters**: Verify outputs are available to downstream steps
3. **Test error handling**: Verify non-critical failures use warnings, not failures
4. **Test environment isolation**: Use matrix strategy for multiple environment testing
5. **Document cleanup**: Clearly document any manual cleanup steps (e.g., monitor shutdown)
6. **Test workflow integration**: Verify action works with different workflow patterns
7. **Test version pinning**: Verify action works with version references and `@main`

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document describes HOW to test. Update when testing strategy changes.*
