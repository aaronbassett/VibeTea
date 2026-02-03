# Coding Conventions

**Purpose**: Document code style, naming conventions, error handling, and common patterns.
**Generated**: 2026-02-03
**Last Updated**: 2026-02-03

## Code Style

### Formatting Tools

| Tool | Configuration | Command |
|------|---------------|---------|
| Prettier (TypeScript/Client) | `.prettierrc` | `npm run format` |
| ESLint (TypeScript/Client) | `eslint.config.js` | `npm run lint` |
| rustfmt (Rust/Server/Monitor) | Default settings | `cargo fmt` |
| clippy (Rust/Server/Monitor) | Default lints | `cargo clippy` |
| Deno fmt (Supabase Edge Functions) | Default settings | `deno fmt` |

### Style Rules

#### TypeScript/Client

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Enforced by Prettier |
| Quotes | Single quotes | `'string'` |
| Semicolons | Required | `const x = 1;` |
| Line length | No specific limit | Prettier handles wrapping |
| Trailing commas | ES5 style | Arrays/objects only (not function args) |
| JSX curly braces | Single-line only | `<Component />` or multi-line |

#### Rust/Server/Monitor

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 4 spaces | rustfmt default |
| Strings | Double quotes | `"string"` |
| Line length | 100 chars (soft) | rustfmt respects natural breaks |
| Comments | `//` for lines, `///` for docs | Doc comments on public items |
| Naming | snake_case for functions, PascalCase for types | `fn get_config()`, `struct Config` |

#### SQL (Database Migrations)

| Rule | Convention | Example |
|------|------------|---------|
| Identifiers | SCREAMING_SNAKE_CASE for table/column names | `public.events`, `event_type`, `created_at` |
| Keywords | UPPERCASE for SQL keywords | `CREATE TABLE`, `NOT NULL`, `PRIMARY KEY` |
| Comments | SQL block comments with descriptions | `-- Event identifier (format: evt_<20-char-suffix>)` |
| Indexing | Descriptive names with `idx_` prefix | `idx_events_timestamp`, `idx_events_source` |
| Constraints | Descriptive constraint names | `CHECK (event_type IN (...)` |
| Functions | snake_case for function names | `bulk_insert_events()`, `get_hourly_aggregates()` |
| Function parameters | snake_case | `days_back INTEGER`, `source_filter TEXT` |
| Return table columns | snake_case | `date DATE`, `hour INTEGER`, `event_count BIGINT` |

#### Deno/TypeScript (Edge Functions - Phase 3)

| Rule | Convention | Example |
|------|------------|---------|
| Indentation | 2 spaces | Standard TypeScript |
| Module imports | Use ESM with URL imports | `import * as ed from "https://esm.sh/@noble/ed25519@2.0.0"` |
| Error handling | Try-catch with null returns | Return `false`/`null` on error |
| Env vars | Use `Deno.env.get()` | `Deno.env.get("VIBETEA_PUBLIC_KEYS")` |
| Logging | `console.error()` for failures | `console.error("Signature verification error:", error)` |
| URL imports | Pin exact versions | `@noble/ed25519@2.0.0` not `@2` |
| JSDoc comments | Document all public functions | Include parameter types and return values |
| Readonly types | Use `readonly` in interfaces | `readonly isValid: boolean` |

## Naming Conventions

### TypeScript/Client

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `ConnectionStatus.tsx`, `TokenForm.tsx`, `EventStream.tsx`, `Heatmap.tsx`, `SessionOverview.tsx` |
| Hooks | camelCase with `use` prefix | `useEventStore.ts`, `useWebSocket.ts`, `useSessionTimeouts.ts` |
| Types | PascalCase in `types/` folder | `types/events.ts` contains `VibeteaEvent` |
| Utilities | camelCase | `utils/formatting.ts` |
| Constants | SCREAMING_SNAKE_CASE in const files | `MAX_EVENTS = 1000`, `TOKEN_STORAGE_KEY` |
| Test files | Same as source + `.test.ts` | `__tests__/events.test.ts`, `__tests__/formatting.test.ts` |
| Test directories | `__tests__/` at feature level | Co-located with related source |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Variables | camelCase | `sessionId`, `eventCount`, `wsRef`, `connectRef`, `displayEvents`, `isAutoScrollEnabled`, `recentEventCount`, `viewDays` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_EVENTS`, `DEFAULT_BUFFER_SIZE`, `TOKEN_STORAGE_KEY`, `ESTIMATED_ROW_HEIGHT`, `RECENT_EVENT_WINDOW_MS`, `LOW_ACTIVITY_THRESHOLD` |
| Functions | camelCase, verb prefix | `selectEventsBySession()`, `isSessionEvent()`, `parseEventMessage()`, `calculateBackoff()`, `formatTimestamp()`, `getEventDescription()`, `countRecentEventsBySession()`, `getActivityLevel()` |
| Classes | PascalCase (rare in modern React) | N/A |
| Interfaces | PascalCase, no `I` prefix | `EventStore`, `Session`, `VibeteaEvent`, `UseWebSocketReturn`, `ConnectionStatusProps`, `EventStreamProps`, `SessionOverviewProps`, `ActivityIndicatorProps` |
| Types | PascalCase | `VibeteaEvent<T>`, `EventPayload`, `ConnectionStatus`, `TokenStatus`, `EventType`, `ActivityLevel`, `SessionStatus`, `HourlyAggregate` |
| Type guards | `is` prefix | `isSessionEvent()`, `isValidEventType()` |
| Enums | PascalCase | N/A (use union types instead) |
| Refs | camelCase with `Ref` suffix | `wsRef`, `reconnectTimeoutRef`, `connectRef`, `parentRef`, `previousEventCountRef` |
| Records/Maps | PascalCase for type, camelCase for variable | `EVENT_TYPE_ICONS`, `EVENT_TYPE_COLORS`, `STATUS_CONFIG`, `PULSE_ANIMATIONS` (const) |

### Rust/Server/Monitor

#### Files & Directories

| Type | Convention | Example |
|------|------------|---------|
| Modules | snake_case | `config.rs`, `error.rs`, `types.rs`, `watcher.rs`, `parser.rs`, `privacy.rs`, `crypto.rs`, `sender.rs`, `main.rs` |
| Types | PascalCase | `Config`, `Event`, `ServerError`, `MonitorError`, `PrivacyConfig`, `Crypto`, `Sender`, `Command` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT`, `DEFAULT_BUFFER_SIZE`, `SENSITIVE_TOOLS`, `PRIVATE_KEY_FILE`, `SHUTDOWN_TIMEOUT_SECS` |
| Test modules | `#[cfg(test)] mod tests` | In same file as implementation |

#### Code Elements

| Type | Convention | Example |
|------|------------|---------|
| Functions | snake_case | `from_env()`, `generate_event_id()`, `parse_jsonl_line()`, `extract_basename()`, `parse_args()` |
| Constants | SCREAMING_SNAKE_CASE | `DEFAULT_PORT = 8080`, `SEED_LENGTH = 32`, `MAX_RETRY_DELAY_SECS = 60` |
| Structs | PascalCase | `Config`, `Event`, `PrivacyPipeline`, `Crypto`, `Sender`, `Command` |
| Enums | PascalCase | `EventType`, `SessionAction`, `ServerError`, `CryptoError`, `SenderError`, `Command` |
| Methods | snake_case | `.new()`, `.to_string()`, `.from_env()`, `.process()`, `.generate()`, `.load()`, `.save()`, `.sign()` |
| Lifetimes | Single lowercase letter | `'a`, `'static` |

### SQL (Database)

#### Identifiers

| Type | Convention | Example |
|------|------------|---------|
| Tables | snake_case | `events`, `sessions`, `aggregates` |
| Columns | snake_case | `event_type`, `session_id`, `timestamp`, `created_at`, `event_count` |
| Indexes | `idx_` prefix + descriptive name | `idx_events_timestamp`, `idx_events_source`, `idx_events_source_timestamp` |
| Functions | snake_case | `bulk_insert_events()`, `get_hourly_aggregates()` |
| Constraints | Type prefix or descriptive | `CHECK (event_type IN (...))` |

### Deno/TypeScript (Edge Functions - Phase 3)

#### Identifiers

| Type | Convention | Example |
|------|------------|---------|
| Functions | camelCase | `verifySignature()`, `getPublicKeyForSource()`, `validateBearerToken()` |
| Exports | camelCase for functions | `export async function verifyIngestAuth()` |
| Interfaces | PascalCase | `AuthResult`, `ErrorResponse`, `IngestResponse` |
| Error types | String-based or inline | Error messages in `error` field of `AuthResult` |
| Type unions | PascalCase, describe with `|` | `EventType` union of valid event types |
| Constants | SCREAMING_SNAKE_CASE | `MAX_BATCH_SIZE`, `EVENT_ID_PATTERN`, `RFC3339_PATTERN` |
| Environment variables | SCREAMING_SNAKE_CASE | `VIBETEA_PUBLIC_KEYS`, `SUPABASE_URL`, `VIBETEA_SUBSCRIBER_TOKEN` |
| HTTP endpoints | Lowercase with hyphens | `/ingest`, `/query`, `/health` |
| Request handlers | Handler function name | `handleRequest()`, `function handler()` |

## Error Handling

### Error Patterns

#### TypeScript

Client error handling uses:
- Try/catch for async operations
- Type guards for runtime validation (e.g., `isValidEventType()`)
- Discriminated unions for safe event handling (see Common Patterns section)
- Null checks with explicit error paths (e.g., `parseEventMessage()` returns `null` on invalid input)
- Console logging for error visibility (e.g., `console.error()`, `console.warn()`)

#### Rust/Server

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `server/src/config.rs` defines `ConfigError` |
| Authentication errors | String-based variant in `ServerError` | `ServerError::Auth(String)` |
| Validation errors | String-based variant in `ServerError` | `ServerError::Validation(String)` |
| Rate limiting | Struct variant with fields | `ServerError::RateLimit { source, retry_after }` |
| WebSocket errors | String-based variant in `ServerError` | `ServerError::WebSocket(String)` |
| Internal errors | String-based variant in `ServerError` | `ServerError::Internal(String)` |

#### Rust/Monitor

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Configuration errors | Custom enum with `#[derive(Error)]` | `monitor/src/config.rs` defines `ConfigError` |
| I/O errors | Use `#[from]` for automatic conversion | `MonitorError::Io(#[from] std::io::Error)` |
| JSON errors | Automatic conversion via `serde_json` | `MonitorError::Json(#[from] serde_json::Error)` |
| HTTP errors | String-based variants | `MonitorError::Http(String)` |
| Cryptographic errors | String-based variants | `CryptoError::InvalidKey`, `CryptoError::KeyExists` |
| Sender errors | Enum with specific variants | `SenderError::AuthFailed`, `SenderError::RateLimited`, `SenderError::MaxRetriesExceeded` |
| File watching errors | String-based variants | `MonitorError::Watch(String)` |
| JSONL parsing errors | String-based variants | `MonitorError::Parse(String)` |

#### Deno/TypeScript (Edge Functions - Phase 3)

| Scenario | Pattern | Example Location |
|----------|---------|------------------|
| Signature verification | Return `AuthResult` with error field | `supabase/functions/_shared/auth.ts` defines `AuthResult` |
| Missing headers | Return `AuthResult` with descriptive error | `verifyIngestAuth()` returns `{ isValid: false, error: "Missing X-Source-ID header" }` |
| Invalid tokens | Return `AuthResult` with specific error message | `verifyQueryAuth()` returns `{ isValid: false, error: "Invalid or missing bearer token" }` |
| JSON parsing errors | Return error response with specific code | `ingest` returns `{ error: "invalid_request", message: "Request body is not valid JSON" }` |
| Validation errors | Return response with 400/422 status based on error type | Event validation returns 400 for format errors, 422 for invalid event type |
| Logging errors | Use `console.error()` with context | `console.error("Signature verification error:", error)` |

### Error Response Format

#### TypeScript (Standard for client responses)

```typescript
{
  error: {
    code: 'ERROR_CODE',
    message: 'Human readable message',
    details?: object
  }
}
```

#### Deno/Edge Functions (AuthResult pattern - Phase 3)

```typescript
interface AuthResult {
  readonly isValid: boolean;
  readonly error?: string;
  readonly sourceId?: string;
}

// Success case
{ isValid: true, sourceId: "my-source" }

// Failure case
{ isValid: false, error: "Missing X-Source-ID header" }
```

#### Deno/Edge Functions (HTTP error response - Phase 3)

```typescript
interface ErrorResponse {
  readonly error: string;
  readonly message: string;
}

// Example response
{
  error: "invalid_event",
  message: "Event at index 0 has invalid id format 'bad-id'"
}
```

#### Rust Error Messages

Errors use `thiserror::Error` with `#[error]` attributes for automatic `Display` impl:

**Server Example** (`server/src/error.rs`):

```rust
#[derive(Debug)]
pub enum ServerError {
    Config(ConfigError),
    Auth(String),
    Validation(String),
    RateLimit { source: String, retry_after: u64 },
    WebSocket(String),
    Internal(String),
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(err) => write!(f, "configuration error: {err}"),
            Self::Auth(msg) => write!(f, "authentication failed: {msg}"),
            // ... other variants
        }
    }
}
```

**Monitor Example** (`monitor/src/error.rs`):

```rust
#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

**Crypto Example** (`monitor/src/crypto.rs` - Phase 6):

```rust
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("key file already exists: {0}")]
    KeyExists(String),
}
```

**Sender Example** (`monitor/src/sender.rs` - Phase 6):

```rust
#[derive(Error, Debug)]
pub enum SenderError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("server error: {status} - {message}")]
    ServerError { status: u16, message: String },

    #[error("authentication failed: invalid signature or source ID")]
    AuthFailed,

    #[error("rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("max retries exceeded after {attempts} attempts")]
    MaxRetriesExceeded { attempts: u32 },
}
```

### Logging Conventions

The `tracing` crate is used for structured logging across async Rust code:

| Level | When to Use | Example |
|-------|-------------|---------|
| error | Unrecoverable failures | `error!(unflushed_events = unflushed, "Some events could not be sent")` |
| warn | Recoverable issues | `warn!("Retry attempted")` |
| info | Important events | `info!("Starting VibeTea Monitor")` |
| debug | Development details | `debug!("Configuration loaded")` |

**Note**: Logging is initialized using `tracing_subscriber` with `EnvFilter` to control verbosity via `RUST_LOG` environment variable.

In TypeScript, use console methods with contextual prefixes:

```typescript
console.warn('[useWebSocket] No authentication token found in localStorage');
console.error('[useWebSocket] Connection error:', event);
console.error('[useWebSocket] Failed to create WebSocket:', error);
```

In Deno/Edge Functions, use `console.error()` for failures:

```typescript
console.error(`Invalid public key length: ${publicKey.length}, expected 32`);
console.error("Signature verification error:", error);
console.error("VIBETEA_PUBLIC_KEYS environment variable not set");
```

## Common Patterns

### Event-Driven Architecture

The codebase uses discriminated unions for type-safe event handling:

#### TypeScript

```typescript
// Type-safe event handling in TypeScript
type VibeteaEvent<T extends EventType = EventType> = {
  id: string;
  source: string;
  timestamp: string;
  type: T;
  payload: EventPayloadMap[T]; // Auto-typed based on T
};

// Type guards for runtime checks
function isSessionEvent(event: VibeteaEvent): event is VibeteaEvent<'session'> {
  return event.type === 'session';
}

// Validation type guard
function isValidEventType(value: unknown): value is EventType {
  return (
    typeof value === 'string' &&
    (VALID_EVENT_TYPES as readonly string[]).indexOf(value) !== -1
  );
}
```

#### Rust

```rust
// Rust equivalent with untagged enums
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventPayload {
    Tool { session_id: Uuid, tool: String, status: ToolStatus, ... },
    Session { session_id: Uuid, action: SessionAction, project: String },
    Activity { session_id: Uuid, project: Option<String> },
    // ... other variants, ordered from most specific to least specific
}

// Tagged wrapper for the full event
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    #[serde(rename = "type")]
    pub event_type: EventType,
    pub payload: EventPayload,
}
```

### Hourly Aggregate Type (Phase 11)

TypeScript interface for time-series data visualization:

```typescript
/**
 * Hourly aggregate of events for heatmap visualization.
 * Returned by the query edge function from get_hourly_aggregates().
 */
export interface HourlyAggregate {
  /** Monitor identifier */
  readonly source: string;
  /** Date in YYYY-MM-DD format (UTC) */
  readonly date: string;
  /** Hour of day 0-23 (UTC) */
  readonly hour: number;
  /** Count of events in this hour */
  readonly eventCount: number;
}
```

Naming conventions:
1. **Interface naming**: Singular noun describing the data (`HourlyAggregate`, not `HourlyAggregates`)
2. **Field naming**: camelCase for all fields (`eventCount`, `source`, `date`, `hour`)
3. **Readonly properties**: All interface properties are readonly to enforce immutability
4. **Field types**: Use simple types (string, number) for JSON compatibility
5. **Documentation**: Include JSDoc with usage context and field descriptions

### Deno Edge Function Authentication Pattern (Phase 3)

Secure authentication for Supabase Edge Functions:

**From `supabase/functions/_shared/auth.ts`**:

```typescript
/**
 * Result of authentication verification
 */
export interface AuthResult {
  readonly isValid: boolean;
  readonly error?: string;
  readonly sourceId?: string;
}

/**
 * Verify Ed25519 signature authentication for ingest endpoint
 *
 * @param request - The incoming Request object
 * @param body - The request body as a string (must be read before calling)
 * @returns AuthResult with validation status
 */
export async function verifyIngestAuth(
  request: Request,
  body: string
): Promise<AuthResult> {
  const sourceId = request.headers.get("X-Source-ID");
  const signature = request.headers.get("X-Signature");

  if (!sourceId) {
    return { isValid: false, error: "Missing X-Source-ID header" };
  }

  if (!signature) {
    return { isValid: false, error: "Missing X-Signature header" };
  }

  const publicKey = getPublicKeyForSource(sourceId);
  if (!publicKey) {
    return { isValid: false, error: `Unknown source: ${sourceId}` };
  }

  const message = new TextEncoder().encode(body);
  const isValid = await verifySignature(publicKey, signature, message);

  if (!isValid) {
    return { isValid: false, error: "Invalid signature" };
  }

  return { isValid: true, sourceId };
}

/**
 * Verify bearer token authentication for query endpoint
 *
 * @param request - The incoming Request object
 * @returns AuthResult with validation status
 */
export function verifyQueryAuth(request: Request): AuthResult {
  const authHeader = request.headers.get("Authorization");

  if (!validateBearerToken(authHeader)) {
    return { isValid: false, error: "Invalid or missing bearer token" };
  }

  return { isValid: true };
}
```

Key patterns:
1. **AuthResult interface**: Standard return type with `isValid` boolean and optional error message
2. **Header extraction**: Use `request.headers.get()` for case-insensitive header access
3. **Validation order**: Check required headers first, then delegate to validation functions
4. **Error messages**: Return descriptive errors for debugging without exposing implementation details
5. **Async function pattern**: Use `async` for Ed25519 verification with `@noble/ed25519`
6. **Environment variables**: Use `Deno.env.get()` for runtime configuration
7. **Readonly types**: Use `readonly` for immutable data structures in interfaces
8. **Return types**: Use union types or explicit result types (avoid throwing exceptions in auth)

### Deno Edge Function Validation Pattern (Phase 3)

Event validation using type discriminators and pattern matching:

**From `supabase/functions/ingest/index.ts`**:

```typescript
/**
 * Result of validating an event
 */
type EventValidationResult =
  | { readonly isValid: true; readonly event: Event }
  | { readonly isValid: false; readonly error: string; readonly errorCode: string };

/**
 * Validate a single event against the Event schema
 */
function validateEvent(value: unknown, index: number): EventValidationResult {
  if (typeof value !== "object" || value === null) {
    return {
      isValid: false,
      error: `Event at index ${index} must be an object`,
      errorCode: "invalid_event",
    };
  }

  const obj = value as Record<string, unknown>;

  // Validate id: string, pattern ^evt_[a-z0-9]{20}$
  if (typeof obj.id !== "string") {
    return {
      isValid: false,
      error: `Event at index ${index} missing required field 'id'`,
      errorCode: "invalid_event",
    };
  }
  if (!EVENT_ID_PATTERN.test(obj.id)) {
    return {
      isValid: false,
      error: `Event at index ${index} has invalid id format '${obj.id}'`,
      errorCode: "invalid_event",
    };
  }

  // Validate source: string, non-empty
  if (typeof obj.source !== "string" || obj.source.length === 0) {
    return {
      isValid: false,
      error: `Event at index ${index} has invalid source`,
      errorCode: "invalid_event",
    };
  }

  // ... additional field validations

  return {
    isValid: true,
    event: {
      id: obj.id,
      source: obj.source,
      timestamp: obj.timestamp,
      eventType: obj.eventType as EventType,
      payload: obj.payload as Record<string, unknown>,
    },
  };
}
```

Key patterns:
1. **Discriminated unions**: Use `{ isValid: true; event }` vs `{ isValid: false; error }` for type narrowing
2. **Early returns**: Return immediately on first validation error (fail-fast)
3. **Index tracking**: Include array index in error messages for debugging
4. **Error codes**: Return both human-readable message and machine-readable error code
5. **Type validation**: Check `typeof` before accessing properties
6. **Regex patterns**: Define patterns as constants at module level
7. **Field validation**: Validate each field independently with clear error messages
8. **Type casting**: Use `as` type assertions after validation confirms type safety

### Database Function Pattern (SQL)

PostgreSQL functions with proper naming and documentation:

```sql
-- Function: bulk_insert_events
-- Description: Atomic batch insertion of events from the ingest edge function
-- Parameters: events_json - JSONB array of event objects
-- Returns: Number of successfully inserted events
-- Note: Uses ON CONFLICT DO NOTHING for idempotency

CREATE OR REPLACE FUNCTION public.bulk_insert_events(events_json JSONB)
RETURNS TABLE(inserted_count BIGINT) AS $$
BEGIN
  RETURN QUERY
  WITH inserted AS (
    INSERT INTO public.events (id, source, timestamp, event_type, payload)
    SELECT
      (e->>'id')::TEXT,
      (e->>'source')::TEXT,
      (e->>'timestamp')::TIMESTAMPTZ,
      (e->>'eventType')::TEXT,
      e->'payload'
    FROM jsonb_array_elements(events_json) AS e
    ON CONFLICT (id) DO NOTHING
    RETURNING 1
  )
  SELECT COUNT(*)::BIGINT FROM inserted;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function: get_hourly_aggregates
-- Description: Retrieve hourly event counts for heatmap visualization
-- Parameters:
--   days_back - Number of days to look back (default: 7)
--   source_filter - Optional source filter (default: NULL for all sources)
-- Returns: Table of (source, date, hour, event_count) sorted by date/hour DESC

CREATE OR REPLACE FUNCTION public.get_hourly_aggregates(
  days_back INTEGER DEFAULT 7,
  source_filter TEXT DEFAULT NULL
)
RETURNS TABLE(
  source TEXT,
  date DATE,
  hour INTEGER,
  event_count BIGINT
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    e.source,
    DATE(e.timestamp AT TIME ZONE 'UTC') AS date,
    EXTRACT(HOUR FROM e.timestamp AT TIME ZONE 'UTC')::INTEGER AS hour,
    COUNT(*)::BIGINT AS event_count
  FROM public.events e
  WHERE
    e.timestamp >= NOW() - (days_back || ' days')::INTERVAL
    AND (source_filter IS NULL OR e.source = source_filter)
  GROUP BY e.source, DATE(e.timestamp AT TIME ZONE 'UTC'), EXTRACT(HOUR FROM e.timestamp AT TIME ZONE 'UTC')
  ORDER BY date DESC, hour DESC;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Grant execute permission to service role only
GRANT EXECUTE ON FUNCTION public.bulk_insert_events(JSONB) TO service_role;
GRANT EXECUTE ON FUNCTION public.get_hourly_aggregates(INTEGER, TEXT) TO service_role;
```

Key patterns:
1. **Function naming**: snake_case (PostgreSQL convention)
2. **Documentation**: Comments describing purpose, parameters, and return values
3. **Parameter names**: snake_case with descriptive names
4. **Return table columns**: Explicitly defined in RETURNS TABLE clause
5. **Default values**: Use LANGUAGE plpgsql with DEFAULT keyword
6. **Security**: SECURITY DEFINER limits execution to function owner
7. **Permissions**: GRANT EXECUTE to service_role only
8. **Type casting**: Explicit `::TYPE` casts for clarity
9. **NULL handling**: Allow NULL for optional parameters
10. **Comments**: Multi-line comments for complex logic

### Zustand Store Pattern (TypeScript)

Client state management uses Zustand with selector functions:

```typescript
export const useEventStore = create<EventStore>()((set) => ({
  // State
  status: 'disconnected',
  events: [],
  sessions: new Map(),

  // Actions (immutable updates)
  addEvent: (event: VibeteaEvent) => {
    set((state) => {
      // Calculate new state from current state
      const newEvents = [event, ...state.events].slice(0, MAX_EVENTS);
      // Return partial updates
      return { events: newEvents };
    });
  },
}));

// Selector utilities to extract derived state
export function selectActiveSessions(state: EventStore): Session[] {
  return Array.from(state.sessions.values()).filter(s => s.status !== 'ended');
}
```

### React Hook Pattern (TypeScript - Phase 7)

Custom React hooks follow a structured pattern with refs, callbacks, and effects:

**From `useWebSocket.ts`** (Phase 7):

```typescript
/**
 * WebSocket connection hook with auto-reconnect and exponential backoff.
 *
 * Manages WebSocket lifecycle and automatically dispatches received events
 * to the event store. Supports manual connection control and automatic
 * reconnection with exponential backoff (1s initial, 60s max, ±25% jitter).
 */
export function useWebSocket(url?: string): UseWebSocketReturn {
  // Refs for persistent values across renders
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectAttemptRef = useRef<number>(0);
  const shouldReconnectRef = useRef<boolean>(true);

  // Get store selectors and actions
  const addEvent = useEventStore((state) => state.addEvent);
  const setStatus = useEventStore((state) => state.setStatus);

  // Callbacks for event handlers
  const connect = useCallback(() => {
    // Establish connection logic
  }, [dependencies]);

  const disconnect = useCallback(() => {
    // Cleanup logic
  }, [dependencies]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      // Cleanup resources
    };
  }, [dependencies]);

  return { connect, disconnect, isConnected: status === 'connected' };
}
```

Key patterns:
1. **Refs**: Use for WebSocket instance, timers, and mutable state that shouldn't trigger re-renders
2. **Callbacks**: Wrap event handlers in `useCallback` to prevent infinite effect loops
3. **Effects**: Handle setup/cleanup, reconnection scheduling, and external subscriptions
4. **Derived state**: Return computed values like `isConnected: status === 'connected'`
5. **Documentation**: Include JSDoc with examples showing usage patterns

### React Component Pattern (TypeScript - Phase 7)

Functional components follow a consistent structure:

**From `ConnectionStatus.tsx`** (Phase 7):

```typescript
/**
 * Props for the ConnectionStatus component.
 */
interface ConnectionStatusProps {
  /** Whether to show the status text label. Defaults to false. */
  readonly showLabel?: boolean;
  /** Additional CSS classes to apply to the container. */
  readonly className?: string;
}

/**
 * Displays the current WebSocket connection status.
 *
 * Uses selective Zustand subscription to only re-render when status changes,
 * preventing unnecessary updates during high-frequency event streams.
 */
export function ConnectionStatus({
  showLabel = false,
  className = '',
}: ConnectionStatusProps) {
  // Selective subscription: only re-render when status changes
  const status = useEventStore((state) => state.status);

  const config = STATUS_CONFIG[status];

  return (
    <div className={`inline-flex items-center gap-2 ${className}`}>
      {/* Component JSX */}
    </div>
  );
}
```

Key patterns:
1. **Props interface**: Define props with JSDoc annotations for optional fields and defaults
2. **Selective subscriptions**: Use Zustand selectors to minimize re-renders
3. **Constants**: Define configuration objects outside components (e.g., `STATUS_CONFIG`)
4. **Accessibility**: Include ARIA attributes and semantic roles
5. **Tailwind classes**: Use utility-first approach for styling

### Form Handling Pattern (TypeScript - Phase 7)

Form components manage state and handle submissions:

**From `TokenForm.tsx`** (Phase 7):

```typescript
/**
 * Form for managing the authentication token.
 *
 * Provides a password input for entering the token, with save and clear buttons.
 * Token is persisted to localStorage for use by the WebSocket connection.
 */
export function TokenForm({
  onTokenChange,
  className = '',
}: TokenFormProps) {
  // State management
  const [tokenInput, setTokenInput] = useState<string>('');
  const [status, setStatus] = useState<TokenStatus>(() =>
    hasStoredToken() ? 'saved' : 'not-saved'
  );

  // Listen to storage changes from other tabs
  useEffect(() => {
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === TOKEN_STORAGE_KEY) {
        setStatus(event.newValue !== null ? 'saved' : 'not-saved');
      }
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  // Event handlers with proper types
  const handleSave = useCallback(
    (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      const trimmedToken = tokenInput.trim();
      if (trimmedToken === '') return;

      localStorage.setItem(TOKEN_STORAGE_KEY, trimmedToken);
      setStatus('saved');
      setTokenInput('');
      onTokenChange?.();
    },
    [tokenInput, onTokenChange]
  );

  return (
    <form onSubmit={handleSave}>
      {/* Form fields */}
    </form>
  );
}
```

Key patterns:
1. **useState with lazy init**: Use callback for initialization based on localStorage
2. **useCallback dependencies**: Include all dependencies to prevent stale closures
3. **Event types**: Use React event types like `React.FormEvent<HTMLFormElement>`
4. **Optional callbacks**: Use optional chaining with callbacks (`onTokenChange?.()`)
5. **localStorage handling**: Abstract into helper functions (e.g., `hasStoredToken()`)
6. **Cross-tab sync**: Listen to `storage` events for multi-tab consistency

### Virtual Scrolling Pattern (TypeScript - Phase 8)

Efficient rendering of large lists using `@tanstack/react-virtual`:

**From `EventStream.tsx`** (Phase 8):

```typescript
/**
 * Virtual scrolling event stream for displaying VibeTea events.
 *
 * Features:
 * - Efficient rendering of 1000+ events using virtual scrolling
 * - Auto-scroll to show new events (pauses when user scrolls up 50px+)
 * - Jump to latest button when auto-scroll is paused
 * - Event type icons and color-coded badges
 * - Accessible with proper ARIA attributes
 */
export function EventStream({ className = '' }: EventStreamProps) {
  // Selective subscription: only re-render when events change
  const events = useEventStore((state) => state.events);

  // Refs for persistent values across renders
  const parentRef = useRef<HTMLDivElement>(null);
  const previousEventCountRef = useRef<number>(events.length);

  // State for auto-scroll control
  const [isAutoScrollEnabled, setIsAutoScrollEnabled] = useState<boolean>(true);
  const [newEventCount, setNewEventCount] = useState<number>(0);

  // Reverse events for display (newest at bottom)
  const displayEvents = [...events].reverse();

  // Virtual scrolling setup
  const virtualizer = useVirtualizer({
    count: displayEvents.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ESTIMATED_ROW_HEIGHT,
    overscan: 5,
  });

  // Handle scroll detection to pause/resume auto-scroll
  const handleScroll = useCallback(() => {
    const scrollElement = parentRef.current;
    if (scrollElement === null) return;

    const { scrollTop, scrollHeight, clientHeight } = scrollElement;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceFromBottom > AUTO_SCROLL_THRESHOLD) {
      setIsAutoScrollEnabled(false);
    } else {
      if (!isAutoScrollEnabled) {
        setIsAutoScrollEnabled(true);
        setNewEventCount(0);
      }
    }
  }, [isAutoScrollEnabled]);

  // Auto-scroll to bottom when new events arrive (if enabled)
  useEffect(() => {
    const currentCount = events.length;
    const previousCount = previousEventCountRef.current;

    if (currentCount > previousCount) {
      const addedCount = currentCount - previousCount;

      if (isAutoScrollEnabled) {
        virtualizer.scrollToIndex(displayEvents.length - 1, { align: 'end' });
      } else {
        setNewEventCount((prev) => prev + addedCount);
      }
    }

    previousEventCountRef.current = currentCount;
  }, [events.length, isAutoScrollEnabled, displayEvents.length, virtualizer]);

  // Attach scroll listener with passive flag for performance
  useEffect(() => {
    const scrollElement = parentRef.current;
    if (scrollElement === null) return;

    scrollElement.addEventListener('scroll', handleScroll, { passive: true });
    return () => scrollElement.removeEventListener('scroll', handleScroll);
  }, [handleScroll]);

  // Render virtual items
  return (
    <div
      ref={parentRef}
      className="h-full overflow-auto"
      role="list"
      aria-label={`${displayEvents.length} events`}
    >
      <div style={{ height: `${virtualizer.getTotalSize()}px`, width: '100%', position: 'relative' }}>
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const event = displayEvents[virtualItem.index];
          if (event === undefined) return null;

          return (
            <div
              key={event.id}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <EventRow event={event} />
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

Key patterns:
1. **Virtual scrolling**: Use `@tanstack/react-virtual` for efficient rendering of 1000+ items
2. **Auto-scroll logic**: Track scroll position to detect user scrolling away from bottom
3. **Jump to latest**: Provide button to quickly return to new content
4. **Refs for state**: Use refs for previous state comparisons that don't trigger re-renders
5. **Passive scroll listeners**: Improve performance with `{ passive: true }` flag
6. **Absolute positioning**: Position virtual items with `transform: translateY()` for best performance
7. **Array reversal**: Reverse data only for display, keep storage in original order

### Formatting Utilities Pattern (TypeScript - Phase 8)

Pure functions for consistent formatting throughout the application:

**From `utils/formatting.ts`** (Phase 8):

```typescript
/**
 * Formats an RFC 3339 timestamp for display as time only (HH:MM:SS).
 * Uses the local timezone for display.
 *
 * @param timestamp - RFC 3339 formatted timestamp string
 * @returns Formatted time string or fallback for invalid input
 */
export function formatTimestamp(timestamp: string): string {
  const date = parseTimestamp(timestamp);
  if (date === null) return INVALID_TIMESTAMP_FALLBACK;

  const hours = padZero(date.getHours());
  const minutes = padZero(date.getMinutes());
  const seconds = padZero(date.getSeconds());

  return `${hours}:${minutes}:${seconds}`;
}

/**
 * Formats a duration in milliseconds as relative time.
 * Returns "just now", "5m ago", "2h ago", "yesterday", "3d ago", "2w ago".
 *
 * @param timestamp - RFC 3339 formatted timestamp string
 * @param now - Optional reference time (defaults to current time)
 * @returns Relative time string or fallback for invalid input
 */
export function formatRelativeTime(timestamp: string, now: Date = new Date()): string {
  const date = parseTimestamp(timestamp);
  if (date === null) return INVALID_RELATIVE_TIME_FALLBACK;

  const diffMs = now.getTime() - date.getTime();

  if (diffMs < MS_PER_MINUTE) return 'just now';
  if (diffMs < MS_PER_HOUR) {
    const minutes = Math.floor(diffMs / MS_PER_MINUTE);
    return `${minutes}m ago`;
  }
  // ... more time units

  return `${weeks}w ago`;
}

/**
 * Formats a duration in milliseconds to human-readable form.
 * Returns "1h 30m", "5m 30s", "30s" (only two most significant units).
 *
 * @param milliseconds - Duration in milliseconds
 * @returns Formatted duration string or fallback for invalid input
 */
export function formatDuration(milliseconds: number): string {
  if (typeof milliseconds !== 'number' || Number.isNaN(milliseconds)) {
    return INVALID_DURATION_FALLBACK;
  }

  if (milliseconds <= 0) return INVALID_DURATION_FALLBACK;

  const totalSeconds = Math.floor(milliseconds / MS_PER_SECOND);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  const parts: string[] = [];
  if (hours > 0) {
    parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
  } else if (minutes > 0) {
    parts.push(`${minutes}m`);
    if (seconds > 0) parts.push(`${seconds}s`);
  } else {
    parts.push(`${seconds}s`);
  }

  return parts.join(' ');
}

/**
 * Formats a duration in milliseconds to compact digital clock format.
 * Returns "1:30:00" (hours), "5:30" (minutes), "0:30" (seconds).
 *
 * @param milliseconds - Duration in milliseconds
 * @returns Compact duration string or fallback for invalid input
 */
export function formatDurationShort(milliseconds: number): string {
  if (typeof milliseconds !== 'number' || Number.isNaN(milliseconds)) {
    return INVALID_DURATION_SHORT_FALLBACK;
  }

  if (milliseconds <= 0) return INVALID_DURATION_SHORT_FALLBACK;

  const totalSeconds = Math.floor(milliseconds / MS_PER_SECOND);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;

  if (hours > 0) {
    return `${hours}:${padZero(minutes)}:${padZero(seconds)}`;
  }

  return `${minutes}:${padZero(seconds)}`;
}
```

Key conventions for formatting utilities:
1. **Pure functions**: No side effects, deterministic output for same inputs
2. **Graceful fallbacks**: Return sensible defaults for invalid input instead of throwing
3. **Type validation**: Check input types before processing (e.g., `typeof milliseconds !== 'number'`)
4. **Helper functions**: Extract common logic like `parseTimestamp()`, `padZero()`, `isSameDay()`
5. **Constants for magic numbers**: Define `MS_PER_SECOND`, `MS_PER_MINUTE`, etc.
6. **JSDoc with examples**: Document behavior, parameters, and return values with real examples
7. **Optional parameters**: Support reference times for testing (e.g., `now: Date = new Date()`)
8. **Consistent formatting**: Similar patterns across all formatting functions

### Unicode Emoji Icon Pattern (Phase 8)

Use Unicode escape sequences for emoji icons with clear fallbacks:

**From `EventStream.tsx`** (Phase 8):

```typescript
/** Icon mapping for each event type using Unicode escape sequences */
const EVENT_TYPE_ICONS: Record<EventType, string> = {
  tool: '\u{1F527}',      // 🔧 wrench
  activity: '\u{1F4AC}',  // 💬 speech bubble
  session: '\u{1F680}',   // 🚀 rocket
  summary: '\u{1F4CB}',   // 📋 clipboard
  error: '\u{26A0}\u{FE0F}', // ⚠️ warning
  agent: '\u{1F916}',     // 🤖 robot
};

// Usage in component
<span className="text-base" aria-hidden="true">
  {icon}
</span>
```

Key conventions:
1. **Unicode escape sequences**: Use `\u{...}` notation for better readability in source code
2. **Variation selectors**: Use `\u{FE0F}` for emoji style on multi-codepoint icons (⚠️)
3. **ARIA hidden**: Mark emoji as `aria-hidden="true"` since description is in text
4. **Consistent mapping**: Create lookup objects for all icon/variant combinations
5. **Clear comments**: Document actual emoji for quick reference during code review

### Event Type Description Pattern (Phase 8)

Type-safe extraction of event details using type assertions:

**From `EventStream.tsx`** (Phase 8):

```typescript
/**
 * Get a brief description of the event payload.
 * Uses type assertions to safely access payload properties based on event type.
 */
function getEventDescription(event: VibeteaEvent): string {
  const { type, payload } = event;

  switch (type) {
    case 'session': {
      // Type assertion: payload is guaranteed to be VibeteaEvent<'session'>['payload']
      const sessionPayload = payload as VibeteaEvent<'session'>['payload'];
      return `Session ${sessionPayload.action}: ${sessionPayload.project}`;
    }
    case 'tool': {
      const toolPayload = payload as VibeteaEvent<'tool'>['payload'];
      return `${toolPayload.tool} ${toolPayload.status}${
        toolPayload.context !== undefined ? `: ${toolPayload.context}` : ''
      }`;
    }
    case 'summary': {
      const summaryPayload = payload as VibeteaEvent<'summary'>['payload'];
      const summary = summaryPayload.summary;
      return summary.length > 80 ? `${summary.slice(0, 80)}...` : summary;
    }
    default:
      return 'Unknown event';
  }
}
```

Key patterns:
1. **Type narrowing with switch**: Use discriminated union type narrowing in switch statements
2. **Type assertions for payload**: Cast `payload as VibeteaEvent<T>['payload']` after type check
3. **Safe optional access**: Check `!== undefined` before using optional fields
4. **Truncation for display**: Limit string length for UI display (e.g., 80 chars for summaries)
5. **Default fallback**: Always have a default case to handle unexpected types gracefully

### CSS Grid Heatmap Pattern (TypeScript - Phase 9)

Creating activity heatmaps with CSS Grid for time-based data visualization:

**From `Heatmap.tsx`** (Phase 9):

```typescript
/**
 * Activity heatmap displaying event frequency over time.
 *
 * Uses CSS Grid with hours on X-axis, days on Y-axis.
 * Color intensity indicates event count per hour.
 */
export function Heatmap({ className = '', onCellClick }: HeatmapProps) {
  const events = useEventStore((state) => state.events);
  const [viewDays, setViewDays] = useState<ViewDays>(7);

  // Memoize event counts by hour bucket
  const eventCounts = useMemo(() => countEventsByHour(events), [events]);

  // Generate cells with memoization
  const cells = useMemo(
    () => generateHeatmapCells(viewDays, eventCounts),
    [viewDays, eventCounts]
  );

  return (
    <div
      role="grid"
      aria-label={`Activity heatmap showing ${viewDays} days`}
      className="grid gap-0.5"
      style={{
        gridTemplateColumns: `auto repeat(24, minmax(0, 1fr))`,
      }}
    >
      {/* Grid cells */}
    </div>
  );
}
```

**Hour Bucket Key Format**:

```typescript
/**
 * Create a bucket key for an event timestamp.
 * Uses local timezone for hour alignment.
 */
function getBucketKey(timestamp: string): string {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  const hour = String(date.getHours()).padStart(2, '0');
  return `${year}-${month}-${day}-${hour}`;
}
```

**Color Scale Function**:

```typescript
/**
 * Get heatmap color based on event count.
 */
function getHeatmapColor(count: number): string {
  if (count === 0) return '#1a1a2e';   // Dark (no activity)
  if (count <= 10) return '#2d4a3e';   // Low activity
  if (count <= 25) return '#3d6b4f';   // Medium activity
  if (count <= 50) return '#4d8c5f';   // High activity
  return '#5dad6f';                     // Very high activity
}
```

**View Toggle Pattern**:

```typescript
/**
 * View toggle with accessible role and state.
 */
function ViewToggle({ viewDays, onViewChange }: ViewToggleProps) {
  return (
    <div className="flex gap-1" role="group" aria-label="View range selector">
      {VIEW_OPTIONS.map((days) => (
        <button
          key={days}
          type="button"
          onClick={() => onViewChange(days)}
          className={viewDays === days ? 'bg-blue-600' : 'bg-gray-700'}
          aria-pressed={viewDays === days}
        >
          {days} Days
        </button>
      ))}
    </div>
  );
}
```

Key conventions for CSS Grid heatmaps:
1. **Grid template columns**: Use `auto repeat(N, minmax(0, 1fr))` for flexible cell sizing
2. **Contents display**: Use `display: contents` on row wrappers to maintain grid flow
3. **Inline color styles**: Use inline `backgroundColor` for dynamic color values
4. **Hour bucket keys**: Format as `YYYY-MM-DD-HH` for unique identification and sorting
5. **Local timezone**: Use `Date.getHours()` for user-expected hour alignment
6. **View toggle**: Use `role="group"` with `aria-pressed` for accessibility
7. **Cell keyboard nav**: Support Enter/Space for activation
8. **Memoization**: Memoize event counting and cell generation for performance

### Session Overview Pattern (TypeScript - Phase 10)

Component for displaying active AI assistant sessions with real-time activity indicators.

## Git Conventions

### Commit Messages

Format: `type(scope): description`

| Type | Usage | Example |
|------|-------|---------|
| feat | New feature | `feat(client): add event store` |
| fix | Bug fix | `fix(server): handle missing env var` |
| docs | Documentation | `docs: update conventions` |
| style | Formatting changes | `style: fix ESLint warnings` |
| refactor | Code restructure | `refactor(config): simplify validation` |
| test | Adding/updating tests | `test(client): add initial event type tests` |
| chore | Maintenance, dependencies | `chore: ignore TypeScript build artifacts` |

Examples with Phase 3 (Supabase Edge Functions):
- `feat(edge-functions): add ingest endpoint with Ed25519 signature auth`
- `feat(edge-functions): add query endpoint with bearer token auth`
- `feat(auth): implement shared auth utilities for edge functions`
- `test(edge-functions): add RLS integration tests for events table`
- `test(auth): add ingest and query authentication unit tests`

Examples with Phase 11:
- `feat(types): add HourlyAggregate type for heatmap data`
- `feat(db): add get_hourly_aggregates SQL function`

Examples with Phase 10:
- `feat(client): add SessionOverview component with activity indicators`
- `feat(client): add session state machine with timeout logic`

Examples with Phase 9:
- `feat(client): add Activity Heatmap component with color scale and accessibility`

Examples with Phase 8:
- `feat(client): add virtual scrolling event stream with auto-scroll`
- `feat(client): add formatting utilities for timestamps and durations`
- `test(client): add 33 tests for formatting utility functions`

Examples with Phase 7:
- `feat(client): add WebSocket connection hook with auto-reconnect`
- `feat(client): add connection status indicator component`
- `feat(client): add token form for authentication`

Examples with Phase 6:
- `feat(monitor): implement CLI with init and run commands`
- `feat(monitor): add HTTP sender with retry and buffering`
- `feat(monitor): add Ed25519 keypair generation and signing`

### Branch Naming

Format: `{type}/{ticket}-{description}`

Example: `feat/001-event-types`

---

## What Does NOT Belong Here

- Test strategies → TESTING.md
- Security practices → SECURITY.md
- Architecture patterns → ARCHITECTURE.md

---

*This document defines HOW to write code. Update when conventions change.*
