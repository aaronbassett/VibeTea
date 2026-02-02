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

#### Rust/Monitor

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

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_missing_env_var_display() {
        // Arrange
        let err = ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string());

        // Act
        let message = err.to_string();

        // Assert
        assert_eq!(
            message,
            "missing required environment variable: VIBETEA_SERVER_URL"
        );
    }
}
```

Key patterns observed in monitor crate:

1. **Isolation**: Tests use helper functions (e.g., `with_clean_env()`) to isolate environment state
2. **Descriptive names**: Test names describe the behavior (e.g., `test_missing_server_url`)
3. **Result types**: Tests verify both success and error cases

Example from `config.rs`:

```rust
#[test]
fn test_missing_server_url() {
    with_clean_env(|| {
        let result = Config::from_env();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::MissingEnvVar(ref s) if s == "VIBETEA_SERVER_URL"));
    });
}
```

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

**Rust Error Type Tests** (from `error.rs`):

```rust
#[test]
fn monitor_error_config_display() {
    let config_err = ConfigError::MissingEnvVar("VIBETEA_SERVER_URL".to_string());
    let err = MonitorError::Config(config_err);
    assert_eq!(
        err.to_string(),
        "configuration error: missing required environment variable: VIBETEA_SERVER_URL"
    );
}

#[test]
fn monitor_error_io_conversion() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: MonitorError = io_err.into();
    assert!(matches!(err, MonitorError::Io(_)));
    assert!(err.to_string().contains("I/O error"));
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

1. **Test helpers**: Helper functions to create test instances
2. **Fixtures**: Constants for common test data (e.g., `with_clean_env()`)
3. **Environment isolation**: Tests clean environment variables before and after

Example isolation from `config.rs`:

```rust
fn with_clean_env<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // Save existing vars
    let saved_vars: Vec<(String, String)> = env::vars()
        .filter(|(k, _)| k.starts_with("VIBETEA_"))
        .collect();

    for (key, _) in &saved_vars {
        env::remove_var(key);
    }

    let result = f();

    // Restore vars
    for (key, value) in saved_vars {
        env::set_var(key, value);
    }

    result
}
```

## Test Data

### Rust Fixtures

Test data is created directly in test modules. Example from `types.rs`:

```rust
#[test]
fn event_serializes_with_camel_case_fields() {
    let session_id = Uuid::nil();  // Known value for testing
    let event = Event {
        id: "evt_12345678901234567890".to_string(),
        source: "test-monitor".to_string(),
        timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        event_type: EventType::Session,
        payload: EventPayload::Session {
            session_id,
            action: SessionAction::Started,
            project: "test".to_string(),
        },
    };
    // ...
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
| Config tests | Configuration loading | `monitor/src/config.rs` tests |
| Serialization tests | JSON round-trips | `monitor/src/types.rs` tests |

### Unit Tests By Module

#### Rust/Monitor

**config.rs** (12 tests)
- Configuration parsing from environment variables
- Default value handling
- Validation and error cases
- Path resolution with home directory

**error.rs** (7 tests)
- Error display formatting
- Error type conversions (`.into()`)
- Error source chain preservation
- String-based error variants

**types.rs** (10+ tests)
- Event ID generation and format
- Serialization to camelCase JSON
- Serialization of enums to snake_case
- Round-trip serialization (serialize → deserialize)
- Payload-specific serialization

### Integration Tests

None yet. Will test:

- Full event creation and serialization pipeline
- Configuration loading + usage together
- File watching workflow
- HTTP request building

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

### Rust Modules

- **config.rs**: 12 test cases covering env parsing, defaults, validation
- **error.rs**: 7 test cases covering error formatting and conversions
- **types.rs**: 10+ test cases covering event serialization

### TypeScript

- No tests yet (test directory is empty placeholder)
- Test framework (Vitest) is installed and ready

## Next Steps for Testing

1. **TypeScript**: Create test fixtures and first unit tests for Zustand store
2. **Rust**: Add integration test directory and full-pipeline tests
3. **E2E**: Evaluate Playwright or Cypress for client workflow testing
4. **Coverage**: Set up coverage reporting in CI/CD pipeline
5. **Snapshot testing**: Consider for event serialization if needed

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing details → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document describes HOW to test. Update when testing strategy changes.*
