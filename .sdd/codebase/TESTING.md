# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-02
**Last Updated**: 2026-02-02

## Test Framework

### TypeScript/Client

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Vitest | `vitest` (dev dependency) | Ready |
| Integration | Vitest | Same as unit | Ready |
| E2E | Not selected | TBD | Not started |

### Rust/Server

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)]` inline | In use |
| Integration | Rust built-in | `tests/` directory | Ready |
| E2E | Not selected | TBD | Not started |

### Rust/Monitor

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)]` inline | In use |
| Integration | Rust built-in | `tests/` directory | Ready |
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
| `cargo test -- --test-threads=1` | Run tests sequentially |

## Test Organization

### TypeScript/Client Directory Structure

```
client/
├── src/
│   ├── types/
│   │   └── events.ts           # Type definitions
│   ├── hooks/
│   │   └── useEventStore.ts    # Zustand store
│   ├── App.tsx
│   └── main.tsx
└── tests/                       # Placeholder (empty with .gitkeep)
    └── .gitkeep
```

### Rust/Server Directory Structure

```
server/
├── src/
│   ├── config.rs               # Config module with inline tests (12 tests)
│   ├── error.rs                # Error module with inline tests (18+ tests)
│   ├── types.rs                # Types module with inline tests (10+ tests)
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint
└── tests/                       # Integration tests (to be created)
    └── (none yet)
```

### Rust/Monitor Directory Structure

```
monitor/
├── src/
│   ├── config.rs               # Config module with inline tests
│   ├── error.rs                # Error module with inline tests
│   ├── types.rs                # Types module with inline tests
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint
└── tests/                       # Integration tests (to be created)
    └── (none yet)
```

### Test File Location Strategy

**TypeScript**: Co-located tests (same filename + `.test.ts` extension)

| Source File | Test File |
|-------------|-----------|
| `src/hooks/useEventStore.ts` | `src/hooks/useEventStore.test.ts` |
| `src/App.tsx` | `src/App.test.tsx` |

**Rust**: Inline tests in same module (`#[cfg(test)] mod tests`)

Tests for a function go in the same file, grouped in a `tests` module at the end of the file.

## Test Patterns

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

### Unit Tests (TypeScript - Not Yet Implemented)

When tests are added, they will follow this pattern:

```typescript
import { describe, it, expect } from 'vitest';
import { selectEventsBySession } from '../hooks/useEventStore';
import type { EventStore, VibeteaEvent } from '../types/events';

describe('selectEventsBySession', () => {
  it('should return events for the specified session', () => {
    // Arrange
    const sessionId = 'session-123';
    const event1: VibeteaEvent = { /* ... */ };
    const event2: VibeteaEvent = { /* ... */ };
    const state: EventStore = {
      status: 'connected',
      events: [event1, event2],
      sessions: new Map(),
      // ... other required fields
    };

    // Act
    const result = selectEventsBySession(state, sessionId);

    // Assert
    expect(result).toHaveLength(2);
  });
});
```

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
| WebSocket | Manual test doubles | Test files |
| State | Create test fixtures | Test files |
| Time | `vi.useFakeTimers()` (Vitest) | Test setup |

### Rust

No mocking framework is used currently. Tests use:

1. **Environment isolation**: `EnvGuard` pattern to manage env vars
2. **Helper functions**: `parse_bool_env()`, `parse_port()` tested independently
3. **Direct instantiation**: Create test instances with test data

## Test Data

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

### TypeScript Fixtures (Planned)

When fixtures are needed:

```typescript
// tests/fixtures/events.ts
export const testUser = {
  id: 'test-user-id',
  email: 'test@example.com',
  name: 'Test User',
};

export function createSession(overrides = {}) {
  return {
    sessionId: 'session-' + Math.random().toString(36).substr(2, 9),
    source: 'test-source',
    project: 'test-project',
    startedAt: new Date(),
    lastEventAt: new Date(),
    status: 'active' as const,
    eventCount: 1,
    ...overrides,
  };
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
- `src/types/` - Type definitions only (no logic)
- `node_modules/` - External dependencies

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

### Unit Tests by Module

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

Tests follow similar patterns to server modules.

### Integration Tests

None yet. Will test:

- Full event creation and serialization pipeline
- Configuration loading + usage together
- File watching workflow
- HTTP request building with signatures
- End-to-end monitor → server communication

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
  - Unit tests (cargo test)
  - Coverage report (optional)
```

### Required Checks

| Check | Blocking | Comment |
|-------|----------|---------|
| TypeScript linting | Yes | Style consistency |
| TypeScript type check | Yes | Type safety |
| Unit tests pass | Yes | Correctness |
| Rust linting (clippy) | Yes | Code quality |
| Rust tests pass | Yes | Correctness |
| Code formatting matches | Yes | Consistency |

## Current Test Coverage

### Rust Modules - Server

- **config.rs**: 12 test cases covering env parsing, defaults, validation, public key parsing
- **error.rs**: 18+ test cases covering error formatting, conversions, utility methods
- **types.rs**: 10+ test cases covering event serialization and round-trips

### Rust Modules - Monitor

- **config.rs**: Test cases covering env parsing, path resolution, defaults
- **error.rs**: Test cases covering error formatting and conversions
- **types.rs**: Test cases covering event serialization

### TypeScript

- No tests yet (test directory is empty placeholder)
- Test framework (Vitest) is installed and ready
- Types and hooks ready for test coverage

## Next Steps for Testing

1. **TypeScript**: Create test fixtures and first unit tests for Zustand store and selector functions
2. **Rust**: Add integration test directory and full-pipeline tests for both server and monitor
3. **E2E**: Evaluate Playwright or Cypress for client workflow testing
4. **Coverage**: Set up coverage reporting in CI/CD pipeline with threshold enforcement
5. **Snapshot testing**: Consider for event serialization if JSON formats become complex

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing details → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document describes HOW to test. Update when testing strategy changes.*
