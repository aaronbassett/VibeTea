# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Test Framework

### TypeScript/Client

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Vitest | Inline in `vite.config.ts` | Ready |
| Integration | Vitest | Inline in `vite.config.ts` | Ready |
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
| Integration | Rust built-in | `tests/` directory | In use (Phase 5) |
| E2E | Not selected | TBD | Not started |

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

## Test Organization

### TypeScript/Client Directory Structure

```
client/
├── src/
│   ├── types/
│   │   └── events.ts           # Type definitions
│   ├── hooks/
│   │   └── useEventStore.ts    # Zustand store
│   ├── components/             # Placeholder (empty with .gitkeep)
│   ├── utils/                  # Placeholder (empty with .gitkeep)
│   ├── App.tsx
│   └── main.tsx
└── src/
    └── __tests__/              # Co-located test directory
        └── events.test.ts      # Event type tests
```

### Rust/Server Directory Structure

```
server/
├── src/
│   ├── config.rs               # Config module with inline tests (12 tests)
│   ├── error.rs                # Error module with inline tests (18+ tests)
│   ├── types.rs                # Types module with inline tests (10+ tests)
│   ├── routes.rs               # HTTP routes implementation
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint
└── tests/
    └── unsafe_mode_test.rs     # Integration test for unsafe auth mode
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
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint
└── tests/
    ├── privacy_test.rs         # Integration tests for privacy compliance (17 tests)
    └── (more integration tests to be created)
```

### Test File Location Strategy

**TypeScript**: Co-located tests in `__tests__/` directory

| Source File | Test File |
|-------------|-----------|
| `src/types/events.ts` | `src/__tests__/events.test.ts` |
| `src/hooks/useEventStore.ts` | `src/__tests__/useEventStore.test.ts` (planned) |
| `src/App.tsx` | `src/__tests__/App.test.tsx` (planned) |

**Rust**: Inline tests in same module (`#[cfg(test)] mod tests`)

Tests for a function go in the same file, grouped in a `tests` module at the end of the file. Integration tests go in separate files in the `tests/` directory.

## Test Patterns

### Unit Tests (TypeScript)

Tests follow the Arrange-Act-Assert pattern using Vitest:

#### Example from `src/__tests__/events.test.ts`

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

    // Act + Assert
    expect(event.type).toBe('session');
    expect(event.payload.action).toBe('started');
  });

  it('should create a valid tool event', () => {
    // Arrange
    const event: VibeteaEvent<'tool'> = {
      id: 'evt_test123456789012345',
      source: 'test-source',
      timestamp: new Date().toISOString(),
      type: 'tool',
      payload: {
        sessionId: '123e4567-e89b-12d3-a456-426614174000',
        tool: 'Read',
        status: 'completed',
        context: 'file.ts',
        project: 'test-project',
      },
    };

    // Act + Assert
    expect(event.type).toBe('tool');
    expect(event.payload.tool).toBe('Read');
    expect(event.payload.status).toBe('completed');
  });

  it('should support all event types', () => {
    // Arrange
    const eventTypes: EventType[] = [
      'session',
      'activity',
      'tool',
      'agent',
      'summary',
      'error',
    ];

    // Act + Assert
    expect(eventTypes).toHaveLength(6);
  });
});
```

Key patterns:
1. **Imports**: Vitest `describe`, `it`, `expect` at top
2. **Type safety**: Tests use actual TypeScript types for validation
3. **Descriptive names**: Test names describe the behavior
4. **Arrange-Act-Assert**: Clear three-part structure (though Act and Assert often combined in simple tests)

### Unit Tests (Rust)

Tests follow the Arrange-Act-Assert pattern and are organized inline:

#### Example from server/src/config.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    struct EnvGuard {
        vars: Vec<(String, Option<String>)>,
    }

    impl EnvGuard {
        fn new() -> Self { Self { vars: Vec::new() } }
        fn set(&mut self, key: &str, value: &str) {
            let old_value = env::var(key).ok();
            self.vars.push((key.to_string(), old_value));
            env::set_var(key, value);
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in &self.vars {
                match value {
                    Some(v) => env::set_var(key, v),
                    None => env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn test_config_with_unsafe_no_auth() {
        // Arrange
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_UNSAFE_NO_AUTH", "true");
        guard.remove("VIBETEA_PUBLIC_KEYS");

        // Act
        let config = Config::from_env();

        // Assert
        assert!(config.is_ok());
        assert!(config.unwrap().unsafe_no_auth);
    }

    #[test]
    fn test_parse_public_keys_invalid_format() {
        let mut guard = EnvGuard::new();
        guard.set("VIBETEA_PUBLIC_KEYS", "invalid-no-colon");

        let result = parse_public_keys();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::InvalidFormat { var, .. } if var == "VIBETEA_PUBLIC_KEYS"));
    }
}
```

Key patterns observed in server crate:

1. **Environment isolation**: Uses `EnvGuard` helper to save/restore environment variables
2. **Descriptive names**: Test names describe the behavior (e.g., `test_config_with_unsafe_no_auth`)
3. **Result types**: Tests verify both success and error cases using `assert!` and `matches!`
4. **Parsing tests**: Dedicated tests for parsing functions with various inputs

#### Example from server/src/error.rs

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_missing_displays_correctly() {
        // Arrange
        let err = ConfigError::missing("API_KEY");

        // Act
        let message = err.to_string();

        // Assert
        assert_eq!(message, "missing required configuration: API_KEY");
    }

    #[test]
    fn server_error_is_client_error_returns_true() {
        assert!(ServerError::auth("bad token").is_client_error());
        assert!(ServerError::validation("bad input").is_client_error());
        assert!(ServerError::rate_limit("client", 60).is_client_error());
    }

    #[test]
    fn config_error_converts_to_server_error() {
        let config_err = ConfigError::missing("PORT");
        let server_err: ServerError = config_err.into();
        assert!(matches!(server_err, ServerError::Config(_)));
    }
}
```

Key patterns:
1. **Error display**: Tests verify error messages format correctly
2. **Conversions**: Tests validate `impl From` and `impl Into` conversions
3. **Utility methods**: Tests verify helper methods like `is_client_error()`

### Unit Tests (Rust) - Privacy Module (Phase 5)

Privacy module tests are comprehensive, organized by component and testing Constitution I compliance:

#### Example from monitor/src/privacy.rs inline tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SessionAction, ToolStatus};
    use uuid::Uuid;

    // =========================================================================
    // PrivacyConfig Tests
    // =========================================================================

    #[test]
    fn config_new_with_no_allowlist_allows_all() {
        let config = PrivacyConfig::new(None);
        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.py"));
        assert!(config.is_extension_allowed("file.anything"));
        assert!(config.is_extension_allowed("Makefile")); // No extension allowed when no filter
    }

    #[test]
    fn config_from_env_parses_comma_separated() {
        std::env::set_var("VIBETEA_BASENAME_ALLOWLIST", ".rs,.ts,.md");
        let config = PrivacyConfig::from_env();
        std::env::remove_var("VIBETEA_BASENAME_ALLOWLIST");

        assert!(config.is_extension_allowed("file.rs"));
        assert!(config.is_extension_allowed("file.ts"));
        assert!(config.is_extension_allowed("file.md"));
        assert!(!config.is_extension_allowed("file.py"));
    }

    // =========================================================================
    // extract_basename Tests
    // =========================================================================

    #[test]
    fn extract_basename_from_unix_absolute_path() {
        assert_eq!(
            extract_basename("/home/user/project/src/auth.ts"),
            Some("auth.ts".to_string())
        );
    }

    // =========================================================================
    // PrivacyPipeline Tests
    // =========================================================================

    #[test]
    fn pipeline_tool_bash_strips_context() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: Uuid::nil(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some("rm -rf / --no-preserve-root".to_string()),
            project: Some("my-project".to_string()),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, tool, .. } = result {
            assert_eq!(tool, "Bash");
            assert_eq!(context, None);
        } else {
            panic!("Expected Tool payload");
        }
    }

    #[test]
    fn pipeline_tool_read_extracts_basename() {
        let pipeline = PrivacyPipeline::default();
        let payload = EventPayload::Tool {
            session_id: Uuid::nil(),
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("/home/user/project/src/auth.ts".to_string()),
            project: Some("my-project".to_string()),
        };

        let result = pipeline.process(payload);
        if let EventPayload::Tool { context, .. } = result {
            assert_eq!(context, Some("auth.ts".to_string()));
        } else {
            panic!("Expected Tool payload");
        }
    }
}
```

Privacy unit tests are organized into sections:
1. **PrivacyConfig Tests** (10 tests): Configuration parsing, environment variable handling, extension allowlist
2. **extract_basename Tests** (8 tests): Path parsing edge cases, Unix/relative paths, hidden files
3. **PrivacyPipeline Tests** (15 tests): Event processing, sensitive tool context stripping, allowlist filtering
4. **Edge Case Tests** (5 tests): Complex paths, Unicode filenames, case sensitivity

### Integration Tests (Rust)

Larger tests that exercise multiple components together:

#### Example from `server/tests/unsafe_mode_test.rs`

Integration tests verify end-to-end functionality with full configuration:

```rust
// Tests for unsafe mode authentication
// Run with: cargo test --test unsafe_mode_test
```

### Integration Tests (Rust) - Privacy Module (Phase 5)

Privacy compliance integration tests in `monitor/tests/privacy_test.rs` validate Constitution I requirements:

```rust
//! Privacy compliance test suite for VibeTea Monitor.
//!
//! These tests validate Constitution I (Privacy by Design) by ensuring
//! no sensitive data is ever present in processed events.

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates a deterministic session ID for testing.
fn test_session_id() -> Uuid {
    Uuid::nil()
}

/// Creates a privacy pipeline with default configuration (no allowlist).
fn default_pipeline() -> PrivacyPipeline {
    PrivacyPipeline::new(PrivacyConfig::new(None))
}

/// Checks that a JSON string does not contain any sensitive path patterns.
fn assert_no_sensitive_paths(json: &str, test_name: &str) {
    for pattern in SENSITIVE_PATH_PATTERNS {
        assert!(
            !json.contains(pattern),
            "{test_name}: JSON contains sensitive path pattern '{pattern}': {json}"
        );
    }
}

// =============================================================================
// Test 1: no_full_paths_in_tool_events
// =============================================================================

/// Verifies that full file paths are reduced to basenames in Tool events.
#[test]
fn no_full_paths_in_tool_events() {
    let pipeline = default_pipeline();

    let payload = EventPayload::Tool {
        session_id: test_session_id(),
        tool: "Read".to_string(),
        status: ToolStatus::Completed,
        context: Some("/home/user/projects/secret/src/auth.rs".to_string()),
        project: Some("my-project".to_string()),
    };

    let result = pipeline.process(payload);

    if let EventPayload::Tool { context, .. } = &result {
        assert_eq!(
            context.as_deref(),
            Some("auth.rs"),
            "Context should be reduced to basename only"
        );
    }

    let json = serde_json::to_string(&result).expect("Failed to serialize");
    assert!(!json.contains("/home/"), "Serialized event should not contain full path");
}

// =============================================================================
// Test 2: bash_commands_never_transmitted
// =============================================================================

/// Verifies that Bash tool context (containing actual shell commands) is always stripped.
#[test]
fn bash_commands_never_transmitted() {
    let pipeline = default_pipeline();

    let dangerous_commands = vec![
        "rm -rf /important",
        "curl -H 'Authorization: Bearer secret_token' https://api.example.com",
        "export API_KEY=sk-1234567890",
        "mysql -u root -pMySecretPass123",
    ];

    for command in dangerous_commands {
        let payload = EventPayload::Tool {
            session_id: test_session_id(),
            tool: "Bash".to_string(),
            status: ToolStatus::Completed,
            context: Some(command.to_string()),
            project: Some("project".to_string()),
        };

        let result = pipeline.process(payload);

        if let EventPayload::Tool { context, tool, .. } = &result {
            assert_eq!(tool, "Bash", "Tool name should be preserved");
            assert_eq!(
                *context, None,
                "Bash context should be None for command: {command}"
            );
        }

        let json = serde_json::to_string(&result).expect("Failed to serialize");
        assert!(!json.contains("secret"), "Serialized event should not contain 'secret'");
    }
}

// =============================================================================
// Additional tests cover:
// - Test 3: grep_patterns_never_transmitted
// - Test 4: glob_patterns_never_transmitted
// - Test 5: websearch_never_transmits_context
// - Test 6: webfetch_never_transmits_context
// - Test 7: summary_text_stripped
// - Test 8: all_event_types_safe (comprehensive integration test)
// - Test 9: allowlist_filtering_removes_sensitive_extensions
// - Test 10+: extract_basename edge cases
// =============================================================================
```

Privacy integration tests cover Constitution I requirements:
1. **No full paths**: Validates path-to-basename conversion
2. **No sensitive tool context**: Bash, Grep, Glob, WebSearch, WebFetch
3. **Summary stripping**: Session summaries neutralized to "Session ended"
4. **Extension filtering**: Allowlist correctly filters by file extension
5. **Comprehensive coverage**: All event types processed safely, no sensitive data in JSON

### Error Handling Tests

Both TypeScript and Rust emphasize testing error cases:

**Rust Error Type Tests** (from `server/src/error.rs`):

```rust
#[test]
fn config_error_invalid_displays_correctly() {
    let err = ConfigError::invalid("port", "must be a positive integer");
    assert_eq!(
        err.to_string(),
        "invalid configuration value for 'port': must be a positive integer"
    );
}

#[test]
fn server_error_rate_limit_displays_correctly() {
    let err = ServerError::rate_limit("192.168.1.100", 30);
    assert_eq!(
        err.to_string(),
        "rate limit exceeded for 192.168.1.100, retry after 30 seconds"
    );
}
```

## Mocking Strategy

### TypeScript (Planned)

When needed, mocking will use:

| Layer | Mock Strategy | Location |
|-------|---------------|----------|
| HTTP | MSW (Mock Service Worker) | `src/mocks/handlers.ts` |
| WebSocket | Manual test doubles or `vitest.mock()` | Test files |
| State | Create test fixtures or real store in tests | Test files |
| Time | `vi.useFakeTimers()` (Vitest) | Test setup |

### Rust

No mocking framework is used currently. Tests use:

1. **Environment isolation**: `EnvGuard` pattern to manage env vars
2. **Helper functions**: `parse_bool_env()`, `parse_port()` tested independently
3. **Direct instantiation**: Create test instances with test data
4. **Deterministic test doubles**: For privacy tests, use fixed UUIDs (`Uuid::nil()`)

## Test Data

### TypeScript Fixtures

Test data is created directly in test files. Example from `__tests__/events.test.ts`:

```typescript
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
```

### Rust Fixtures

Test data is created directly in test modules. Example from `server/src/types.rs`:

```rust
#[test]
fn test_event_serialization_tool() {
    let session_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let timestamp = DateTime::parse_from_rfc3339("2026-02-02T14:30:00Z")
        .unwrap()
        .with_timezone(&Utc);

    let event = Event {
        id: "evt_k7m2n9p4q1r6s3t8u5v0".to_string(),
        source: "macbook-pro".to_string(),
        timestamp,
        event_type: EventType::Tool,
        payload: EventPayload::Tool {
            session_id,
            tool: "Read".to_string(),
            status: ToolStatus::Completed,
            context: Some("main.rs".to_string()),
            project: Some("vibetea".to_string()),
        },
    };

    let json = serde_json::to_string_pretty(&event).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed["id"], "evt_k7m2n9p4q1r6s3t8u5v0");
    assert_eq!(parsed["type"], "tool");
}
```

### Privacy Test Helpers (Phase 5)

Privacy tests use helper functions and constants for consistency:

```rust
/// Sensitive path patterns that should NEVER appear in processed events.
const SENSITIVE_PATH_PATTERNS: &[&str] = &[
    "/home/",
    "/Users/",
    "/root/",
    "/var/",
    "/etc/",
    "C:\\Users\\",
    "C:\\Program Files\\",
];

/// Sensitive command/content patterns that should be stripped from context.
const SENSITIVE_COMMAND_PATTERNS: &[&str] = &[
    "rm -rf",
    "sudo ",
    "chmod ",
    "curl -",
    "wget ",
    "Bearer ",
    "Authorization:",
];

/// Creates a privacy pipeline with default configuration (no allowlist).
fn default_pipeline() -> PrivacyPipeline {
    PrivacyPipeline::new(PrivacyConfig::new(None))
}

/// Creates a privacy pipeline with a specific extension allowlist.
fn pipeline_with_allowlist(extensions: &[&str]) -> PrivacyPipeline {
    let allowlist: HashSet<String> = extensions.iter().map(|s| s.to_string()).collect();
    PrivacyPipeline::new(PrivacyConfig::new(Some(allowlist)))
}
```

## Coverage Requirements

### Targets

| Metric | Target | Current |
|--------|--------|---------|
| Line coverage | 80% | TBD |
| Branch coverage | 75% | TBD |
| Function coverage | 80% | TBD |

### Coverage Exclusions

Files/patterns excluded from coverage (will be configured):

- `dist/` - Built artifacts
- `*.config.ts` - Configuration files (already tested by usage)
- `src/types/` - Type definitions (logic tested via usage)
- `node_modules/` - External dependencies
- `src/__tests__/` - Test files themselves

## Test Categories

### Smoke Tests (Critical Path)

Tests that must pass before any deploy:

| Test | Purpose | Location |
|------|---------|----------|
| Type checking | TypeScript compilation | `npm run typecheck` |
| Linting | Code style compliance | `npm run lint` |
| Config tests | Configuration loading | `server/src/config.rs` and `monitor/src/config.rs` tests |
| Error tests | Error type safety | `server/src/error.rs` tests |
| Serialization tests | JSON round-trips | `server/src/types.rs` tests |
| Event type tests | Type-safe event creation | `client/src/__tests__/events.test.ts` |
| Privacy tests | Constitution I compliance | `monitor/src/privacy.rs` (unit) + `monitor/tests/privacy_test.rs` (integration) |

### Unit Tests by Module

#### TypeScript/Client

**events.test.ts** (3 tests)
- Session event creation and validation
- Tool event creation and validation
- Event type enumeration validation

#### Rust/Server

**config.rs** (12 tests)
- Configuration parsing from environment variables
- Default value handling
- Public key parsing with whitespace tolerance
- Validation and error cases
- Port number parsing and range validation

**error.rs** (18+ tests)
- Error display formatting for all variants
- Error type conversions (`.into()`, `.from()`)
- Error source chain preservation
- Utility methods (`is_client_error()`, `is_server_error()`)
- Helper constructors (`ServerError::auth()`, `ServerError::rate_limit()`, etc.)

**types.rs** (10+ tests)
- Event type and enum serialization
- Serialization to camelCase JSON for event payloads
- Serialization of enums to snake_case
- Round-trip serialization (serialize → deserialize)
- Payload-specific serialization with optional fields
- Field omission when None

#### Rust/Monitor

Tests follow similar patterns to server modules covering:
- Configuration parsing and validation
- Error type formatting and conversions
- Event type serialization

**privacy.rs** (38 unit tests - Phase 5)
- **PrivacyConfig tests** (10): Configuration creation, environment variable parsing, allowlist filtering
  - `config_new_with_no_allowlist_allows_all`
  - `config_new_with_allowlist_filters_extensions`
  - `config_allowlist_rejects_no_extension`
  - `config_from_env_with_no_var_allows_all`
  - `config_from_env_parses_comma_separated`
  - `config_from_env_handles_missing_dots`
  - `config_from_env_trims_whitespace`
  - `config_from_env_filters_empty_entries`
  - `config_default_allows_all`
  - Plus environment-specific tests

- **extract_basename tests** (8): Path parsing for various formats
  - `extract_basename_from_unix_absolute_path`
  - `extract_basename_from_unix_relative_path`
  - `extract_basename_already_basename`
  - `extract_basename_with_dots_in_name`
  - `extract_basename_hidden_file`
  - `extract_basename_empty_path`
  - `extract_basename_root_path`
  - `extract_basename_trailing_slash`

- **PrivacyPipeline tests** (15): Event processing and context stripping
  - `pipeline_session_passes_through`
  - `pipeline_activity_passes_through`
  - `pipeline_agent_passes_through`
  - `pipeline_error_passes_through`
  - `pipeline_summary_strips_text`
  - `pipeline_tool_bash_strips_context`
  - `pipeline_tool_grep_strips_context`
  - `pipeline_tool_glob_strips_context`
  - `pipeline_tool_websearch_strips_context`
  - `pipeline_tool_webfetch_strips_context`
  - `pipeline_tool_read_extracts_basename`
  - `pipeline_tool_write_extracts_basename`
  - `pipeline_tool_edit_extracts_basename`
  - `pipeline_tool_with_no_context`
  - `pipeline_tool_allowlist_filters`
  - Plus allowlist and project preservation tests

- **Edge case tests** (5): Complex scenarios
  - `pipeline_handles_complex_paths`
  - `pipeline_handles_unicode_filenames`
  - `pipeline_case_sensitive_tool_names`

### Integration Tests

#### Rust/Server

**unsafe_mode_test.rs** (Tests integration of auth bypass)
- Tests server startup with unsafe auth mode
- Verifies event acceptance without signatures
- Validates WebSocket connections in unsafe mode

Planned integration tests will cover:
- Full event creation and serialization pipeline
- Configuration loading + usage together
- HTTP request handling with signatures
- WebSocket connection and message broadcast

#### Rust/Monitor (Phase 5)

**privacy_test.rs** (17 integration tests)
- **Constitution I Compliance Tests**: Validates privacy guarantees in production scenarios
  1. `no_full_paths_in_tool_events` - Path reduction validation
  2. `no_full_paths_various_formats` - Multiple path format handling
  3. `no_full_paths_in_session_events` - Session event path validation
  4. `bash_commands_never_transmitted` - Command stripping verification
  5. `grep_patterns_never_transmitted` - Search pattern stripping
  6. `glob_patterns_never_transmitted` - File pattern stripping
  7. `websearch_never_transmits_context` - Search query stripping
  8. `webfetch_never_transmits_context` - URL stripping
  9. `summary_text_stripped` - Summary neutralization
  10. `all_event_types_safe` - Comprehensive multi-type test
  11. `allowlist_filtering_removes_sensitive_extensions` - Extension filtering
  12. Additional edge case and serialization tests

Each integration test:
- Creates realistic event payloads with sensitive data
- Processes them through the privacy pipeline
- Verifies no sensitive data appears in JSON output
- Uses helper functions for consistency and maintainability

## CI Integration

### Test Pipeline

```yaml
# GitHub Actions (when configured)
test:
  - Lint TypeScript (ESLint)
  - Format check (Prettier)
  - Type check TypeScript
  - Unit tests (Vitest)
  - Lint Rust (clippy)
  - Format check Rust (rustfmt)
  - Unit tests (cargo test --lib)
  - Privacy unit tests (cargo test -p vibetea-monitor privacy)
  - Integration tests (cargo test --test '*')
  - Privacy integration tests (cargo test --test privacy_test)
  - Coverage report (optional)
```

### Required Checks

| Check | Blocking | Comment |
|-------|----------|---------|
| TypeScript linting | Yes | Style consistency |
| TypeScript type check | Yes | Type safety |
| TypeScript unit tests | Yes | Correctness |
| Rust linting (clippy) | Yes | Code quality |
| Rust unit tests | Yes | Correctness |
| Privacy unit tests | Yes | Constitution I compliance |
| Rust integration tests | Yes | End-to-end behavior |
| Privacy integration tests | Yes | Constitution I compliance in production scenarios |
| Code formatting matches | Yes | Consistency |

## Current Test Coverage

### TypeScript/Client

- **__tests__/events.test.ts**: 3 test cases for event type validation and creation
- Framework (Vitest) installed and ready
- Test organization structure established

### Rust Modules - Server

- **config.rs**: 12 test cases covering env parsing, defaults, validation, public key parsing
- **error.rs**: 18+ test cases covering error formatting, conversions, utility methods
- **types.rs**: 10+ test cases covering event serialization and round-trips
- **Integration tests**: 1 test file covering unsafe authentication mode

### Rust Modules - Monitor

- **config.rs**: Test cases covering env parsing, path resolution, defaults
- **error.rs**: Test cases covering error formatting and conversions
- **types.rs**: Test cases covering event serialization
- **privacy.rs** (Phase 5): 38 unit tests covering privacy configuration, path extraction, event processing
- **privacy_test.rs** (Phase 5): 17 integration tests covering Constitution I compliance
- Integration tests directory ready for full pipeline tests

## Test Execution

### Running All Tests

```bash
# TypeScript
cd client && npm test              # Run once
cd client && npm run test:watch    # Watch mode

# Rust (entire workspace)
cargo test                         # All tests
cargo test -- --test-threads=1    # Sequential (prevents env var interference)

# Rust (specific modules)
cargo test -p vibetea-monitor privacy  # Privacy tests only
```

### Running Specific Test Modules

```bash
# Rust - specific module
cargo test -p vibetea-server config::tests
cargo test -p vibetea-server error::tests

# Rust - integration tests
cargo test --test unsafe_mode_test
cargo test --test privacy_test

# Privacy tests specifically
cargo test -p vibetea-monitor privacy::tests  # Unit tests
cargo test --test privacy_test                # Integration tests
```

## Next Steps for Testing

1. **TypeScript**: Create additional unit tests for Zustand store and selector functions
2. **TypeScript**: Add component tests for UI elements as they're built
3. **Rust/Server**: Expand integration tests for HTTP routes and WebSocket functionality
4. **Rust/Monitor**: Add integration tests for file watching and JSONL parsing
5. **Coverage**: Set up coverage reporting in CI/CD pipeline with threshold enforcement
6. **E2E**: Evaluate Playwright or Cypress for client workflow testing once UI is more complete
7. **Snapshot testing**: Consider for event serialization if JSON formats become complex
8. **Property-based testing**: Consider `proptest` for privacy module edge cases and path handling

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing details → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document describes HOW to test. Update when testing strategy changes.*
