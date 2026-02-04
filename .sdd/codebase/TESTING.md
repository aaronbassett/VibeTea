# Testing Strategy

**Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04 (Phase 5: Historic data UI and hook testing)

## Test Framework

### TypeScript/Client (Updated for Phase 5)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Vitest | `vite.config.ts` test config | In use |
| Component | @testing-library/react | jsdom environment | In use (Phase 5) |
| E2E | Not selected | TBD | Not started |
| Mocks | Mock Service Worker (MSW) | `client/src/mocks/` | New (Phase 5) |

### Rust/Server and Monitor (Phase 4)

| Type | Framework | Configuration | Status |
|------|-----------|---------------|--------|
| Unit | Rust built-in | `#[cfg(test)] mod tests` inline | In use |
| Integration | Rust built-in + wiremock | `tests/` directory with MockServer | In use (Phase 4) |
| E2E | Not selected | TBD | Not started |

### Running Tests

#### TypeScript/Client (Phase 5)

| Command | Purpose |
|---------|---------|
| `npm test` | Run all tests in watch mode |
| `npm run test` | Run all tests once |
| `npm run test:watch` | Run tests in watch mode |

#### Rust/Server and Monitor (Phase 4)

| Command | Purpose |
|---------|---------|
| `cargo test` | Run all tests in the workspace |
| `cargo test --lib` | Run library unit tests only |
| `cargo test --test '*'` | Run integration tests only |
| `cargo test -- --nocapture` | Run tests with println output |
| `cargo test -- --test-threads=1` | Run tests sequentially (prevents env var interference) |
| `cargo test -p vibetea-monitor persistence` | Run persistence module tests |
| `cargo test --test sender_recovery_test` | Run sender recovery integration tests |
| `cargo test -p vibetea-server unsafe_mode` | Run unsafe mode integration tests |

## Test Organization

### TypeScript/Client Directory Structure (Phase 5)

```
client/
├── src/
│   ├── __tests__/                  # Test directory
│   │   ├── App.test.tsx            # App component tests
│   │   ├── events.test.ts          # Event type tests
│   │   └── formatting.test.ts      # Utility tests
│   ├── hooks/                      # React hooks
│   │   ├── useEventStore.ts        # Zustand store hook
│   │   ├── useHistoricData.ts      # Historic data hook with auto-fetch
│   │   ├── useWebSocket.ts         # WebSocket connection hook
│   │   └── useSessionTimeouts.ts   # Session timeout logic
│   ├── mocks/                      # MSW mock setup (NEW Phase 5)
│   │   ├── handlers.ts             # MSW handlers for endpoints
│   │   ├── server.ts               # MSW server setup
│   │   ├── data.ts                 # Mock data factories
│   │   └── index.ts                # Barrel export
│   ├── types/
│   │   └── events.ts               # Event type definitions
│   └── App.tsx                     # Main app component
└── vite.config.ts                  # Vitest configuration
```

### Rust/Monitor Directory Structure (Phase 4)

```
monitor/
├── src/
│   ├── config.rs               # Config module with inline tests
│   ├── error.rs                # Error module with inline tests
│   ├── types.rs                # Types module with inline tests
│   ├── watcher.rs              # File watching implementation
│   ├── parser.rs               # JSONL parser implementation
│   ├── privacy.rs              # Privacy pipeline with 38 inline unit tests
│   ├── crypto.rs               # Ed25519 crypto with 14 inline unit tests (Phase 6)
│   ├── sender.rs               # HTTP sender with 8 inline unit tests (Phase 6)
│   ├── persistence.rs          # Event batching and persistence (Phase 4)
│   ├── lib.rs                  # Library entrypoint
│   └── main.rs                 # Binary entrypoint (CLI)
└── tests/
    ├── privacy_test.rs         # Integration tests for privacy compliance (17 tests)
    └── sender_recovery_test.rs # Sender retry logic with wiremock (Phase 4)
```

## Test Patterns

### MSW Handler Testing (Phase 5 - New)

Setup MSW server in tests to mock HTTP endpoints:

```typescript
// mocks/server.ts
import { setupServer } from 'msw/node';
import { queryHandlers } from './handlers';

export const server = setupServer(...queryHandlers);
```

Use in tests with beforeAll/afterEach/afterAll hooks:

```typescript
// Example test setup (if added to future hook tests)
import { server } from '../mocks/server';

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

Key MSW patterns:
1. **Server setup**: Use `setupServer(...handlers)` with handlers array
2. **Lifecycle**: `listen()` → tests → `resetHandlers()` → `close()`
3. **Request matching**: Match by method, path, headers, body
4. **Response templates**: Return `HttpResponse.json()` with status
5. **Handler override**: Use `server.use()` to override for specific tests

Example handler override in a test:

```typescript
import { server } from '../mocks/server';
import { http, HttpResponse } from 'msw';

it('handles API errors gracefully', () => {
  // Override handler for this test
  server.use(
    http.get('*/functions/v1/query', () => {
      return HttpResponse.json(
        { error: 'internal_error', message: 'Database query failed' },
        { status: 500 }
      );
    })
  );

  // Test that error is handled
});
```

### Component Testing with renderHook (Phase 5 - New)

Test React hooks and store interactions without full component renders:

```typescript
import { renderHook, act, waitFor } from '@testing-library/react';
import { useEventStore } from '../hooks/useEventStore';

describe('useEventStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useEventStore.setState({
      status: 'disconnected',
      events: [],
      sessions: new Map(),
      filters: { sessionId: null, timeRange: null },
    });
  });

  it('adds events to the buffer', () => {
    const { result } = renderHook(() => useEventStore());

    act(() => {
      result.current.addEvent(mockEvent);
    });

    expect(result.current.events).toHaveLength(1);
    expect(result.current.events[0]).toEqual(mockEvent);
  });

  it('enforces MAX_EVENTS limit', () => {
    const { result } = renderHook(() => useEventStore());

    act(() => {
      // Add more than MAX_EVENTS
      for (let i = 0; i < 1001; i++) {
        result.current.addEvent(createMockEvent(i));
      }
    });

    expect(result.current.events).toHaveLength(1000);
  });

  it('transitions sessions from active to inactive', () => {
    const { result } = renderHook(() => useEventStore());

    act(() => {
      result.current.addEvent(mockEvent);
    });

    const sessionId = mockEvent.payload.sessionId;
    expect(result.current.sessions.get(sessionId)?.status).toBe('active');

    // Simulate time passing and update session states
    act(() => {
      vi.useFakeTimers();
      vi.advanceTimersByTime(5 * 60 * 1001); // 5 minutes + 1 second
      result.current.updateSessionStates();
    });

    expect(result.current.sessions.get(sessionId)?.status).toBe('inactive');

    vi.useRealTimers();
  });
});
```

Key renderHook patterns:
1. **Reset state**: Use `setState()` in `beforeEach`
2. **Wrap updates in act()**: All state changes must be in `act()`
3. **Access hook result**: Use `result.current` to access hook values
4. **Test async effects**: Use `waitFor()` for async operations
5. **Fake timers**: Use `vi.useFakeTimers()` for time-dependent logic

### Store Direct Testing (Phase 5)

Test Zustand store state without rendering:

```typescript
// __tests__/App.test.tsx
import { useEventStore } from '../hooks/useEventStore';

beforeEach(() => {
  localStorage.clear();
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
  });
});

describe('Filter State Management', () => {
  it('updates session filter via store actions', () => {
    const { setSessionFilter, clearFilters } = useEventStore.getState();

    // Call action
    setSessionFilter('test-session-id');

    // Assert state changed
    expect(useEventStore.getState().filters.sessionId).toBe('test-session-id');

    // Clear filters
    clearFilters();
    expect(useEventStore.getState().filters.sessionId).toBeNull();
  });

  it('maintains time range filter independently', () => {
    const { setTimeRangeFilter, clearFilters } = useEventStore.getState();

    const start = new Date('2024-01-01T10:00:00Z');
    const end = new Date('2024-01-01T11:00:00Z');

    setTimeRangeFilter({ start, end });

    expect(useEventStore.getState().filters.timeRange).toEqual({ start, end });

    clearFilters();
    expect(useEventStore.getState().filters.timeRange).toBeNull();
  });
});
```

Key direct testing patterns:
1. **No rendering overhead**: Test logic without React component rendering
2. **Direct state access**: Use `getState()` to read state
3. **Direct action calls**: Call actions via `getState()`
4. **Deterministic tests**: No async timing or effect complications
5. **Fast feedback**: No component lifecycle overhead

### Zustand Store Mocking (Phase 5)

Mock the store in component tests:

```typescript
// __tests__/App.test.tsx
import { render, screen } from '@testing-library/react';
import { useEventStore } from '../hooks/useEventStore';

beforeEach(() => {
  // Reset store to clean initial state
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
    historicData: [],
    historicDataStatus: 'idle',
    historicDataFetchedAt: null,
    historicDataError: null,
  });
});

it('renders with initial store state', () => {
  render(<App />);

  // Component should use store's initial state
  expect(screen.getByText('VibeTea Dashboard')).toBeInTheDocument();
});

it('updates view when store state changes', () => {
  const { rerender } = render(<App />);

  // Update store state
  useEventStore.setState({ status: 'connected' });
  rerender(<App />);

  // Component should reflect new state
});

it('localStorage persists token across renders', () => {
  localStorage.setItem('vibetea_token', 'test-token-123');

  render(<App />);

  expect(screen.getByText('Sessions')).toBeInTheDocument();
});
```

Key mocking patterns:
1. **State reset**: Use `setState()` to set known initial state
2. **No vi.mock() needed**: Zustand hooks don't need explicit mocking
3. **Direct state mutation**: Modify store directly for test setup
4. **Render verification**: Assert component responds to store state
5. **localStorage mocking**: Vitest handles globals, just set/get items

### Component Testing with MSW (Phase 5)

Test components that fetch data:

```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { server } from '../mocks/server';
import App from '../App';

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

it('displays historic data when fetch succeeds', async () => {
  localStorage.setItem('vibetea_token', 'test-token');

  render(<App />);

  // Wait for async fetch to complete
  await waitFor(() => {
    expect(screen.getByText(/heatmap/i)).toBeInTheDocument();
  });
});

it('displays error message when fetch fails', async () => {
  localStorage.setItem('vibetea_token', 'invalid-token');

  // Override handler for this test
  server.use(
    http.get('*/functions/v1/query', () => {
      return HttpResponse.json(
        { error: 'invalid_token', message: 'Bearer token is invalid' },
        { status: 401 }
      );
    })
  );

  render(<App />);

  // Wait for error message
  await waitFor(() => {
    expect(screen.getByText(/invalid token/i)).toBeInTheDocument();
  });
});
```

Key component + MSW patterns:
1. **Server lifecycle**: Setup in `beforeAll`, reset in `afterEach`, close in `afterAll`
2. **Override handlers**: Use `server.use()` for test-specific behavior
3. **Async expectations**: Use `waitFor()` for fetch completion
4. **Error scenarios**: Test both success and error paths
5. **User interactions**: Test fetch triggering on user action

### Type Tests (Phase 5)

Test event type structure:

```typescript
// __tests__/events.test.ts
import { describe, it, expect } from 'vitest';
import type { VibeteaEvent, EventType } from '../types/events';

describe('Event Types', () => {
  it('should create a valid session event', () => {
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

    expect(event.type).toBe('session');
    expect(event.payload.action).toBe('started');
  });

  it('should support all event types', () => {
    const eventTypes: EventType[] = [
      'session',
      'activity',
      'tool',
      'agent',
      'summary',
      'error',
    ];

    expect(eventTypes).toHaveLength(6);
  });
});
```

Key type test patterns:
1. **Type instantiation**: Create typed objects to verify structure
2. **Discriminated unions**: Test type discrimination works
3. **Runtime validation**: Verify values match TypeScript types
4. **Type coverage**: Test all event variants

### Integration Tests (Rust) - Persistence Module (Phase 4)

Larger tests using wiremock to test event batching and retry logic:

```rust
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_oversized_event_does_not_block_normal_events() {
    let mock_server = MockServer::start().await;

    // First request: oversized chunk -> 413
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(413).set_body_string("Payload too large"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Second request: normal chunk -> 200
    Mock::given(method("POST"))
        .and(path("/events"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let mut sender = create_test_sender(&mock_server.uri());

    // Queue events: [normal, oversized, normal]
    sender.queue(create_small_event());
    sender.queue(create_oversized_event());
    sender.queue(create_small_event());

    // Flush should succeed overall
    let result = sender.flush().await;
    assert!(result.is_ok(), "Flush should succeed: {:?}", result);
    assert!(sender.is_empty(), "Buffer should be empty after flush");
}
```

Key patterns for Phase 4 integration tests:
1. **Helper functions**: Reusable test event creation
2. **wiremock MockServer**: Lightweight HTTP mocking without external services
3. **Async tests with #[tokio::test]**: Test async code patterns
4. **Response templates**: Match requests and return configured responses
5. **Scenario-based tests**: Test recovery paths (413, 500, timeout)
6. **Buffer verification**: Check buffer state after operations

## Mocking Strategy (Phase 5 Update)

### TypeScript/Client - MSW Mocks

MSW handlers intercept HTTP requests at the browser level:

```typescript
// mocks/handlers.ts
import { http, HttpResponse } from 'msw';

const queryHandler = http.get('*/functions/v1/query', ({ request }) => {
  // Extract and validate authorization
  const authHeader = request.headers.get('Authorization');
  if (authHeader === null) {
    return HttpResponse.json(
      { error: 'missing_auth', message: 'Authorization header required' },
      { status: 401 }
    );
  }

  // Extract and validate days parameter
  const url = new URL(request.url);
  const days = url.searchParams.get('days') ?? '7';

  if (!['7', '30'].includes(days)) {
    return HttpResponse.json(
      { error: 'invalid_days', message: 'days parameter must be 7 or 30' },
      { status: 400 }
    );
  }

  // Return mock data
  return HttpResponse.json(createQueryResponse(Number(days) as 7 | 30), {
    status: 200,
  });
});

export const queryHandlers = [queryHandler] as const;
```

### Zustand Store Mocking

Store is mocked by resetting state and injecting test data:

```typescript
beforeEach(() => {
  // Reset to clean state
  useEventStore.setState({
    status: 'disconnected',
    events: [],
    sessions: new Map(),
    filters: { sessionId: null, timeRange: null },
    historicData: [],
    historicDataStatus: 'idle',
    historicDataFetchedAt: null,
    historicDataError: null,
  });

  // Can also pre-populate with test data
  useEventStore.setState({
    status: 'connected',
    events: [mockEvent1, mockEvent2],
    sessions: new Map([['session-1', mockSession]]),
  });
});
```

### Mock Locations

| Mock Type | Location | Usage |
|-----------|----------|-------|
| HTTP handlers | `client/src/mocks/handlers.ts` | MSW handlers for endpoints |
| Mock data factories | `client/src/mocks/data.ts` | Generate test data |
| Server setup | `client/src/mocks/server.ts` | MSW server configuration |
| Store reset | `__tests__/setup.ts` (if added) | Reset store before tests |
| Fixtures | Test files inline | Small, test-specific data |

## Coverage Requirements (Phase 5 Update)

### Targets

| Metric | Target | Status |
|--------|--------|--------|
| Line coverage | 80% | Phase 5: Growing with client tests |
| Branch coverage | 75% | Phase 5: New hook logic branches |
| Function coverage | 80% | Phase 5: useHistoricData, store actions |

### New Coverage Areas (Phase 5)

- `useHistoricData`: Stale threshold detection, auto-fetch logic
- `useEventStore`: Historic data fetch, error handling
- MSW handlers: Token validation, query parameter validation
- Mock data factories: Data generation for various scenarios
- Store selectors: Event filtering, session selection

### Excluded Files

Files excluded from coverage:

- `types/` - Type definitions only
- `*.config.ts` - Configuration files
- `main.tsx` - App entry point
- `mocks/` - Test infrastructure (generate during tests)

## Test Categories (Phase 5 Update)

### Client Unit Tests (New for Phase 5)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `events.test.ts` | Event type structure | Vitest | Phase 5 |
| `formatting.test.ts` | Utility functions | Vitest | Phase 5 |

### Client Component Tests (New for Phase 5)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `App.test.tsx` | Token handling, filter state | @testing-library/react | Phase 5 |
| Hook tests (future) | useHistoricData, useEventStore | renderHook, MSW | Future |

### Persistence Tests (Phase 4)

| Test | Purpose | Framework | Status |
|------|---------|-----------|--------|
| `test_oversized_event_does_not_block_normal_events` | 413 handling | wiremock | Phase 4 |
| `test_multiple_oversized_events_all_skipped` | Batch cleanup | wiremock | Phase 4 |
| `test_normal_events_flush_successfully` | Happy path | wiremock | Phase 4 |
| `test_server_error_still_fails_flush` | 5xx handling | wiremock | Phase 4 |
| `test_batch_size_limit` | MAX_BATCH_SIZE | unit | Phase 4 |
| `test_buffer_overflow_evicts_oldest` | FIFO eviction | unit | Phase 4 |
| `test_retry_policy_exponential_backoff` | Backoff math | unit | Phase 4 |
| `test_jitter_within_bounds` | Jitter validation | unit | Phase 4 |

### Smoke Tests (Critical Path - Phase 5 Update)

Tests that must pass before any deploy:

| Test | Purpose | Location |
|------|---------|----------|
| Config tests | Configuration loading | `server/src/config.rs`, `monitor/src/config.rs` |
| Error tests | Error type safety | `server/src/error.rs` |
| Privacy tests | Constitution I compliance | `monitor/src/privacy.rs` + `privacy_test.rs` |
| Crypto tests | Ed25519 operations | `monitor/src/crypto.rs` |
| Sender tests | Event buffering | `monitor/src/sender.rs` + `sender_recovery_test.rs` |
| **Event type tests** | **Event structure validation** | **`client/src/__tests__/events.test.ts`** |
| **App token tests** | **Token persistence and UI** | **`client/src/__tests__/App.test.tsx`** |

## CI Integration

### Test Pipeline (Phase 5 Update)

```yaml
# Client tests
- Unit tests: npm test (vitest)
- Coverage: vitest --coverage

# Server/Monitor tests
- Unit tests: cargo test --lib
- Integration tests: cargo test --test '*'
- Parallel safety: cargo test -- --test-threads=1
```

### Required Checks

| Check | Blocking | Phase |
|-------|----------|-------|
| Client unit tests pass | Yes | Phase 5 |
| Client component tests pass | Yes | Phase 5 |
| Server unit tests pass | Yes | Phase 2 |
| Integration tests pass | Yes | Phase 4 |
| Coverage threshold met | Yes | Phase 4+ |
| MSW handlers verified | Yes | Phase 5 |

## Test Setup & Utilities

### Vitest Configuration (Phase 5)

```typescript
// vite.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  test: {
    globals: true,
    // Use jsdom only for component tests via inline config
    // Other tests use default node environment
    include: ['src/__tests__/**/*.test.{ts,tsx}'],
  },
});
```

Tests can specify environment inline:

```typescript
/**
 * @vitest-environment happy-dom
 */
import { render } from '@testing-library/react';
```

### Test Imports (Phase 5)

Standard test file structure:

```typescript
/**
 * Tests for [component/hook/function].
 *
 * @vitest-environment happy-dom
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom/vitest';

import { server } from '../mocks/server';
import { useEventStore } from '../hooks/useEventStore';
```

---

*This document describes HOW to test for Phase 5 (Historic data UI). Update when testing strategy changes.*
