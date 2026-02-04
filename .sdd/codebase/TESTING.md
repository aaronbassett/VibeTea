# Testing Strategy

> **Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Test Frameworks

| Type | Framework | Configuration | Location |
|------|-----------|----------------|----------|
| Unit (TypeScript) | Vitest | `client/vite.config.ts` | `client/src/__tests__/` |
| Unit (Rust) | Built-in (cargo test) | `Cargo.toml` | `src/` (co-located) |
| Integration (Rust) | Built-in (cargo test) | `Cargo.toml` | `src/` (co-located) |
| E2E | None planned | N/A | N/A |

### Running Tests

| Command | Purpose | Environment |
|---------|---------|-------------|
| `npm test` | Run all TypeScript unit tests (watch mode available) | Client |
| `npm run test:watch` | Run TypeScript tests in watch mode | Client |
| `cargo test --workspace` | Run all Rust tests | Server + Monitor |
| `cargo test --workspace --test-threads=1` | Run Rust tests serially (required for env var tests) | Server + Monitor |
| `cargo test -p vibetea-server` | Run server tests only | Server |
| `cargo test -p vibetea-monitor` | Run monitor tests only | Monitor |

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
    └── stats_tracker.rs   # Token usage tracking (Phase 8) with tests
```

**Test organization strategy**: Co-located `#[cfg(test)] mod tests` blocks at end of each module file.

## Test Patterns

### TypeScript Unit Tests

**Pattern: Describe/It structure with Arrange/Act/Assert**

From `client/src/__tests__/events.test.ts`:

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

**Pattern: Mocking browser APIs**

From `client/src/__tests__/App.test.tsx`:

```typescript
// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};
  return {
    getItem: (key: string) => store[key] ?? null,
    setItem: (key: string, value: string) => { store[key] = value; },
    removeItem: (key: string) => { delete store[key]; },
    clear: () => { store = {}; },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

// Mock WebSocket
class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  // ... other properties
  constructor(_url: string) {
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      if (this.onopen) this.onopen(new Event('open'));
    }, 0);
  }
}

Object.defineProperty(window, 'WebSocket', {
  value: MockWebSocket,
  writable: true,
});
```

**Pattern: Component testing with state management**

From `client/src/__tests__/App.test.tsx`:

```typescript
beforeEach(() => {
  localStorage.clear();
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
  });
});

describe('App Token Handling', () => {
  it('renders token form when no token is stored', () => {
    render(<App />);
    expect(screen.getByText('VibeTea Dashboard')).toBeInTheDocument();
    expect(
      screen.getByText(/enter your authentication token/i)
    ).toBeInTheDocument();
  });

  it('transitions from token form to dashboard when token is saved', async () => {
    render(<App />);

    const tokenInput = screen.getByLabelText(/authentication token/i);
    fireEvent.change(tokenInput, { target: { value: 'new-test-token' } });

    const saveButton = screen.getByRole('button', { name: /save token/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(screen.getByText('Sessions')).toBeInTheDocument();
    });

    expect(localStorage.getItem('vibetea_token')).toBe('new-test-token');
  });
});
```

**Pattern: Utility function testing**

From `client/src/__tests__/formatting.test.ts`:

```typescript
describe('formatTimestamp', () => {
  it('formats a valid RFC 3339 timestamp to HH:MM:SS', () => {
    const timestamp = '2026-02-02T14:30:45Z';
    const date = new Date(timestamp);
    const expected = [
      String(date.getHours()).padStart(2, '0'),
      String(date.getMinutes()).padStart(2, '0'),
      String(date.getSeconds()).padStart(2, '0'),
    ].join(':');

    expect(formatTimestamp(timestamp)).toBe(expected);
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatTimestamp('not-a-date')).toBe('--:--:--');
  });

  it('returns fallback for empty string', () => {
    expect(formatTimestamp('')).toBe('--:--:--');
  });
});
```

### Rust Unit Tests

**Pattern: Module-level test organization with helper functions**

From `server/src/auth.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey, SECRET_KEY_LENGTH};

    // Helper functions
    fn create_test_keypair(seed: u8) -> (SigningKey, String) {
        let mut seed_bytes = [0u8; SECRET_KEY_LENGTH];
        for (i, byte) in seed_bytes.iter_mut().enumerate() {
            *byte = seed.wrapping_add(i as u8);
        }
        let signing_key = SigningKey::from_bytes(&seed_bytes);
        let public_key_bytes = signing_key.verifying_key().to_bytes();
        let public_key_base64 = BASE64_STANDARD.encode(public_key_bytes);
        (signing_key, public_key_base64)
    }

    fn generate_test_keypair() -> (SigningKey, String) {
        create_test_keypair(1)
    }

    fn create_keys_map(source_id: &str, public_key_base64: &str) -> HashMap<String, String> {
        let mut keys = HashMap::new();
        keys.insert(source_id.to_string(), public_key_base64.to_string());
        keys
    }

    // Test cases
    #[test]
    fn verify_signature_succeeds_for_valid_signature() {
        let (signing_key, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let message = b"test message to sign";
        let signature = signing_key.sign(message);
        let signature_base64 = BASE64_STANDARD.encode(signature.to_bytes());

        let result = verify_signature("monitor-1", &signature_base64, message, &public_keys);

        assert!(result.is_ok());
    }

    #[test]
    fn verify_signature_fails_for_unknown_source() {
        let (_, public_key_base64) = generate_test_keypair();
        let public_keys = create_keys_map("monitor-1", &public_key_base64);

        let result = verify_signature(
            "unknown-monitor",
            "c29tZXNpZ25hdHVyZQ==",
            b"message",
            &public_keys,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AuthError::UnknownSource(ref s) if s == "unknown-monitor"));
        assert!(err.is_unknown_source());
    }
}
```

**Pattern: Environment variable testing with serial_test crate**

Tests modifying environment variables must run with `--test-threads=1` to prevent interference. See `CLAUDE.md` for Phase 3 learning about `EnvGuard` RAII pattern.

### Skill Tracker Tests (Phase 5)

**Pattern: File parsing and event creation tests**

From `monitor/src/trackers/skill_tracker.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_history_entry_valid() {
        let line = r#"{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "abc-123"}"#;
        let entry = parse_history_entry(line).expect("should parse");

        assert_eq!(entry.display, "/commit");
        assert_eq!(entry.timestamp, 1738567268363);
        assert_eq!(entry.project, "/home/user/project");
        assert_eq!(entry.session_id, "abc-123");
    }

    #[test]
    fn parse_history_entry_invalid_json() {
        let line = "not json";
        let result = parse_history_entry(line);
        assert!(matches!(result, Err(HistoryParseError::InvalidJson(_))));
    }

    #[test]
    fn create_skill_invocation_event_extracts_skill_name() {
        let entry = HistoryEntry {
            display: "/commit -m \"message\"".to_string(),
            timestamp: 1738567268363,
            project: "/home/user/project".to_string(),
            session_id: "abc-123".to_string(),
        };

        let event = create_skill_invocation_event(&entry).expect("should create event");
        assert_eq!(event.skill_name, "commit");
        assert_eq!(event.session_id, "abc-123");
    }
}
```

Key patterns for Phase 5 skill_tracker:
- **Deterministic timestamps**: Use fixed millisecond values for reproducible tests
- **Privacy validation**: Verify only skill name extracted, not command arguments
- **Error cases**: Test missing fields, invalid JSON, malformed entries
- **Event creation**: Verify timestamp conversion from ms to UTC DateTime

### Todo Tracker Tests (Phase 6)

The `todo_tracker` module establishes comprehensive test patterns with 79 unit tests organized in marked sections:

**Pattern: Organized test sections with clear task IDs**

From `monitor/src/trackers/todo_tracker.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // =========================================================================
    // T118: Todo Filename Parsing Tests
    // =========================================================================

    #[test]
    fn extract_session_id_valid_filename() {
        let path = Path::new(
            "/home/user/.claude/todos/6e45a55c-3124-4cc8-ad85-040a5c316009-agent-6e45a55c-3124-4cc8-ad85-040a5c316009.json",
        );
        let session_id = extract_session_id_from_filename(path).unwrap();
        assert_eq!(session_id, "6e45a55c-3124-4cc8-ad85-040a5c316009");
    }

    // =========================================================================
    // T119: Todo Status Counting Tests
    // =========================================================================

    #[test]
    fn count_statuses_mixed_entries() {
        let entries = vec![
            TodoEntry { content: "A".to_string(), status: TodoStatus::Completed, active_form: None },
            TodoEntry { content: "B".to_string(), status: TodoStatus::Completed, active_form: None },
            TodoEntry { content: "C".to_string(), status: TodoStatus::InProgress, active_form: Some("...".to_string()) },
        ];
        let counts = count_todo_statuses(&entries);
        assert_eq!(counts.completed, 2);
        assert_eq!(counts.in_progress, 1);
    }

    // =========================================================================
    // T120: Abandonment Detection Tests
    // =========================================================================

    #[test]
    fn abandonment_session_ended_with_incomplete_tasks() {
        let counts = TodoStatusCounts { completed: 2, in_progress: 0, pending: 3 };
        assert!(is_abandoned(&counts, true));  // Session ended + incomplete = abandoned
    }
}
```

Key patterns for Phase 6 todo_tracker:
- **Comprehensive test sections**: Tests organized with clear task IDs (T118-T120) and section headers
- **79 total tests**: Covers parsing, counting, abandonment detection, edge cases, and async integration
- **Async test patterns**: Uses `#[tokio::test]` for async integration tests with real file watching
- **Temporary file handling**: Uses `tempfile::TempDir` for isolated test file operations
- **Timeout management**: Uses `tokio::time::timeout()` for time-limited async assertions
- **Realistic test data**: Uses actual JSON format matching `~/.claude/todos/` structure
- **Privacy verification**: Tests validate that only status counts/metadata captured, not task content
- **Abandonment logic**: Tests verify the combination of session end + incomplete tasks = abandoned

### Stats Tracker Tests (Phase 8)

The `stats_tracker` module watches `~/.claude/stats-cache.json` and emits token usage events:

**Pattern: File parsing with retry logic and event emission**

From `monitor/src/trackers/stats_tracker.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stats_cache_with_valid_json() {
        let json_str = r#"{
            "totalSessions": 150,
            "totalMessages": 2500,
            "totalToolUsage": 8000,
            "longestSession": "00:45:30",
            "hourCounts": { "0": 10, "1": 5 },
            "modelUsage": {
                "claude-sonnet-4-20250514": {
                    "inputTokens": 1500000,
                    "outputTokens": 300000,
                    "cacheReadInputTokens": 800000,
                    "cacheCreationInputTokens": 100000
                }
            }
        }"#;

        let cache: StatsCache = serde_json::from_str(json_str).expect("should parse");
        assert_eq!(cache.total_sessions, 150);
        assert_eq!(cache.total_messages, 2500);
        assert_eq!(cache.total_tool_usage, 8000);
    }

    #[test]
    fn model_tokens_uses_serde_defaults() {
        // Stats cache JSON may be missing fields; serde(default) handles this
        let json_str = r#"{ "inputTokens": 1000 }"#;
        let tokens: ModelTokens = serde_json::from_str(json_str).expect("should parse");

        assert_eq!(tokens.input_tokens, 1000);
        assert_eq!(tokens.output_tokens, 0);  // default
        assert_eq!(tokens.cache_read_input_tokens, 0);  // default
    }

    #[test]
    fn stats_event_enum_pattern_matching() {
        let metrics_event = SessionMetricsEvent {
            total_sessions: 42,
            total_messages: 1234,
            total_tool_usage: 567,
            longest_session: "sess_longest".to_string(),
        };

        let stats_event = StatsEvent::SessionMetrics(metrics_event.clone());

        match stats_event {
            StatsEvent::SessionMetrics(evt) => {
                assert_eq!(evt.total_sessions, 42);
            }
            StatsEvent::TokenUsage(_) => panic!("expected SessionMetrics"),
        }
    }

    #[test]
    fn token_usage_event_from_model_tokens() {
        let model_tokens = ModelTokens {
            input_tokens: 5000,
            output_tokens: 2500,
            cache_read_input_tokens: 1000,
            cache_creation_input_tokens: 500,
        };

        let event = TokenUsageEvent {
            model: "claude-3-opus".to_string(),
            input_tokens: model_tokens.input_tokens,
            output_tokens: model_tokens.output_tokens,
            cache_read_tokens: model_tokens.cache_read_input_tokens,
            cache_creation_tokens: model_tokens.cache_creation_input_tokens,
        };

        assert_eq!(event.model, "claude-3-opus");
        assert_eq!(event.input_tokens, 5000);
        assert_eq!(event.cache_read_tokens, 1000);
    }
}
```

Key patterns for Phase 8 stats_tracker:
- **Lenient JSON parsing**: All fields use `#[serde(default)]` for missing or extra fields
- **camelCase mapping**: Uses `#[serde(rename_all = "camelCase")]` to match Claude Code JSON format
- **Event emission pattern**: Tests verify StatsEvent enum matching for both TokenUsage and SessionMetrics
- **Model iteration**: Tests verify events emitted per model in stats cache
- **File retry logic**: Debouncing and retry handling for mid-write file reads
- **Error handling**: Tests for file watcher init, parse failures, and channel closure

### Integration Tests

**Strategy for Rust integration tests**:
- Co-located in same module with unit tests
- Use `#[tokio::test]` for async integration tests
- Setup and teardown using fixtures/helper functions
- Can test multiple modules together

## Mocking Strategy

### TypeScript Mocking

| Target | Strategy | Location |
|--------|----------|----------|
| Browser APIs (localStorage, WebSocket) | Inline mock objects in test file | Test setup section |
| Zustand store | Direct `setState()` calls | Test setup with beforeEach |
| React components | React Testing Library render + query selectors | Test body |
| HTTP requests | Not yet implemented (no integration tests) | Future |

### Rust Mocking

| Target | Strategy | Location |
|--------|----------|----------|
| File I/O | Not mocked; use real temp files in tests | Test function |
| Network | Not mocked; use wiremock (Phase 6+) | Future |
| Time | Not mocked; use DateTime::new() for fixed times | Test data |
| Cryptography | Real Ed25519 operations with deterministic seeds | Test setup |
| File watching | Use real file changes in isolated test directories | Test function |

## Test Data

### Fixtures

TypeScript fixtures are embedded in test files as constants:

```typescript
const testUser = {
  id: 'test-user-id',
  email: 'test@example.com',
  name: 'Test User',
};

const testEvent: VibeteaEvent<'session'> = {
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

Rust fixtures are generated by helper functions:

```rust
fn create_test_keypair(seed: u8) -> (SigningKey, String) {
    // Deterministic seed ensures reproducible tests
    let mut seed_bytes = [0u8; SECRET_KEY_LENGTH];
    for (i, byte) in seed_bytes.iter_mut().enumerate() {
        *byte = seed.wrapping_add(i as u8);
    }
    // ... create keypair
}
```

### Phase 5 Skill Tracker Fixtures

```rust
fn create_test_history_entry(display: &str, session_id: &str) -> HistoryEntry {
    HistoryEntry {
        display: display.to_string(),
        timestamp: 1738567268363,  // 2025-02-03T14:34:28Z
        project: "/home/user/project".to_string(),
        session_id: session_id.to_string(),
    }
}
```

### Phase 6 Todo Tracker Fixtures

```rust
const SAMPLE_TODO: &str = r#"[
    {"content": "Task 1", "status": "completed", "activeForm": null},
    {"content": "Task 2", "status": "in_progress", "activeForm": "Working on task 2..."},
    {"content": "Task 3", "status": "pending", "activeForm": null}
]"#;

fn create_test_todo_file(dir: &TempDir, session_id: &str, content: &str) -> PathBuf {
    let filename = format!("{}-agent-{}.json", session_id, session_id);
    let todo_path = dir.path().join(&filename);
    let mut file = std::fs::File::create(&todo_path).expect("Failed to create todo file");
    file.write_all(content.as_bytes()).expect("Failed to write content");
    file.flush().expect("Failed to flush");
    todo_path
}
```

### Phase 8 Stats Tracker Fixtures

```rust
fn create_test_stats_cache() -> StatsCache {
    let mut model_usage = HashMap::new();
    model_usage.insert(
        "claude-sonnet-4-20250514".to_string(),
        ModelTokens {
            input_tokens: 1500000,
            output_tokens: 300000,
            cache_read_input_tokens: 800000,
            cache_creation_input_tokens: 100000,
        },
    );

    StatsCache {
        total_sessions: 150,
        total_messages: 2500,
        total_tool_usage: 8000,
        longest_session: "00:45:30".to_string(),
        hour_counts: {
            let mut counts = HashMap::new();
            counts.insert("0".to_string(), 10);
            counts.insert("9".to_string(), 25);
            counts.insert("17".to_string(), 50);
            counts
        },
        model_usage,
    }
}
```

## Coverage Requirements

### Target Coverage

| Metric | Target | Strategy |
|--------|--------|----------|
| Line coverage | 80%+ | Focus on critical paths |
| Branch coverage | 75%+ | Test error cases |
| Function coverage | 80%+ | Public APIs fully tested |

### Coverage Exclusions

Files/patterns excluded from coverage:

- `src/generated/` - Auto-generated code
- `*.config.ts` - Configuration files
- `src/types/` - Type definitions only
- Test files themselves (`*.test.ts`, `#[cfg(test)]`)
- Binary main entry points

## Test Categories

### Smoke Tests

Critical path tests that verify basic functionality:

**TypeScript**:
- `App.test.tsx`: Token handling, connection status, filter integration
- `events.test.ts`: Valid event creation for all event types
- `formatting.test.ts`: Core timestamp and duration formatting

**Rust**:
- `auth.rs`: Signature verification succeeds with valid data
- `config.rs`: Configuration loads from environment variables
- `agent_tracker.rs`: Task tool input parsing works (Phase 4)
- `skill_tracker.rs`: History entry parsing works (Phase 5)
- `todo_tracker.rs`: Todo file parsing and abandonment detection works (Phase 6)
- `stats_tracker.rs`: Stats cache JSON parsing and event creation works (Phase 8)

### Regression Tests

Tests for previously fixed bugs would follow this pattern:

```rust
#[test]
fn regression_issue_123() {
    // Test that reproduces the bug and verifies fix
}
```

## Test Execution

### Local Development

```bash
# TypeScript watch mode for rapid feedback
npm run test:watch

# Run specific test file
npm run test -- App.test.tsx

# Rust with serial execution for env var safety
cargo test --workspace --test-threads=1

# Run specific test
cargo test --workspace test_name

# Run monitor tests only (includes todo_tracker and stats_tracker)
cargo test -p vibetea-monitor --test-threads=1
```

### CI Pipeline

The CI workflow would run:

1. TypeScript tests (Vitest)
   - Unit tests run in parallel
   - Coverage threshold check

2. Rust tests (Cargo)
   - Server tests with `--test-threads=1`
   - Monitor tests with `--test-threads=1` (includes todo_tracker and stats_tracker)
   - All tests must pass

3. Code quality checks
   - ESLint for TypeScript
   - Clippy for Rust
   - Type checking

### Required Checks

| Check | Blocking | Tool |
|-------|----------|------|
| Unit tests pass | Yes | Vitest / Cargo |
| Type checking passes | Yes | TypeScript / Rust compiler |
| Linting passes | Yes | ESLint / Clippy |
| Coverage threshold met | No (for now) | Built-in reporters |

## Test Documentation

### Test Comments

Use JSDoc comments to explain non-obvious test logic:

```typescript
/**
 * Test that WebSocket auto-reconnection works after connection loss.
 *
 * Mock WebSocket closes unexpectedly, triggering exponential backoff
 * reconnection attempts. Verifies that connection recovers within
 * a reasonable time frame with correct backoff delays.
 */
it('should reconnect after connection loss', async () => {
  // Test code
});
```

Use doc comments for Rust test helpers:

```rust
/// Creates a test key pair from a deterministic seed.
///
/// Using deterministic seeds makes tests reproducible. The seed is expanded
/// to fill the 32-byte private key requirement.
fn create_test_keypair(seed: u8) -> (SigningKey, String) {
    // Implementation
}
```

## Known Test Limitations

1. **No E2E tests**: Browser-based E2E testing not yet implemented
2. **No integration tests**: TypeScript and Rust integration tests not yet implemented
3. **No HTTP mocking**: Server and Monitor HTTP interactions not yet mocked for testing
4. **No database tests**: In-memory SQLite or test containers not yet used
5. **Coverage reporting**: TypeScript coverage not yet reported; Rust coverage optional

## Future Testing Improvements

- [ ] E2E tests with Playwright for full app flows
- [ ] Integration tests for Server + Client communication
- [ ] HTTP request mocking with wiremock (Phase 6+)
- [ ] Coverage reporting in CI pipeline
- [ ] Performance benchmarks for critical paths
- [ ] Property-based testing for edge cases
- [ ] Snapshot testing for complex data structures

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing details → SECURITY.md
- Architecture patterns → ARCHITECTURE.md
- Technology choices → STACK.md

---

*This document describes HOW to test. Update when testing strategy changes.*
