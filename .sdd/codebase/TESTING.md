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
| `cargo test -p vibetea-monitor crypto` | Run crypto module tests |
| `cargo test -p vibetea-monitor sender` | Run sender module tests |

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
│   ├── crypto.rs               # Ed25519 crypto operations with 14 inline unit tests (Phase 6)
│   ├── sender.rs               # HTTP sender with 8 inline unit tests (Phase 6)
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint (CLI)
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
| `src/hooks/useWebSocket.ts` | `src/__tests__/useWebSocket.test.ts` (planned) |
| `src/hooks/useSessionTimeouts.ts` | `src/__tests__/useSessionTimeouts.test.ts` (planned) |
| `src/components/ConnectionStatus.tsx` | `src/__tests__/ConnectionStatus.test.tsx` (planned) |
| `src/components/TokenForm.tsx` | `src/__tests__/TokenForm.test.tsx` (planned) |
| `src/components/EventStream.tsx` | `src/__tests__/EventStream.test.tsx` (planned) |
| `src/components/Heatmap.tsx` | `src/__tests__/Heatmap.test.tsx` (planned) |
| `src/components/SessionOverview.tsx` | `src/__tests__/SessionOverview.test.tsx` (planned) |
| `src/utils/formatting.ts` | `src/__tests__/formatting.test.ts` |
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

### Formatting Utilities Tests (TypeScript - Phase 8)

Formatting utilities are tested comprehensively with edge cases and various input scenarios:

#### Example from `src/__tests__/formatting.test.ts` (33 tests)

```typescript
import { describe, it, expect } from 'vitest';
import {
  formatTimestamp,
  formatTimestampFull,
  formatRelativeTime,
  formatDuration,
  formatDurationShort,
} from '../utils/formatting';

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

  it('handles timestamps with timezone offsets', () => {
    const timestamp = '2026-02-02T14:30:45+05:30';
    const result = formatTimestamp(timestamp);
    expect(result).toMatch(/^\d{2}:\d{2}:\d{2}$/);
  });

  it('returns fallback for empty string', () => {
    expect(formatTimestamp('')).toBe('--:--:--');
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatTimestamp('not-a-date')).toBe('--:--:--');
  });
});

describe('formatRelativeTime', () => {
  const now = new Date('2026-02-02T14:30:00Z');

  it('returns "just now" for timestamps less than a minute ago', () => {
    expect(formatRelativeTime('2026-02-02T14:29:30Z', now)).toBe('just now');
    expect(formatRelativeTime('2026-02-02T14:29:59Z', now)).toBe('just now');
  });

  it('returns minutes ago for timestamps less than an hour', () => {
    expect(formatRelativeTime('2026-02-02T14:29:00Z', now)).toBe('1m ago');
    expect(formatRelativeTime('2026-02-02T14:25:00Z', now)).toBe('5m ago');
    expect(formatRelativeTime('2026-02-02T13:31:00Z', now)).toBe('59m ago');
  });

  it('returns hours ago for timestamps less than a day', () => {
    expect(formatRelativeTime('2026-02-02T13:30:00Z', now)).toBe('1h ago');
    expect(formatRelativeTime('2026-02-02T12:30:00Z', now)).toBe('2h ago');
  });

  it('returns days ago for timestamps less than a week', () => {
    expect(formatRelativeTime('2026-01-30T14:30:00Z', now)).toBe('3d ago');
    expect(formatRelativeTime('2026-01-27T14:30:00Z', now)).toBe('6d ago');
  });

  it('returns weeks ago for timestamps more than a week old', () => {
    expect(formatRelativeTime('2026-01-26T14:30:00Z', now)).toBe('1w ago');
    expect(formatRelativeTime('2026-01-19T14:30:00Z', now)).toBe('2w ago');
  });

  it('returns fallback for invalid timestamp', () => {
    expect(formatRelativeTime('invalid')).toBe('unknown');
  });
});

describe('formatDuration', () => {
  it('formats hours and minutes', () => {
    expect(formatDuration(5400000)).toBe('1h 30m'); // 1.5 hours
    expect(formatDuration(3600000)).toBe('1h'); // 1 hour exactly
    expect(formatDuration(7260000)).toBe('2h 1m'); // 2 hours 1 minute
  });

  it('formats minutes and seconds', () => {
    expect(formatDuration(330000)).toBe('5m 30s'); // 5.5 minutes
    expect(formatDuration(60000)).toBe('1m'); // 1 minute exactly
    expect(formatDuration(90000)).toBe('1m 30s'); // 1.5 minutes
  });

  it('formats seconds only', () => {
    expect(formatDuration(30000)).toBe('30s');
    expect(formatDuration(1000)).toBe('1s');
  });

  it('omits seconds when hours are present', () => {
    expect(formatDuration(3630000)).toBe('1h'); // 1 hour, 0 minutes, 30 seconds
  });

  it('returns fallback for zero', () => {
    expect(formatDuration(0)).toBe('0s');
  });

  it('returns fallback for negative values', () => {
    expect(formatDuration(-1000)).toBe('0s');
    expect(formatDuration(-5400000)).toBe('0s');
  });

  it('returns fallback for NaN', () => {
    expect(formatDuration(NaN)).toBe('0s');
  });

  it('handles large durations', () => {
    expect(formatDuration(172800000)).toBe('48h'); // 48 hours
    expect(formatDuration(360000000)).toBe('100h'); // 100 hours
  });
});

describe('formatDurationShort', () => {
  it('formats hours:minutes:seconds for durations >= 1 hour', () => {
    expect(formatDurationShort(5400000)).toBe('1:30:00'); // 1.5 hours
    expect(formatDurationShort(3600000)).toBe('1:00:00'); // 1 hour
    expect(formatDurationShort(3661000)).toBe('1:01:01'); // 1h 1m 1s
  });

  it('formats minutes:seconds for durations < 1 hour', () => {
    expect(formatDurationShort(330000)).toBe('5:30'); // 5.5 minutes
    expect(formatDurationShort(60000)).toBe('1:00'); // 1 minute
    expect(formatDurationShort(90000)).toBe('1:30'); // 1.5 minutes
  });

  it('formats seconds with leading zero for durations < 1 minute', () => {
    expect(formatDurationShort(30000)).toBe('0:30'); // 30 seconds
    expect(formatDurationShort(1000)).toBe('0:01'); // 1 second
  });

  it('returns fallback for zero', () => {
    expect(formatDurationShort(0)).toBe('0:00');
  });

  it('returns fallback for negative values', () => {
    expect(formatDurationShort(-1000)).toBe('0:00');
  });

  it('returns fallback for NaN', () => {
    expect(formatDurationShort(NaN)).toBe('0:00');
  });

  it('handles large durations', () => {
    expect(formatDurationShort(172800000)).toBe('48:00:00'); // 48 hours
    expect(formatDurationShort(361845000)).toBe('100:30:45'); // 100 hours, 30 minutes, 45 seconds
  });
});
```

Key patterns for formatting utilities testing (Phase 8, 33 tests):
1. **Pure function testing**: Test deterministic output for given inputs
2. **Graceful fallbacks**: Verify sensible defaults for invalid input
3. **Type coercion**: Test behavior with invalid types (NaN, wrong types)
4. **Edge cases**: Zero, negative, and extreme values
5. **Format validation**: Regex patterns to verify output format
6. **Timezone handling**: Test with UTC, offsets, and local times
7. **Reference parameters**: Use fixed `now` parameter for predictable relative time tests
8. **Comprehensive coverage**: Tests cover all time units and fallback scenarios

### Session Overview Tests (TypeScript - Phase 10, Planned)

Session overview tests will validate activity counting, session sorting, and pulse animations:

#### Planned Test Patterns for `SessionOverview.tsx`

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { SessionOverview } from '../SessionOverview';
import { useEventStore } from '../../hooks/useEventStore';

describe('SessionOverview Component', () => {
  it('should render sessions from store', () => {
    // Mock store with sample sessions
    vi.mocked(useEventStore).mockImplementation((selector) =>
      selector({
        sessions: new Map([
          ['session-1', {
            sessionId: 'session-1',
            source: 'test',
            project: 'TestProject',
            startedAt: new Date(),
            lastEventAt: new Date(),
            status: 'active',
            eventCount: 10,
          }],
        ]),
        events: [],
      } as any)
    );

    render(<SessionOverview />);

    expect(screen.getByText('TestProject')).toBeInTheDocument();
  });

  it('should show empty state when no sessions', () => {
    vi.mocked(useEventStore).mockImplementation((selector) =>
      selector({
        sessions: new Map(),
        events: [],
      } as any)
    );

    render(<SessionOverview />);

    expect(screen.getByText('No active sessions')).toBeInTheDocument();
  });

  it('should display activity indicators based on event count', () => {
    // Test pulse animation classes based on recent event count
    // Low activity (1-5 events) = animate-pulse-slow
    // Medium activity (6-15 events) = animate-pulse-medium
    // High activity (16+ events) = animate-pulse-fast
  });

  it('should sort sessions with active first', () => {
    // Verify active sessions appear before inactive/ended
  });

  it('should display status badges correctly', () => {
    // Test Active (green), Idle (yellow), Ended (gray) badges
  });

  it('should call onSessionClick when card is clicked', () => {
    const onSessionClick = vi.fn();
    // Test click handler invocation
  });

  it('should support keyboard navigation (Enter/Space)', () => {
    // Test Enter and Space key activation
  });

  it('should have proper accessibility labels', () => {
    // Test aria-label presence and content
    // Test role="region" for session overview
    // Test role="listitem" for session cards
  });
});
```

Key patterns for session overview testing:
1. **Activity counting**: Verify `countRecentEventsBySession` returns correct counts
2. **Pulse animation mapping**: Test activity level determination (low/medium/high)
3. **Session sorting**: Verify active first, then by recent activity
4. **Status badge rendering**: Verify correct styling for each status
5. **Click handling**: Test `onSessionClick` callback invocation
6. **Keyboard accessibility**: Support Enter/Space activation
7. **ARIA labels**: Verify accessibility attributes

### Component Tests (TypeScript - Phase 7)

Component tests validate rendering, props, and interactions using Vitest:

#### Planned Test Patterns for `ConnectionStatus.tsx`

```typescript
import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { ConnectionStatus } from '../ConnectionStatus';
import { useEventStore } from '../../hooks/useEventStore';

describe('ConnectionStatus Component', () => {
  it('should render with connected status', () => {
    // Mock Zustand store to return connected status
    vi.mocked(useEventStore).mockImplementation((selector) =>
      selector({ status: 'connected' } as any)
    );

    render(<ConnectionStatus />);

    const indicator = screen.getByRole('status');
    expect(indicator).toBeInTheDocument();
  });

  it('should show label when showLabel prop is true', () => {
    vi.mocked(useEventStore).mockImplementation((selector) =>
      selector({ status: 'connected' } as any)
    );

    render(<ConnectionStatus showLabel />);

    expect(screen.getByText('Connected')).toBeInTheDocument();
  });

  it('should apply custom className', () => {
    vi.mocked(useEventStore).mockImplementation((selector) =>
      selector({ status: 'disconnected' } as any)
    );

    render(<ConnectionStatus className="custom-class" />);

    const container = screen.getByRole('status');
    expect(container).toHaveClass('custom-class');
  });

  it('should display correct color for each status', () => {
    const statuses = ['connected', 'connecting', 'reconnecting', 'disconnected'];
    const expectedColors = ['bg-green-500', 'bg-yellow-500', 'bg-yellow-500', 'bg-red-500'];

    statuses.forEach((status, index) => {
      vi.mocked(useEventStore).mockImplementation((selector) =>
        selector({ status } as any)
      );

      const { unmount } = render(<ConnectionStatus />);

      const dot = screen.getByRole('status').querySelector('span[aria-hidden]');
      expect(dot).toHaveClass(expectedColors[index]);

      unmount();
    });
  });
});
```

Key patterns for component testing:
1. **Mock store**: Use `vi.mock()` to mock Zustand store behavior
2. **Render and query**: Use React Testing Library utilities
3. **Accessibility testing**: Check ARIA roles and labels
4. **Props validation**: Test different prop combinations
5. **Style verification**: Verify Tailwind classes are applied

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
1. **PrivacyConfig Tests** (10 tests): Configuration creation, environment variable parsing, allowlist filtering
2. **extract_basename Tests** (8 tests): Path parsing for various formats
3. **PrivacyPipeline Tests** (15 tests): Event processing and context stripping
4. **Edge Case Tests** (5 tests): Complex scenarios

### Unit Tests (Rust) - Crypto Module (Phase 6)

Crypto module tests validate Ed25519 keypair generation, storage, and signing:

#### Example from monitor/src/crypto.rs inline tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Verifier;
    use tempfile::TempDir;

    #[test]
    fn test_generate_creates_valid_keypair() {
        let crypto = Crypto::generate();
        let pubkey = crypto.public_key_base64();

        // Public key should be base64-encoded 32 bytes (44 chars with padding)
        assert!(!pubkey.is_empty());
        assert!(pubkey.len() >= 43); // Base64 of 32 bytes
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();

        // Generate and save
        let original = Crypto::generate();
        let original_pubkey = original.public_key_base64();
        original.save(dir_path).unwrap();

        // Load and verify
        let loaded = Crypto::load(dir_path).unwrap();
        let loaded_pubkey = loaded.public_key_base64();

        assert_eq!(original_pubkey, loaded_pubkey);
    }

    #[test]
    fn test_exists_returns_false_for_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        assert!(!Crypto::exists(temp_dir.path()));
    }

    #[test]
    fn test_exists_returns_true_after_save() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        assert!(Crypto::exists(temp_dir.path()));
    }

    #[test]
    fn test_sign_produces_verifiable_signature() {
        let crypto = Crypto::generate();
        let message = b"test message for signing";

        let signature_b64 = crypto.sign(message);
        let signature_bytes = BASE64_STANDARD.decode(&signature_b64).unwrap();
        let signature = Signature::from_slice(&signature_bytes).unwrap();

        // Verify the signature using the public key
        let verifying_key = crypto.verifying_key();
        assert!(verifying_key.verify(message, &signature).is_ok());
    }

    #[test]
    fn test_sign_raw_produces_64_byte_signature() {
        let crypto = Crypto::generate();
        let message = b"test message";

        let signature = crypto.sign_raw(message);
        assert_eq!(signature.len(), 64);
    }

    #[test]
    fn test_different_messages_produce_different_signatures() {
        let crypto = Crypto::generate();
        let sig1 = crypto.sign(b"message one");
        let sig2 = crypto.sign(b"message two");

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_same_message_produces_same_signature() {
        let crypto = Crypto::generate();
        let message = b"same message";

        // Note: Ed25519 is deterministic, so same message = same signature
        let sig1 = crypto.sign(message);
        let sig2 = crypto.sign(message);

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_load_from_nonexistent_dir_fails() {
        let result = Crypto::load(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_from_empty_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);

        // Create empty file
        File::create(&priv_path).unwrap();

        let result = Crypto::load(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKey(_)));
    }

    #[test]
    fn test_load_from_short_file_fails() {
        let temp_dir = TempDir::new().unwrap();
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);

        // Create file with only 16 bytes (should be 32)
        let mut file = File::create(&priv_path).unwrap();
        file.write_all(&[0u8; 16]).unwrap();

        let result = Crypto::load(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidKey(_)));
    }

    #[cfg(unix)]
    #[test]
    fn test_save_sets_correct_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Check private key permissions (0600)
        let priv_path = temp_dir.path().join(PRIVATE_KEY_FILE);
        let priv_perms = fs::metadata(&priv_path).unwrap().permissions();
        assert_eq!(priv_perms.mode() & 0o777, 0o600);

        // Check public key permissions (0644)
        let pub_path = temp_dir.path().join(PUBLIC_KEY_FILE);
        let pub_perms = fs::metadata(&pub_path).unwrap().permissions();
        assert_eq!(pub_perms.mode() & 0o777, 0o644);
    }

    #[test]
    fn test_public_key_file_contains_base64() {
        let temp_dir = TempDir::new().unwrap();
        let crypto = Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Read public key file
        let pub_path = temp_dir.path().join(PUBLIC_KEY_FILE);
        let contents = fs::read_to_string(pub_path).unwrap();
        let pubkey = contents.trim();

        // Should be valid base64 and decode to 32 bytes
        let decoded = BASE64_STANDARD.decode(pubkey).unwrap();
        assert_eq!(decoded.len(), 32);
    }
}
```

Crypto unit tests cover (14 tests):
1. **Keypair generation**: Valid keypair creation
2. **Save/Load roundtrip**: Persistence and recovery
3. **File existence**: Detection of existing keys
4. **Signature generation**: Base64 and raw signature creation
5. **Signature verification**: Ed25519 signature validation
6. **Determinism**: Same message produces same signature
7. **Error handling**: Load failures on invalid files
8. **File permissions**: Unix-specific 0600/0644 permission validation
9. **Public key format**: Base64 encoding validation

### Unit Tests (Rust) - Sender Module (Phase 6)

Sender module tests validate event buffering, retry logic, and jitter:

#### Example from monitor/src/sender.rs inline tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventPayload, EventType, SessionAction};
    use uuid::Uuid;

    fn create_test_event() -> Event {
        Event::new(
            "test-monitor".to_string(),
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        )
    }

    fn create_test_crypto() -> Crypto {
        Crypto::generate()
    }

    fn create_test_sender() -> Sender {
        let config = SenderConfig::new(
            "http://localhost:8080".to_string(),
            "test-monitor".to_string(),
            10, // Small buffer for testing
        );
        Sender::new(config, create_test_crypto())
    }

    #[test]
    fn test_queue_adds_events() {
        let mut sender = create_test_sender();
        assert!(sender.is_empty());

        sender.queue(create_test_event());
        assert_eq!(sender.buffer_len(), 1);

        sender.queue(create_test_event());
        assert_eq!(sender.buffer_len(), 2);
    }

    #[test]
    fn test_queue_evicts_oldest_when_full() {
        let mut sender = create_test_sender();

        // Fill buffer to capacity (10 events)
        for _ in 0..10 {
            let evicted = sender.queue(create_test_event());
            assert_eq!(evicted, 0);
        }
        assert_eq!(sender.buffer_len(), 10);

        // Add one more - should evict oldest
        let evicted = sender.queue(create_test_event());
        assert_eq!(evicted, 1);
        assert_eq!(sender.buffer_len(), 10);
    }

    #[test]
    fn test_sender_config_with_defaults() {
        let config = SenderConfig::with_defaults(
            "https://example.com".to_string(),
            "my-monitor".to_string(),
        );
        assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_add_jitter_stays_within_bounds() {
        let sender = create_test_sender();
        let base = Duration::from_secs(10);

        // Run multiple times to test randomness bounds
        for _ in 0..100 {
            let jittered = sender.add_jitter(base);
            let secs = jittered.as_secs_f64();
            // Should be within ±25% of 10 seconds
            assert!(secs >= 7.5 && secs <= 12.5, "Jitter out of bounds: {}", secs);
        }
    }

    #[test]
    fn test_increase_retry_delay_doubles() {
        let mut sender = create_test_sender();
        assert_eq!(sender.current_retry_delay.as_secs(), INITIAL_RETRY_DELAY_SECS);

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), 2);

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), 4);
    }

    #[test]
    fn test_increase_retry_delay_caps_at_max() {
        let mut sender = create_test_sender();
        sender.current_retry_delay = Duration::from_secs(MAX_RETRY_DELAY_SECS);

        sender.increase_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), MAX_RETRY_DELAY_SECS);
    }

    #[test]
    fn test_reset_retry_delay() {
        let mut sender = create_test_sender();
        sender.current_retry_delay = Duration::from_secs(30);

        sender.reset_retry_delay();
        assert_eq!(sender.current_retry_delay.as_secs(), INITIAL_RETRY_DELAY_SECS);
    }

    #[test]
    fn test_is_empty() {
        let mut sender = create_test_sender();
        assert!(sender.is_empty());

        sender.queue(create_test_event());
        assert!(!sender.is_empty());
    }
}
```

Sender unit tests cover (8 tests):
1. **Queueing**: Adding events to buffer
2. **Buffer overflow**: FIFO eviction when full
3. **Configuration**: Default values
4. **Jitter**: Exponential backoff with jitter bounds
5. **Retry delay**: Doubling with max cap
6. **Retry reset**: Reset to initial delay on success
7. **Buffer state**: Empty/non-empty checks

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
5. **Temporary directories**: `tempfile::TempDir` for file system tests (crypto, sender)

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
| Formatting tests | Timestamp/duration formatting | `client/src/__tests__/formatting.test.ts` (Phase 8, 33 tests) |
| Privacy tests | Constitution I compliance | `monitor/src/privacy.rs` (unit) + `monitor/tests/privacy_test.rs` (integration) |
| Crypto tests | Ed25519 keypair and signing | `monitor/src/crypto.rs` unit tests (Phase 6) |
| Sender tests | Event buffering and retry | `monitor/src/sender.rs` unit tests (Phase 6) |

### Unit Tests by Module

#### TypeScript/Client

**events.test.ts** (3 tests)
- Session event creation and validation
- Tool event creation and validation
- Event type enumeration validation

**formatting.test.ts** (33 tests - Phase 8)
- **formatTimestamp** (5 tests): Valid RFC 3339 parsing, timezone offsets, empty strings, invalid inputs, whitespace
- **formatTimestampFull** (4 tests): Full date-time formatting, timezone handling, error cases
- **formatRelativeTime** (7 tests): Minutes/hours/days/weeks ago, "just now", future timestamps, invalid input, edge cases
- **formatDuration** (8 tests): Human-readable duration formatting, hours/minutes/seconds combinations, zero/negative values, NaN handling, large durations
- **formatDurationShort** (9 tests): Digital clock format (H:MM:SS, M:SS, 0:SS), zero/negative/NaN handling, large values

**useEventStore.test.ts** (planned)
- Store initialization
- Event addition and FIFO eviction
- Session tracking
- Store selectors

**useWebSocket.test.ts** (planned - Phase 7)
- Connection establishment with token
- WebSocket message parsing and validation
- Exponential backoff retry logic
- Reconnection scheduling
- Event dispatch to store
- Cleanup on disconnect/unmount

**useSessionTimeouts.test.ts** (planned - Phase 10)
- Interval setup for session state transitions
- Active -> Inactive transition after 5 minutes
- Inactive/Ended -> Removed transition after 30 minutes
- Cleanup on unmount

**ConnectionStatus.test.tsx** (planned - Phase 7)
- Rendering with different connection statuses
- Label display based on props
- Custom className application
- ARIA accessibility attributes
- Color classes for each status

**TokenForm.test.tsx** (planned - Phase 7)
- Token input and save
- localStorage persistence
- Token clearing
- Cross-tab synchronization via storage events
- Button enable/disable states
- Callback invocation on token changes

**EventStream.test.tsx** (planned - Phase 8)
- Virtual scrolling rendering
- Auto-scroll behavior
- Jump to latest button
- Event type badges and icons
- Empty state display
- Accessibility attributes

**Heatmap.test.tsx** (planned - Phase 9)
- Color scale function (getHeatmapColor) returns correct colors for count thresholds
- Hour bucket key generation (getBucketKey) parses timestamps correctly
- Event counting by hour (countEventsByHour) aggregates correctly
- Cell generation (generateHeatmapCells) creates correct number of cells for 7/30-day views
- View toggle switches between 7-day and 30-day views
- Cell click calls onCellClick callback with correct start/end dates
- Tooltip displays on hover with correct count and datetime
- Empty state displays when no events
- Accessibility: grid/row/gridcell roles present
- Accessibility: aria-labels contain event count and datetime
- Keyboard navigation: Enter/Space activates cells

**SessionOverview.test.tsx** (planned - Phase 10)
- Rendering sessions from store
- Empty state when no sessions
- Activity indicators based on event count (pulse animations)
- Session sorting (active first, then by recency)
- Status badge display (Active/Idle/Ended)
- Event click handler invocation
- Keyboard navigation support (Enter/Space)
- Accessibility labels and roles

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
- **extract_basename tests** (8): Path parsing for various formats
- **PrivacyPipeline tests** (15): Event processing and context stripping
- **Edge case tests** (5): Complex scenarios

**crypto.rs** (14 unit tests - Phase 6)
- **Keypair generation** (1): Valid keypair creation
- **Persistence** (2): Save/load roundtrip, file existence
- **Signing** (6): Base64, raw signatures, determinism, verification
- **Error handling** (3): Load failures on invalid files
- **File permissions** (1): Unix-specific 0600/0644 validation
- **Format validation** (1): Base64 encoding verification

**sender.rs** (8 unit tests - Phase 6)
- **Buffering** (2): Adding events, FIFO eviction
- **Configuration** (1): Default values
- **Retry logic** (3): Jitter bounds, delay doubling, max cap
- **Buffer state** (2): Empty checks, reset

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
  2. `bash_commands_never_transmitted` - Command stripping verification
  3. `grep_patterns_never_transmitted` - Search pattern stripping
  4. `glob_patterns_never_transmitted` - File pattern stripping
  5. `websearch_never_transmits_context` - Search query stripping
  6. `webfetch_never_transmits_context` - URL stripping
  7. `summary_text_stripped` - Summary neutralization
  8. `all_event_types_safe` - Comprehensive multi-type test
  9. `allowlist_filtering_removes_sensitive_extensions` - Extension filtering
  10. Additional edge case and serialization tests

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
  - Crypto unit tests (cargo test -p vibetea-monitor crypto)
  - Sender unit tests (cargo test -p vibetea-monitor sender)
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
| Crypto unit tests | Yes | Security and correctness (Phase 6) |
| Sender unit tests | Yes | Reliability of event transmission (Phase 6) |
| Rust integration tests | Yes | End-to-end behavior |
| Privacy integration tests | Yes | Constitution I compliance in production scenarios |
| Code formatting matches | Yes | Consistency |

## Current Test Coverage

### TypeScript/Client

- **__tests__/events.test.ts**: 3 test cases for event type validation and creation
- **__tests__/formatting.test.ts**: 33 test cases for timestamp and duration formatting (Phase 8)
- Framework (Vitest) installed and ready
- Test organization structure established
- Planned tests for hooks and components (Phase 7)

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
- **crypto.rs** (Phase 6): 14 unit tests covering keypair generation, persistence, signing, validation
- **sender.rs** (Phase 6): 8 unit tests covering buffering, retry logic, jitter
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
cargo test -p vibetea-monitor crypto   # Crypto tests only (Phase 6)
cargo test -p vibetea-monitor sender   # Sender tests only (Phase 6)
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

# Crypto tests specifically (Phase 6)
cargo test -p vibetea-monitor crypto::tests

# Sender tests specifically (Phase 6)
cargo test -p vibetea-monitor sender::tests
```

## Next Steps for Testing

1. **TypeScript**: Create unit tests for `useSessionTimeouts` hook (Phase 10)
2. **TypeScript**: Create component tests for `SessionOverview` (Phase 10)
3. **TypeScript**: Create unit tests for `useEventStore` store
4. **TypeScript**: Create unit tests for `useWebSocket` hook (Phase 7)
5. **TypeScript**: Create component tests for `ConnectionStatus` (Phase 7)
6. **TypeScript**: Create component tests for `TokenForm` (Phase 7)
7. **TypeScript**: Create component tests for `EventStream` (Phase 8)
8. **Rust/Server**: Expand integration tests for HTTP routes and WebSocket functionality
9. **Rust/Monitor**: Add integration tests for file watching and JSONL parsing
10. **Rust/Monitor**: Add integration tests for crypto module (key persistence scenarios)
11. **Rust/Monitor**: Add integration tests for sender module (retry scenarios, rate limiting)
12. **Coverage**: Set up coverage reporting in CI/CD pipeline with threshold enforcement
13. **E2E**: Evaluate Playwright or Cypress for client workflow testing once UI is more complete
14. **Snapshot testing**: Consider for event serialization if JSON formats become complex
15. **Property-based testing**: Consider `proptest` for privacy module edge cases and path handling

---

## What Does NOT Belong Here

- Code style rules → CONVENTIONS.md
- Security testing details → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document describes HOW to test. Update when testing strategy changes.*
