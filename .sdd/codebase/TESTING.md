# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-03

## Test Framework

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
| Integration | Rust built-in | `tests/` directory with `serial_test` crate | In use (Phase 11) |
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
| `cargo test --test env_key_test` | Run environment key loading tests (Phase 11, 21 tests) |
| `cargo test -p vibetea-monitor crypto` | Run crypto module tests |
| `cargo test -p vibetea-monitor sender` | Run sender module tests |

**Important**: Monitor tests run with `--test-threads=1` in CI to prevent environment variable interference:

```bash
cargo test --package vibetea-monitor -- --test-threads=1
```

## Test Organization

### TypeScript/Client Directory Structure

```
client/
├── src/
│   ├── types/
│   │   └── events.ts           # Type definitions
│   ├── hooks/
│   │   ├── useEventStore.ts    # Zustand store
│   │   ├── useWebSocket.ts     # WebSocket connection hook (Phase 7)
│   │   └── useSessionTimeouts.ts # Session timeouts hook (Phase 10)
│   ├── components/
│   │   ├── ConnectionStatus.tsx # Connection indicator (Phase 7)
│   │   ├── TokenForm.tsx        # Token input form (Phase 7)
│   │   ├── EventStream.tsx      # Virtual scrolling list (Phase 8)
│   │   ├── Heatmap.tsx          # Activity heatmap (Phase 9)
│   │   └── SessionOverview.tsx  # Session overview (Phase 10)
│   ├── utils/
│   │   └── formatting.ts        # Timestamp/duration formatting (Phase 8)
│   ├── App.tsx
│   └── main.tsx
└── src/
    └── __tests__/              # Co-located test directory
        ├── events.test.ts      # Event type tests
        └── formatting.test.ts  # Formatting utility tests (Phase 8, 33 tests)
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
│   ├── crypto.rs               # Ed25519 crypto operations with 14 inline unit tests
│   ├── sender.rs               # HTTP sender with 8 inline unit tests
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint (CLI)
└── tests/
    ├── env_key_test.rs         # 21 integration tests for env var key loading (Phase 11)
    ├── privacy_test.rs         # 17 integration tests for privacy compliance
    └── sender_recovery_test.rs # Integration tests for error recovery
```

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

## Test Patterns

### Unit Tests (TypeScript)

Tests follow the Arrange-Act-Assert pattern using Vitest:

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
});
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

    let result = pipeline.process(payload);

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

### Mocking Strategy

| Layer | Mock Strategy | Location |
|-------|---------------|----------|
| External APIs (TypeScript) | MSW (Mock Service Worker) | `tests/mocks/` (planned) |
| External APIs (Rust) | wiremock | `tests/` with `wiremock::MockServer` |
| Database (TypeScript) | In-memory or test database | TBD |
| Time | `vi.useFakeTimers()` (Vitest) | In test functions |
| Environment | `EnvGuard` RAII pattern (Rust) | Integration tests |

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

### Regression Tests

Tests for previously fixed bugs:

| Test | Issue | Description |
|------|-------|-------------|
| `unsafe_mode_test.rs` | N/A | Verify unsafe auth mode disables validation |

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

### Required Checks

| Check | Blocking | Notes |
|-------|----------|-------|
| Unit tests pass | Yes | Must pass before merge |
| Integration tests pass | Yes | Must pass before merge |
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
   - Includes privacy, crypto, sender tests

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

### TypeScript

1. **Use Arrange-Act-Assert**: Clear test structure
2. **Test edge cases**: Empty inputs, null, invalid types
3. **Use type guards**: Verify types during tests
4. **Mock external dependencies**: Use MSW for API mocking
5. **Test error paths**: Not just happy paths
6. **Use fixtures**: Centralize test data
7. **Keep tests focused**: One assertion or related assertions per test

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document describes HOW to test. Update when testing strategy changes.*
