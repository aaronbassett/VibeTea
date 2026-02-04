# Testing Strategy

> **Purpose**: Document test frameworks, patterns, organization, and coverage requirements.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Test Frameworks

### Server (Rust)

| Type | Framework | Configuration |
|------|-----------|---------------|
| Unit | Rust built-in (`#[test]`) | Inline in source modules |
| Integration | Rust built-in + wiremock | `server/tests/*.rs` |
| Async | Tokio test runtime | `#[tokio::test]` |

**Dev Dependencies**:
- `tokio-test` - Tokio testing utilities
- `serial_test` - Serial test execution
- `wiremock` - HTTP mocking for Supabase API tests

### Client (TypeScript)

| Type | Framework | Configuration |
|------|-----------|---------------|
| Unit | Vitest | `client/src/__tests__/**/*.test.ts` |
| Component | Vitest + React Testing Library | `client/src/__tests__/**/*.test.tsx` |
| Visual | Storybook | `client/src/**/*.stories.ts` |

**Dev Dependencies**:
- `vitest` - Unit testing framework
- `@testing-library/react` - React component testing
- `@storybook/react` - Visual component development and documentation

## Running Tests

### Server

```bash
# Run all tests (required: --test-threads=1 for env-dependent tests)
cd server && cargo test --test-threads=1

# Run specific test module
cargo test session:: --test-threads=1

# Run with output (don't suppress println!)
cargo test -- --nocapture

# Run tests in release mode (faster)
cargo test --release --test-threads=1

# Watch mode (requires cargo-watch)
cargo watch -x 'test --test-threads=1'
```

### Client

```bash
# Run all tests
cd client && npm test

# Run tests in watch mode
npm run test:watch

# Run with coverage
npm test -- --coverage

# Run specific test file
npm test events.test.ts
```

## Test Organization

### Server Structure

```
server/
├── src/
│   ├── error.rs              # ~30 unit tests
│   ├── session.rs            # ~30 unit tests
│   ├── supabase.rs           # ~40 unit tests
│   └── [other modules]
└── tests/
    ├── auth_privacy_test.rs  # Privacy compliance tests (11 tests)
    └── [integration tests]
```

### Client Structure

```
client/
├── src/
│   ├── __tests__/
│   │   ├── events.test.ts
│   │   ├── formatting.test.ts
│   │   └── App.test.tsx
│   ├── components/
│   │   └── [components].tsx
│   ├── hooks/
│   │   └── [hooks].ts
│   └── [other modules]
└── vitest.config.ts
```

## Test Patterns

### Rust Unit Tests (Arrange-Act-Assert)

**Basic Pattern**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creates_session() {
        // Arrange
        let store = SessionStore::new(SessionStoreConfig::default());
        let user_id = "user-123".to_string();

        // Act
        let result = store.create_session(user_id, None);

        // Assert
        assert!(result.is_ok());
        let token = result.unwrap();
        assert_eq!(token.len(), 43);
    }
}
```

**Async Tests**:
```rust
#[tokio::test]
async fn test_validates_jwt() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .mount(&mock_server)
        .await;

    let client = SupabaseClient::new(mock_server.uri(), "test-key")
        .expect("should create client");
    
    let result = client.validate_jwt("test-jwt").await;
    assert!(result.is_ok());
}
```

**Environment-Dependent Tests**:
```rust
#[test]
#[serial_test::serial]
fn test_with_env_var() {
    std::env::set_var("TEST_VAR", "value");
    // test code
    std::env::remove_var("TEST_VAR");
    // Cleanup happens automatically with serial_test
}
```

### Rust Integration Tests

**Location**: `server/tests/auth_privacy_test.rs` (594 lines)

**Pattern**:
```rust
// Custom log capture infrastructure
#[derive(Clone, Default)]
struct LogCapture {
    logs: Arc<Mutex<Vec<String>>>,
}

// Run test code with log capture
fn with_log_capture<F>(test_fn: F) -> String
where
    F: FnOnce(),
{
    let capture = LogCapture::new();
    let layer = CaptureLayer::new(capture.clone());
    let subscriber = tracing_subscriber::registry()
        .with(layer.with_filter(LevelFilter::TRACE));
    tracing::subscriber::with_default(subscriber, test_fn);
    capture.get_logs()
}

// Assert sensitive data not in logs
#[test]
fn session_token_not_logged_on_creation() {
    let logs = with_log_capture(|| {
        let store = SessionStore::new(SessionStoreConfig::default());
        let token = store.create_session("user-123".into(), None).unwrap();
        CAPTURED_TOKENS.with(|t| t.borrow_mut().push(token));
    });
    
    CAPTURED_TOKENS.with(|t| {
        for token in t.borrow().iter() {
            assert!(!logs.contains(token), "Token leaked in logs!");
        }
    });
}
```

### TypeScript Unit Tests (Vitest)

**Component Test**:
```typescript
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { EventStream } from '../components/EventStream';

describe('EventStream', () => {
  it('renders events list', () => {
    const events = [
      {
        id: 'evt_1',
        type: 'session',
        payload: { action: 'started' }
      }
    ];

    render(<EventStream events={events} />);
    
    expect(screen.getByText(/session/i)).toBeInTheDocument();
  });
});
```

**Type Test**:
```typescript
import { describe, it, expect } from 'vitest';
import type { VibeteaEvent, EventType } from '../types/events';

describe('Event Types', () => {
  it('should create valid session event', () => {
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
});
```

## Test Categories

### Unit Tests

**Purpose**: Test individual functions/methods in isolation

**Scope**:
- Error handling and edge cases
- Data transformations
- Business logic validation

**Examples in codebase**:
- `server/src/error.rs`: 50+ unit tests for error types
- `server/src/session.rs`: 40+ unit tests for session store operations
- `server/src/supabase.rs`: 40+ unit tests for client operations
- `client/src/__tests__/events.test.ts`: 5+ tests for event types

### Integration Tests

**Purpose**: Test multiple components working together

**Scope**:
- API endpoint behavior
- Database interactions
- External service integration

**Examples in codebase**:
- `server/tests/auth_privacy_test.rs`: 11 tests for privacy compliance (594 lines)
- Tests cover JWT validation, session operations, log inspection

### Privacy Compliance Tests

**Purpose**: Verify sensitive data is never logged (Constitution I)

**Location**: `server/tests/auth_privacy_test.rs`

**Coverage**:
1. Session token creation - token not logged
2. Session validation - token not logged
3. Session expiry - tokens not logged
4. TTL extension - token not logged
5. Session removal - token not logged
6. JWT validation (success) - JWT not logged
7. JWT validation (failure) - JWT not logged
8. JWT validation (server error) - JWT not logged
9. Combined auth flow - neither JWT nor token leaked
10. SessionStore debug output - no tokens revealed
11. Capacity warnings - existing tokens not logged

**Test Approach**:
- Custom log capture layer at TRACE level
- Exercise all code paths that touch tokens
- Assert tokens/JWTs don't appear in captured logs

## Mocking Strategy

### Rust HTTP Mocking (wiremock)

**Setup**:
```rust
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{method, path, header};

#[tokio::test]
async fn test_with_mock() {
    let mock_server = MockServer::start().await;
    
    // Define mock expectation
    Mock::given(method("GET"))
        .and(path("/auth/v1/user"))
        .and(header("apikey", "test-anon-key"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"id": "user-123", "email": "test@example.com"}))
        )
        .mount(&mock_server)
        .await;
    
    // Create client pointing to mock
    let client = SupabaseClient::new(mock_server.uri(), "test-anon-key")
        .expect("should create client");
    
    // Test code using client
    let user = client.validate_jwt("valid-jwt").await.unwrap();
    assert_eq!(user.id, "user-123");
}
```

### TypeScript Mocking

**Pattern**: React Testing Library with vitest mocks

```typescript
import { vi } from 'vitest';

describe('Component with API', () => {
  it('handles API response', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ data: 'test' })
    });
    
    global.fetch = mockFetch;
    
    // test component
    
    expect(mockFetch).toHaveBeenCalledWith('/api/events');
  });
});
```

## Coverage Goals

### Rust

**Target Coverage**:
- Core modules (error, session, supabase): 80%+ line coverage
- Public APIs: 100% tested
- Error paths: All major error variants tested
- Privacy: All sensitive operations tested to ensure no logging

**Current Coverage**:
- `error.rs`: 50+ test cases covering all variants and HTTP mappings
- `session.rs`: 30+ test cases covering operations, expiration, extension, concurrent access
- `supabase.rs`: 40+ test cases covering all endpoints and error conditions
- `auth_privacy_test.rs`: 11 comprehensive integration tests

### TypeScript

**Target Coverage**:
- Utility functions: 90%+
- Components: Core user-facing components have tests
- Types: Type validation tests

**Current Coverage**:
- `events.test.ts`: Event type validation
- `formatting.test.ts`: Utility function tests
- `App.test.tsx`: Main component integration test

## Test Execution Commands

### CI/CD (GitHub Actions)

```bash
# Server: Run tests with serialization
cargo test --workspace --test-threads=1

# Client: Run tests and coverage
npm test
npm run format:check
npm run lint
```

### Local Development

```bash
# Server: Quick validation
cargo test --test-threads=1

# Server: With output
cargo test --test-threads=1 -- --nocapture

# Client: Watch mode
npm run test:watch

# All: Pre-commit check
cd server && cargo test --test-threads=1 && cd ../client && npm test
```

## Important Test Requirements

### Test Parallelism (Phase 3 Learning)

**Critical**: Environment variable tests must not run in parallel

```bash
# Wrong (tests interfere with each other)
cargo test

# Correct (serialized execution)
cargo test --test-threads=1

# Or use serial_test attribute
#[test]
#[serial_test::serial]
fn test_with_env_var() { ... }
```

**Why**: Tests that modify `std::env` variables affect the entire process. Without serialization, race conditions occur.

### Privacy Compliance (Phase 2 Learning)

**Requirement**: No authentication tokens should appear in logs

**Test Coverage**:
- Verify with log capture infrastructure
- Test at TRACE level (most verbose)
- Cover all code paths that handle tokens
- Verify both success and error cases

**Example from codebase**:
```rust
// Tests verify:
// 1. Session tokens not logged during creation
// 2. JWTs not logged during validation
// 3. Tokens not visible in Debug output
// 4. No leakage in error messages
```

---

*This document describes HOW to test. Update when testing strategy changes.*
