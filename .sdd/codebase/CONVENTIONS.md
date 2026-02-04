# Coding Conventions

> **Purpose**: Document code style, naming conventions, error handling, and common patterns.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Code Style

### Rust Server

**Formatter**: `rustfmt` (default)

| Rule | Convention |
|------|------------|
| Indentation | 4 spaces (Rust default) |
| Line length | 100 characters (checked by clippy) |
| Trailing commas | Always in multiline structures |
| Module organization | Type definitions, then implementations |

**Commands**:
- Format: `cd server && cargo fmt`
- Lint: `cd server && cargo clippy -- -D warnings`

### TypeScript Client

**Prettier Configuration** (`client/.prettierrc`):

| Rule | Convention |
|------|------------|
| Indentation | 2 spaces |
| Quotes | Single quotes (`'string'`) |
| Semicolons | Always required |
| Trailing commas | ES5 (trailing commas in arrays/objects, not function params) |
| Line length | Automatic (Prettier default ~80 chars) |

**Commands**:
- Format: `cd client && npm run format`
- Check: `cd client && npm run format:check`
- Lint: `cd client && npm run lint`

### Git Commits

Format: `type(scope): description`

**Commit Types** (observed in codebase):
- `feat` - New feature
- `fix` - Bug fix
- `test` - Adding tests
- `style` - Code formatting/style changes
- `docs` - Documentation updates
- `chore` - Maintenance tasks

**Examples from codebase**:
```
feat(server): add session store, Supabase client, and auth errors
test(server): add auth privacy compliance tests
feat(supabase): add public_keys migration and edge function
style: apply code formatting
feat(client): add auth hook and page components (Phase 3)
```

## Naming Conventions

### Rust

**File naming**: `snake_case.rs`

| Type | Convention | Example |
|------|------------|---------|
| Modules | `snake_case` | `session.rs`, `supabase.rs` |
| Structs | `PascalCase` | `SessionStore`, `SupabaseClient` |
| Enums | `PascalCase` | `ServerError`, `SessionError` |
| Constants | `SCREAMING_SNAKE_CASE` | `DEFAULT_TTL_SECS`, `TOKEN_BYTES` |
| Functions | `snake_case` | `create_session()`, `validate_jwt()` |
| Methods | `snake_case` | `.validate_session()`, `.extend_session_ttl()` |
| Type aliases | `PascalCase` | `Result<T>` |

### TypeScript

**File naming**:
- Components: `PascalCase.tsx` (e.g., `EventStream.tsx`)
- Hooks: `camelCase.ts` with `use` prefix (e.g., `useWebSocket.ts`, `useAuth.ts`)
- Utilities: `camelCase.ts` (e.g., `formatting.ts`)
- Pages: `PascalCase.tsx` (e.g., `Login.tsx`, `Dashboard.tsx`)
- Tests: Same as source + `.test.ts` or `.test.tsx`

| Type | Convention | Example |
|------|------------|---------|
| Variables | `camelCase` | `eventStore`, `sessionToken`, `userDisplayName` |
| Constants | `SCREAMING_SNAKE_CASE` (global) | `DEFAULT_TIMEOUT`, `TOKEN_STORAGE_KEY` |
| Functions | `camelCase` | `formatDate()`, `validateToken()`, `hasStoredToken()` |
| Components | `PascalCase` | `<EventStream />`, `<ConnectionStatus />`, `<Login />` |
| Hooks | `camelCase` with `use` prefix | `useWebSocket`, `useEventStore`, `useAuth` |
| Interfaces | `PascalCase` | `VibeteaEvent`, `EventType`, `UseAuthReturn` |
| Types | `PascalCase` | `EventType`, `SessionMetadata`, `TimeRange` |

## Error Handling

### Rust Error Types

**Custom Errors** (in `server/src/error.rs`):

1. **`ConfigError`**: Configuration-related failures
   - Variants: `Missing`, `Invalid`, `FileError`

2. **`ServerError`**: Top-level application errors
   - Variants: `Config`, `Auth`, `JwtInvalid`, `SessionInvalid`, `Validation`, `RateLimit`, `SessionCapacityExceeded`, `SupabaseUnavailable`, `WebSocket`, `Internal`
   - Maps to HTTP status codes automatically via `IntoResponse`
   - Use `.auth()`, `.validation()`, `.jwt_invalid()` etc. helper methods

3. **`SessionError`**: Session store failures
   - Variants: `AtCapacity`, `NotFound`, `InvalidToken`

4. **`SupabaseError`**: Supabase API failures
   - Variants: `Unauthorized`, `Timeout`, `Unavailable`, `InvalidResponse`, `Configuration`, `RetriesExhausted`

**HTTP Status Mapping**:

| Error | Status | Note |
|-------|--------|------|
| `JwtInvalid`, `SessionInvalid` | 401 | Unauthorized |
| `Validation` | 400 | Bad Request |
| `RateLimit` | 429 | Includes Retry-After header |
| `SessionCapacityExceeded`, `SupabaseUnavailable` | 503 | Service Unavailable |
| `Config`, `Internal` | 500 | Server Error |

**Key Privacy Rule**: Never log sensitive data (JWTs, session tokens) even at TRACE level.

### TypeScript Error Handling

**Patterns**:
- Use `try/catch` for async operations
- Throw typed errors from services and hooks
- React components use ErrorBoundary for unhandled errors
- Hook errors logged with descriptive prefixes (e.g., `[useAuth]`, `[Dashboard]`)

**Example from Phase 3**:
```typescript
// In useAuth hook - catch errors and log safely
try {
  const { data, error } = await supabase.auth.getSession();
  if (error) {
    console.error('[useAuth] Failed to get session:', error.message);
    setUser(null);
    setSession(null);
  }
} catch (err) {
  console.error('[useAuth] Unexpected error during initialization:', err);
  setUser(null);
  setSession(null);
}
```

## Common Patterns

### Rust Modules

**Session Module** (`server/src/session.rs`):
- Thread-safe with `RwLock` for interior mutability
- Configuration via `SessionStoreConfig` struct with defaults
- Lazy cleanup on access plus optional background cleanup
- Token generation: 32 bytes random, base64-url encoded (43 chars)

**Supabase Client** (`server/src/supabase.rs`):
- Shareable via `Arc<SupabaseClient>`
- 5 second timeout for requests
- Exponential backoff retry: `2^attempt * 100ms + jitter`, capped at 10s
- Methods: `validate_jwt()` for one-off, `fetch_public_keys_with_retry()` for startup

**Error Module** (`server/src/error.rs`):
- Custom error types with `#[derive(Error)]`
- Helper methods: `.auth()`, `.validation()`, `.jwt_invalid()`, etc.
- Classification: `.is_client_error()`, `.is_server_error()`

### Rust Testing Patterns

**Test Structure** (Arrange-Act-Assert):
```rust
#[test]
fn test_name() {
    // Arrange
    let fixture = setup();

    // Act
    let result = fixture.operation();

    // Assert
    assert!(result.is_ok());
}
```

**Test Requirements**:
- Async tests use `#[tokio::test]`
- Tests touching `std::env` run with `cargo test --test-threads=1`
- Integration tests in `server/tests/` use `wiremock` for HTTP mocking

### TypeScript Patterns

**Component** (Page or UI):
```typescript
interface Props {
  events: VibeteaEvent[];
  onSelect?: (event: VibeteaEvent) => void;
}

export function EventStream({ events, onSelect }: Props) {
  return <div>{/* ... */}</div>;
}
```

**Hook** (Zustand store):
```typescript
export const useStore = create<StoreState>((set) => ({
  data: null,
  setData: (data) => set({ data }),
}));
```

**Hook** (Custom logic with state and lifecycle):
```typescript
export function useAuth(): UseAuthReturn {
  const [user, setUser] = useState<User | null>(null);
  const [session, setSession] = useState<Session | null>(null);
  const [loading, setLoading] = useState<boolean>(true);

  // Initialize and subscribe to auth changes
  useEffect(() => {
    const initializeAuth = async (): Promise<void> => {
      try {
        const { data, error } = await supabase.auth.getSession();
        if (error) {
          console.error('[useAuth] Failed to get session:', error.message);
          setUser(null);
          setSession(null);
        } else {
          setSession(data.session);
          setUser(data.session?.user ?? null);
        }
      } catch (err) {
        console.error('[useAuth] Unexpected error during initialization:', err);
        setUser(null);
        setSession(null);
      } finally {
        setLoading(false);
      }
    };

    // Set up listener and initialize
    const { data: { subscription } } = supabase.auth.onAuthStateChange((_event, newSession) => {
      setSession(newSession);
      setUser(newSession?.user ?? null);
      setLoading(false);
    });

    void initializeAuth();

    // Cleanup on unmount
    return () => {
      subscription.unsubscribe();
    };
  }, []);

  // Return stable callback references
  const signInWithGitHub = useCallback(async (): Promise<void> => {
    try {
      const { error } = await supabase.auth.signInWithOAuth({
        provider: 'github',
      });
      if (error) {
        console.error('[useAuth] GitHub sign-in failed:', error.message);
        throw error;
      }
    } catch (err) {
      console.error('[useAuth] Unexpected error during sign-in:', err);
      throw err;
    }
  }, []);

  return { user, session, loading, signInWithGitHub, signOut };
}
```

**Test** (Vitest):
```typescript
describe('Feature Name', () => {
  it('should do something', () => {
    const result = process(input);
    expect(result).toBe(expected);
  });
});
```

### Page Component Patterns (Phase 3)

Page components are page-level containers with:
- Required props documented in `Props` interface
- Internal state for local UI concerns (errors, loading during actions)
- Hooks for domain logic (useAuth, useEventStore, useWebSocket)
- Semantic HTML structure with ARIA attributes
- Error boundaries for animation components
- Spring-based animations with reduced motion respect

**Page Example** (`Login.tsx`):
```typescript
export interface LoginProps {
  readonly onAuthSuccess?: () => void;
}

export function Login({ onAuthSuccess }: LoginProps) {
  const { loading, signInWithGitHub } = useAuth();
  const prefersReducedMotion = useReducedMotion();

  const [isSigningIn, setIsSigningIn] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  const handleSignIn = useCallback(async () => {
    setError(null);
    setIsSigningIn(true);
    try {
      await signInWithGitHub();
      onAuthSuccess?.();
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to sign in';
      setError(message);
    } finally {
      setIsSigningIn(false);
    }
  }, [signInWithGitHub, onAuthSuccess]);

  if (loading) {
    return <LoadingState />;
  }

  return <AuthUI onSignIn={handleSignIn} error={error} />;
}
```

### Auth Hook Pattern (Phase 3)

The `useAuth` hook manages Supabase authentication state and provides:
- `user`: Current authenticated user or null
- `session`: Current Supabase session or null
- `loading`: Boolean for initial auth check (not per-action loading)
- `signInWithGitHub()`: Async function to initiate OAuth
- `signOut()`: Async function to sign out

**Key Features**:
- Initializes auth state via `supabase.auth.getSession()`
- Subscribes to auth state changes via `onAuthStateChange`
- Cleans up subscription on unmount
- Returns stable callback references via `useCallback`
- Logs errors with `[useAuth]` prefix (privacy-safe)
- No loading state updates after unmount

## Import Ordering

**Rust**:
1. Standard library
2. External crates
3. Internal modules
4. Re-exports

**TypeScript**:
1. React and external packages
2. Internal types (with `type` keyword)
3. Internal components and utilities
4. Hooks from utilities
5. Pages/Components from pages directory

## Documentation

### Rust

**Module docs** (`//!`):
```rust
//! Session token store for managing authenticated user sessions.
//!
//! This module provides an in-memory session store with TTL management.
```

**Item docs** (`///`):
- Arguments section with parameter descriptions
- Returns section for return values
- Errors section for error conditions
- Example section with code snippet

### TypeScript

**TSDoc**:
```typescript
/**
 * Formats a date into a readable string.
 * @param date - The date to format
 * @returns Formatted date string
 */
export function formatDate(date: Date): string { ... }

/**
 * Authentication hook for GitHub OAuth via Supabase.
 *
 * Manages authentication state including user, session, and loading states.
 * Automatically subscribes to auth state changes and handles session persistence
 * through Supabase's built-in session management (FR-004).
 *
 * @returns Object with user, session, loading state, and auth methods
 *
 * @example
 * ```tsx
 * function App() {
 *   const { user, session, loading, signInWithGitHub, signOut } = useAuth();
 *   if (!user) return <Login />;
 *   return <Dashboard />;
 * }
 * ```
 */
export function useAuth(): UseAuthReturn { ... }
```

## Key Learnings

### Phase 2: Privacy by Design
- Never log JWTs or session tokens, even at TRACE level
- Use integration tests with log capture to verify compliance
- Test file: `server/tests/auth_privacy_test.rs`

### Phase 2: Token Generation
- Use `rand::rng().fill_bytes()` for cryptographic randomness
- Base64-URL encoding without padding produces 43-char tokens from 32 bytes

### Phase 2: Session Store
- `RwLock` enables thread-safe read/write access patterns
- Lazy cleanup on validation, optional background cleanup sweep
- One-time TTL extension for WebSocket connections (grace period)

### Phase 2: Error Design
- Distinguish client errors (4xx) from server errors (5xx)
- Map custom error types to HTTP status codes via `IntoResponse`
- Include context in error messages for debugging

### Phase 3: Auth Hook Pattern
- Auth hooks manage Supabase state and OAuth flow
- Separate loading state during initial check from action-specific loading
- Use `onAuthStateChange` for persistent subscription
- Provide stable callback references with `useCallback`
- Use `useEffect` cleanup to unsubscribe on unmount

### Phase 3: Page Component Structure
- Page components are routing-level containers, not reusable components
- Manage local UI state (errors, loading flags) separately from domain state
- Use composition with error boundaries for animations
- Provide optional callbacks for parent routing concerns
- Respect reduced motion preferences consistently

---

*This document defines HOW to write code. Update when conventions change.*
