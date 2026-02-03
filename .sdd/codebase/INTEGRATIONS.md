# External Integrations

**Status**: Phase 4 Implementation - Event Batching with Async Persistence Manager
**Last Updated**: 2026-02-03

## Summary

VibeTea is designed as a distributed event system with four main components:
- **Monitor**: Captures Claude Code session events from local JSONL files, applies privacy sanitization, signs with Ed25519, and transmits to server or Supabase (with optional async event batching)
- **Server**: Receives, validates, verifies Ed25519 signatures, and broadcasts events via WebSocket
- **Client**: Subscribes to server events via WebSocket and optionally queries historical data from Supabase
- **Supabase**: Managed backend with PostgreSQL database for persistence, Edge Functions for authentication and data aggregation (Phase 3), and ingest endpoint for event persistence (Phase 4)

All integrations use standard protocols (HTTPS, WebSocket) with cryptographic message authentication and privacy-by-design data handling.

## File System Integration

### Claude Code JSONL Files

**Source**: `~/.claude/projects/**/*.jsonl`
**Format**: JSON Lines (one JSON object per line)
**Update Mechanism**: File system watcher via `notify` crate (inotify/FSEvents)

**Parser Location**: `monitor/src/parser.rs` (SessionParser, ParsedEvent, ParsedEventKind)
**Watcher Location**: `monitor/src/watcher.rs` (FileWatcher, WatchEvent)
**Privacy Pipeline**: `monitor/src/privacy.rs` (PrivacyConfig, PrivacyPipeline, extract_basename)

**Privacy-First Approach**:
- Only metadata extracted: tool names, timestamps, file basenames
- Never processes code content, prompts, or assistant responses
- File path parsing for project name extraction (slugified format)
- All event payloads pass through PrivacyPipeline before transmission

**Session File Structure**:
```
~/.claude/projects/<project-slug>/<session-uuid>.jsonl
```

**Supported Event Types** (from Claude Code JSONL):
| Claude Code Type | Parsed As | VibeTea Event | Fields Extracted |
|------------------|-----------|---------------|------------------|
| `assistant` with `tool_use` | Tool invocation | ToolStarted | tool name, context |
| `progress` with `PostToolUse` | Tool completion | ToolCompleted | tool name, success |
| `user` | User activity | Activity | timestamp only |
| `summary` | Session end marker | Summary | session metadata |
| File creation | Session start | SessionStarted | project from path |

**Watcher Behavior**:
- Monitors `~/.claude/projects/` directory recursively
- Detects file creation, modification, deletion events
- Maintains position map for efficient tailing (no re-reading)
- Emits WatchEvent::FileCreated, WatchEvent::LinesAdded, WatchEvent::FileRemoved
- Automatic cleanup of removed files from tracking state

**Configuration** (`monitor/src/config.rs`):
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude Code directory to monitor |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | Comma-separated file extensions to watch |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |

## Privacy & Data Sanitization

### Privacy Pipeline Architecture

**Location**: `monitor/src/privacy.rs` (1039 lines)

**Core Components**:

1. **PrivacyConfig** - Configuration management
   - Optional extension allowlist (e.g., `.rs`, `.ts`)
   - Loaded from `VIBETEA_BASENAME_ALLOWLIST` environment variable
   - Supports comma-separated format: `.rs,.ts,.md` or `rs,ts,md` (auto-dots)
   - Whitespace-tolerant: ` .rs , .ts ` normalized to `[".rs", ".ts"]`
   - Empty entries filtered: `.rs,,.ts,,,` becomes `[".rs", ".ts"]`

2. **PrivacyPipeline** - Event sanitization processor
   - Processes EventPayload before transmission to server
   - Strips sensitive contexts from dangerous tools
   - Extracts basenames from file paths
   - Applies extension allowlist filtering
   - Neutralizes session summary text

3. **extract_basename()** - Path safety function
   - Converts full paths to secure basenames
   - Handles Unix: `/home/user/src/auth.ts` → `auth.ts`
   - Handles Windows: `C:\Users\user\src\auth.ts` → `auth.ts`
   - Handles relative: `src/auth.ts` → `auth.ts`
   - Returns `None` for invalid/empty paths

**Sensitive Tools** (context always stripped):
- `Bash` - Commands may contain secrets, passwords, API keys
- `Grep` - Patterns reveal what user is searching for
- `Glob` - File patterns reveal project structure
- `WebSearch` - Queries reveal user intent
- `WebFetch` - URLs may contain sensitive parameters

**Privacy Processing Rules**:

| Payload Type | Processing |
|--------------|-----------|
| Session | Pass through (project already sanitized at parse time) |
| Activity | Pass through unchanged |
| Tool (sensitive) | Context set to `None` |
| Tool (other) | Context → basename, apply allowlist, pass if allowed else `None` |
| Agent | Pass through unchanged |
| Summary | Summary text replaced with "Session ended" |
| Error | Pass through (category already sanitized) |

**Extension Allowlist Filtering**:
- When `VIBETEA_BASENAME_ALLOWLIST` is not set: All extensions allowed
- When set to `.rs,.ts`: Only `.rs` and `.ts` files transmitted; others filtered to `None`
- If no extension and allowlist set: Context filtered to `None`
- Examples:
  - `file.rs` with allowlist `.rs,.ts` → ALLOWED
  - `file.py` with allowlist `.rs,.ts` → FILTERED
  - `Makefile` with allowlist `.rs,.ts` → FILTERED (no extension)

**Example Privacy Processing**:
```
Input:  Tool { context: Some("/home/user/project/src/auth.rs"), tool: "Read", ... }
Output: Tool { context: Some("auth.rs"), tool: "Read", ... }

Input:  Tool { context: Some("rm -rf /home"), tool: "Bash", ... }
Output: Tool { context: None, tool: "Bash", ... }  # Sensitive tool

Input:  Tool { context: Some("/home/user/config.py"), tool: "Read", allowlist: [.rs,.ts] }
Output: Tool { context: None, tool: "Read", ... }  # Filtered by allowlist
```

### Privacy Test Suite

**Location**: `monitor/tests/privacy_test.rs` (951 lines)

**Coverage**: 18+ comprehensive privacy compliance tests
**Validates**: Constitution I (Privacy by Design)

**Test Categories**:
1. **Path Sanitization**
   - No full paths in output (Unix, Windows, relative)
   - Basenames correctly extracted
   - Hidden files handled

2. **Sensitive Tool Stripping**
   - Bash commands removed entirely
   - Grep patterns omitted
   - Glob patterns stripped
   - WebSearch queries removed
   - WebFetch URLs removed

3. **Content Stripping**
   - File contents never transmitted
   - Diffs excluded from payloads
   - Code excerpts removed

4. **Prompt/Response Stripping**
   - User prompts not included
   - Assistant responses excluded
   - Message content sanitized

5. **Command Argument Removal**
   - Arguments separated from descriptions
   - Descriptions allowed for Bash context
   - Actual commands never sent

6. **Summary Neutralization**
   - Summary text set to generic "Session ended"
   - Original text discarded
   - No content leakage

7. **Extension Allowlist Filtering**
   - Correct files allowed through
   - Disallowed extensions filtered
   - No-extension files handled properly

8. **Sensitive Pattern Detection**
   - Path patterns never appear (e.g., `/home/`, `/Users/`, `C:\`)
   - Command patterns removed (e.g., `rm -rf`, `sudo`, `curl -`, `Bearer`)
   - Credentials not transmitted

## Cryptographic Authentication & Key Management

### Phase 6: Monitor Cryptographic Operations

**Module Location**: `monitor/src/crypto.rs` (438 lines)

**Crypto Module Features**:

1. **Keypair Generation**
   - `Crypto::generate()` creates new Ed25519 keypair
   - Uses OS cryptographically secure RNG via `rand` crate
   - Returns Crypto struct managing SigningKey

2. **Key Persistence**
   - `save(dir)` writes keypair to files
   - Private key: `key.priv` (raw 32-byte seed, permissions 0600)
   - Public key: `key.pub` (base64-encoded, permissions 0644)
   - Creates directory if not present
   - Error on invalid file permissions (Unix)

3. **Key Loading**
   - `load(dir)` reads existing keypair
   - Validates private key is exactly 32 bytes
   - Returns CryptoError if format invalid
   - Reconstructs SigningKey from seed bytes

4. **Key Existence Check**
   - `exists(dir)` checks if private key file present
   - Used to prevent accidental overwrite

5. **Public Key Export**
   - `public_key_base64()` returns base64-encoded public key
   - Format suitable for `VIBETEA_PUBLIC_KEYS` environment variable
   - Derived from SigningKey via VerifyingKey

6. **Event Signing**
   - `sign(message)` returns base64-encoded Ed25519 signature
   - Message is JSON-encoded event payload (bytes)
   - Signature verifiable by server with public key
   - Uses RFC 8032 compliant signing via ed25519-dalek

**CryptoError Types**:
- `Io` - File system errors
- `InvalidKey` - Seed not 32 bytes or malformed
- `Base64` - Public key decoding error
- `KeyExists` - Files already present (can be overwritten)

**File Locations** (configurable):
- Default key directory: `~/.vibetea/`
- Override with `VIBETEA_KEY_PATH` environment variable
- Private key: `{key_dir}/key.priv`
- Public key: `{key_dir}/key.pub`

### Monitor → Server Authentication

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Ed25519 digital signatures | Rust `ed25519-dalek` crate |
| **Protocol** | HTTPS POST with signed payload | Event signatures in X-Signature header |
| **Key Management** | Source-specific public key registration | `VIBETEA_PUBLIC_KEYS` env var |
| **Key Format** | Base64-encoded Ed25519 public keys | `source1:pubkey1,source2:pubkey2` |
| **Verification** | Constant-time comparison using `subtle` crate | `server/src/auth.rs` |
| **Flow** | Monitor signs event → Server validates signature | `server/src/auth.rs`, `server/src/routes.rs` |
| **Fallback** | Unsafe no-auth mode (dev only) | `VIBETEA_UNSAFE_NO_AUTH=true` |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_PUBLIC_KEYS` (required unless unsafe mode enabled)
- Parses `VIBETEA_UNSAFE_NO_AUTH` (dev-only authentication bypass)
- Validates on every server startup with comprehensive error messages
- Supports multiple comma-separated source:key pairs

**Example Key Format**:
```
VIBETEA_PUBLIC_KEYS=monitor-prod:dGVzdHB1YmtleTEx,monitor-dev:dGVzdHB1YmtleTIy
```

**Implementation Details**:
- Uses `HashMap<String, String>` to map source_id to base64-encoded keys
- Public keys stored in plain text (no decryption needed)
- Empty public_keys map allowed if unsafe_no_auth is enabled
- Error handling with ConfigError enum for missing/invalid formats
- Constant-time comparison prevents timing attacks on signature verification

**Signature Verification Process** (`server/src/auth.rs`):
- Decode base64 signature from X-Signature header
- Decode base64 public key from configuration
- Extract Ed25519 VerifyingKey from public key bytes
- Use `ed25519_dalek::Signature::verify()` for verification
- Apply `subtle::ConstantTimeEq` to compare results

### Monitor → Supabase Edge Functions Authentication (Phase 3)

**Module Location**: `supabase/functions/_shared/auth.ts` (173 lines)

**Authentication Methods**:

1. **Ed25519 Signature Verification** (for ingest endpoint)
   - Library: `@noble/ed25519` version 2.0.0 (RFC 8032 compliant)
   - Signature in `X-Signature` header (base64-encoded)
   - Source ID in `X-Source-ID` header
   - Public keys from `VIBETEA_PUBLIC_KEYS` environment variable
   - Format: `source_id:public_key_base64,source_id2:public_key_base64`
   - Verifies using `ed.verifyAsync(signature, message, publicKey)`
   - Validates key length (32 bytes) and signature length (64 bytes)
   - Returns AuthResult with isValid flag and optional error/sourceId

2. **Bearer Token Validation** (for query endpoint)
   - Token in `Authorization: Bearer <token>` header
   - Validated against `VIBETEA_SUBSCRIBER_TOKEN` environment variable
   - Constant-time comparison to prevent timing attacks
   - Returns AuthResult with isValid flag

**Shared Auth Module** (`supabase/functions/_shared/auth.ts`):

Functions exported:
- `verifySignature(publicKeyBase64, signatureBase64, message)`: Verifies Ed25519 signature
- `getPublicKeyForSource(sourceId)`: Lookup public key from environment
- `validateBearerToken(authHeader)`: Validates bearer token
- `verifyIngestAuth(request, body)`: Combined verification for ingest endpoint
- `verifyQueryAuth(request)`: Token validation for query endpoint

**Key Validation**:
- Public key must be exactly 32 bytes (base64 decoded)
- Signature must be exactly 64 bytes (base64 decoded)
- Returns validation errors if formats incorrect
- Graceful error handling with console logging

**Phase 3 Ingest Function** (`supabase/functions/ingest/index.ts`):
- Accepts POST requests with up to 1000 events per batch
- Validates event schema: id (evt_*), source, timestamp (RFC 3339), eventType, payload
- Validates event source matches X-Source-ID header
- Calls `bulk_insert_events()` RPC for database persistence
- CORS support with configurable methods and headers
- Comprehensive error responses: validation errors (400), auth errors (401), type errors (422)

**Phase 3 Query Function** (`supabase/functions/query/index.ts`):
- Accepts GET requests with query parameters (days: 7|30, source: optional)
- Validates query parameters and bearer token
- Calls `get_hourly_aggregates()` RPC for historical data
- Returns JSON response with aggregates and metadata

**Phase 3 Test Infrastructure**:
- `ingest/index.test.ts`: Tests Ed25519 signature authentication with test keypairs
- `query/index.test.ts`: Tests bearer token validation with EnvGuard environment isolation
- `_tests/rls.test.ts`: Tests Row Level Security enforcement

**Client Authentication (Server → Client)**

| Aspect | Details | Configuration |
|--------|---------|---------------|
| **Method** | Bearer token in WebSocket headers | Static token per deployment |
| **Protocol** | WebSocket upgrade with `Authorization: Bearer <token>` | Client sends on connect |
| **Token Type** | Opaque string (no expiration in Phase 4) | `VIBETEA_SUBSCRIBER_TOKEN` env var |
| **Scope** | All clients use the same token | No per-user differentiation |
| **Validation** | Server-side validation only | In-memory, no persistence |

**Configuration Location**: `server/src/config.rs`
- Parses `VIBETEA_SUBSCRIBER_TOKEN` (required unless unsafe mode enabled)
- Token required for all WebSocket connections
- No token refresh mechanism in Phase 5
- Stored as `Option<String>` in Config struct

**Future Enhancements**: Per-user tokens, token expiration, refresh tokens

## Database Integration (Phase 3)

### Supabase PostgreSQL

**Service**: Supabase managed PostgreSQL 17
**Connection**: Port 54322 (local), auto-configured on hosted platform
**Authentication**: Service role key for Edge Functions, RLS for client access

**Database Schema** (`supabase/migrations/20260203000000_create_events_table.sql`):

**Events Table**:
| Column | Type | Constraints | Purpose |
|--------|------|-------------|---------|
| `id` | TEXT | PRIMARY KEY | Event identifier (evt_<20-char>) |
| `source` | TEXT | NOT NULL | Monitor source identifier |
| `timestamp` | TIMESTAMPTZ | NOT NULL | Event occurrence time |
| `event_type` | TEXT | NOT NULL, CHECK | Discriminated type: session, activity, tool, agent, summary, error |
| `payload` | JSONB | NOT NULL | Full event payload (privacy-filtered by monitor) |
| `created_at` | TIMESTAMPTZ | DEFAULT NOW() | Server-side persistence timestamp |

**Indexes**:
- `idx_events_timestamp` - For efficient time-range queries (descending, newest-first)
- `idx_events_source` - For source filtering (optional, multi-tenant queries)
- `idx_events_source_timestamp` - Composite for common source+time pattern

**Row Level Security**:
- Enabled with implicit deny-all policy
- Force RLS enabled (defense in depth)
- No policies defined = service_role only access
- Clients cannot directly query events table (use Edge Functions instead)

**PostgreSQL Functions** (`supabase/migrations/20260203000001_create_functions.sql`):

1. **bulk_insert_events(events_json JSONB)** - Batch event insertion
   - Accepts: JSONB array of event objects
   - Parses fields: id, source, timestamp, eventType, payload
   - Returns: COUNT(*)::BIGINT of successfully inserted events
   - Idempotency: ON CONFLICT (id) DO NOTHING
   - Language: plpgsql
   - Grants: EXECUTE to service_role only

2. **get_hourly_aggregates(days_back INT DEFAULT 7, source_filter TEXT DEFAULT NULL)** - Aggregation for heatmap
   - Returns: (source, date, hour, event_count) grouped by hour
   - Parameters:
     - `days_back`: Number of days to look back (default 7, supports 30)
     - `source_filter`: Optional source identifier (NULL = all sources)
   - Calculation: COUNT(*) of events grouped by hour in UTC timezone
   - Ordering: date DESC, hour DESC (newest first)
   - Language: plpgsql
   - Grants: EXECUTE to service_role only

**Connection Details**:
- Database URL format: `postgresql://user:password@host:54322/postgres`
- Local development: `http://127.0.0.1:54322` (see supabase/config.toml)
- Production: Managed by Supabase platform
- Connection pooling: PostgREST handles automatically

**Data Isolation**:
- Events are organization-wide (no per-user isolation yet)
- RLS prevents anonymous access
- Only service_role and authenticated Edge Functions can query
- Clients access via Edge Functions with authentication

## Supabase Edge Functions (Phase 3)

### Function Deployment

**Location**: `supabase/functions/` directory
**Runtime**: Deno 2 with TypeScript support
**Entry**: Each function is a `index.ts` or `index.js` file
**Shared Code**: `_shared/` directory for reusable modules

**Available Functions** (Phase 3):
- `ingest/index.ts` - Event ingestion endpoint with Ed25519 signature authentication
- `query/index.ts` - Historical event querying with bearer token authentication
- `_shared/auth.ts` - Shared authentication utilities for both endpoints
- `ingest/index.test.ts` - Authentication tests for ingest endpoint
- `query/index.test.ts` - Authentication tests for query endpoint
- `_tests/rls.test.ts` - Row-level security tests

### Auth Shared Module

**Location**: `supabase/functions/_shared/auth.ts` (173 lines)

**Dependencies**:
- `@noble/ed25519@2.0.0` - Ed25519 signature verification (imported from esm.sh)
- Deno.env - Environment variable access

**Environment Variables** (from `supabase/.env.local.example`):
| Variable | Required | Purpose |
|----------|----------|---------|
| `VIBETEA_PUBLIC_KEYS` | Yes* | Monitor public keys (source_id:base64_key,source_id2:base64_key2) |
| `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | Client query endpoint authentication token |
| `SUPABASE_URL` | Yes | Supabase API base URL (from `supabase start` output) |
| `SUPABASE_SERVICE_ROLE_KEY` | Yes | Service role key for database access |

*Required unless developing with unsafe authentication mode

**Configuration File**: `supabase/.env.local` (development only)
- Copy from `supabase/.env.local.example`
- Populate with values from `supabase start` output
- Never commit to version control

**Deno Configuration** (`supabase/config.toml`):
- Deno version: 2
- Request policy: `per_worker` (hot reload enabled for development)
- Inspector port: 8083 (for debugging edge functions)

**Edge Function Features**:
- HTTP request handling via Deno.serve()
- Environment variable access via Deno.env.get()
- Cross-Origin Resource Sharing (CORS) support
- Async/await for API calls
- Request/response streaming
- Error handling with typed responses

## Event Transmission & HTTP API

### Monitor → Supabase (Phase 3)

**Endpoint**: `https://<supabase-url>/functions/v1/ingest`
**Method**: POST
**Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty string) |
| X-Signature | Yes* | Base64-encoded Ed25519 signature |
| Content-Type | Yes | application/json |

*Required unless unsafe authentication mode enabled

**Request Body**: Array of Events (max 1000) or single Event (JSON)

**Event Schema** (validated by ingest function):
| Field | Type | Pattern | Purpose |
|-------|------|---------|---------|
| `id` | string | evt_[a-z0-9]{20} | Event identifier |
| `source` | string | non-empty | Must match X-Source-ID header |
| `timestamp` | string | RFC 3339 | Event occurrence time |
| `eventType` | string | session\|activity\|tool\|agent\|summary\|error | Event type |
| `payload` | object | N/A | Event-specific data |

**Authentication Flow**:
1. Monitor loads Ed25519 private key from `~/.vibetea/key.priv`
2. Monitor signs JSON event payload using `crypto.rs` module
3. Monitor creates X-Signature header with base64-encoded signature
4. Monitor sends HTTPS POST to Supabase edge function
5. Edge function verifies signature using @noble/ed25519
6. Edge function validates request body and event schema
7. Edge function calls `bulk_insert_events()` PostgreSQL function
8. Events atomically inserted into events table with idempotency

**Response Codes**:
- 200 OK - Events successfully processed and inserted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Missing/empty X-Source-ID or signature verification failed
- 422 Unprocessable Entity - Invalid event type or source mismatch
- 500 Internal Server Error - Database insertion failed

**Response Format** (success):
```json
{
  "inserted": 45,
  "message": "Successfully processed 45 events"
}
```

**Response Format** (error):
```json
{
  "error": "invalid_event_type",
  "message": "Invalid event type 'unknown' at index 3"
}
```

### Monitor → Supabase Persistence (Phase 4)

**Module Location**: `monitor/src/persistence.rs` (1898 lines)

**Architecture Overview**:
- **EventBatcher**: Low-level buffering and HTTP transmission
- **PersistenceManager**: Async background task wrapping EventBatcher with timer-based flushing

**EventBatcher Implementation**:
- Buffered event collection with max capacity (1000 events)
- FIFO eviction when buffer full (rare condition with monitoring)
- Single HTTP POST per flush to Supabase ingest endpoint
- Exponential backoff retry with configurable limits
- Tracks consecutive failures for intelligent error handling
- Signature generation and header management

**PersistenceManager Runtime**:
- Spawned as independent tokio task: `tokio::spawn(manager.run())`
- Receives events via mpsc channel from main event loop
- Timer-based flushing on configurable interval (default 60 seconds)
- Immediate flush on buffer reaching capacity (MAX_BATCH_SIZE = 1000)
- Graceful shutdown: flushes remaining events when channel closes
- Non-blocking event queue: accepts events without waiting for transmission

**Event Flow**:
1. Main event loop queues event to persistence sender channel
2. PersistenceManager receives event via channel receiver
3. EventBatcher adds event to buffer
4. Check triggers flush if needed:
   - Buffer full: Immediate flush
   - Timer tick: Flush if buffer not empty
   - Channel closed: Flush remaining on shutdown
5. Flush sends batch to Supabase via HTTPS POST
6. Failed flushes use exponential backoff and retry
7. Max retries exceeded: Batch dropped, error logged

**Configuration**:
```rust
PersistenceConfig {
    supabase_url: "https://xyz.supabase.co/functions/v1",
    batch_interval_secs: 60,
    retry_limit: 3,
}
```

**Retry Strategy** (FR-015):
- Initial delay: 1000ms
- Backoff multiplier: 2x per retry
- Sequence: 1s, 2s, 4s, 8s, 16s...
- Limit: 3 retries by default (configurable 1-10)
- Non-retriable: 401 Unauthorized (auth failures)
- Behavior: Auth errors fail immediately, batch retained
- Max retries: Batch dropped, failure counter reset

**Error Handling**:
- `PersistenceError` enum with variants:
  - `Http` - Network/client errors
  - `AuthFailed` - 401 unauthorized (non-retriable)
  - `ServerError` - 5xx responses (retriable)
  - `MaxRetriesExceeded` - All retries exhausted
  - `Serialization` - JSON encoding failures
  - `InvalidHeader` - Header value format errors

**Testing Infrastructure**:
- Unit tests (77 tests in persistence.rs)
- Integration tests using wiremock mock server
- PersistenceManager integration tests with tokio
- Retry behavior validation with timing checks
- Graceful shutdown and final flush testing
- Full-buffer flushing verification

**Environment Configuration** (`monitor/src/config.rs`):
```rust
// Optional persistence config
pub persistence: Option<PersistenceConfig>

// Environment variables:
// VIBETEA_SUPABASE_URL - Base URL for edge function (enables persistence)
// VIBETEA_SUPABASE_BATCH_INTERVAL_SECS - Flush interval (default 60)
// VIBETEA_SUPABASE_RETRY_LIMIT - Max retries (default 3, range 1-10)
```

**Interaction with Sender Module**:
- Persistence is independent parallel path
- Does not interfere with real-time server transmission (sender module)
- Both can be active simultaneously
- Monitor sends to server for real-time + persistence buffer for Supabase

### Client → Supabase Query (Phase 3)

**Endpoint**: `https://<supabase-url>/functions/v1/query`
**Method**: GET
**Headers**:
| Header | Required | Value |
|--------|----------|-------|
| Authorization | Yes* | Bearer <token> |
| Content-Type | No | application/json |

*Required unless unsafe authentication mode enabled

**Request Parameters** (query string):
| Parameter | Required | Type | Default | Purpose |
|-----------|----------|------|---------|---------|
| `days` | No | number | 7 | Lookback period (7 or 30) |
| `source` | No | string | null | Filter by monitor source |

**Validation**:
- `days` parameter must be 7 or 30 (returns 400 if invalid)
- `source` parameter optional, no validation on value
- Bearer token required (returns 401 if missing or invalid)

**Response**: JSON object with aggregates and metadata

**Response Format** (success):
```json
{
  "aggregates": [
    {"source": "monitor-1", "date": "2026-02-03", "hour": 10, "eventCount": 45},
    {"source": "monitor-1", "date": "2026-02-03", "hour": 9, "eventCount": 32}
  ],
  "meta": {
    "totalCount": 2,
    "daysRequested": 7,
    "fetchedAt": "2026-02-03T14:30:00.000Z"
  }
}
```

**Response Format** (error):
```json
{
  "error": "invalid_days",
  "message": "days parameter must be 7 or 30"
}
```

**Response Codes**:
- 200 OK - Query successful
- 400 Bad Request - Invalid query parameters
- 401 Unauthorized - Missing or invalid bearer token
- 500 Internal Server Error - Database query failed

**Authentication Flow**:
1. Client reads token from localStorage (TokenForm component)
2. Client sends Authorization header with Bearer token
3. Edge function validates token using `verifyQueryAuth()`
4. Edge function calls `get_hourly_aggregates()` PostgreSQL function
5. Results returned to client for heatmap visualization

### Monitor → Server (Event Publishing)

**Endpoint**: `https://<server-url>/events`
**Method**: POST
**Headers**:
| Header | Required | Value |
|--------|----------|-------|
| X-Source-ID | Yes | Monitor identifier (non-empty string) |
| X-Signature | No* | Base64-encoded Ed25519 signature |
| Content-Type | Yes | application/json |

*Required unless `VIBETEA_UNSAFE_NO_AUTH=true`

**Request Body**: Single Event or array of Events (JSON)

**Flow**:
1. Monitor watches local JSONL files via file watcher
2. Parser extracts metadata from new/modified JSONL lines
3. Events processed through PrivacyPipeline
4. Monitor signs event payload with Ed25519 private key
5. Monitor POSTs signed event to server with X-Source-ID and X-Signature headers
6. Server validates signature against registered public key
7. Server rate limits based on source ID (100 events/sec default)
8. Server broadcasts to all connected clients via WebSocket

**Rate Limiting** (`server/src/rate_limit.rs`):
- Token bucket algorithm per source
- 100.0 tokens/second refill rate (configurable)
- 100 token capacity (configurable)
- Exceeded limit returns 429 Too Many Requests with Retry-After header
- Automatic cleanup of inactive sources after 60 seconds

**Client Library**: `reqwest` crate (HTTP client)
**Configuration**: `monitor/src/config.rs`
- `VIBETEA_SERVER_URL` - Server endpoint (required)
- `VIBETEA_SOURCE_ID` - Source identifier for event attribution (default: hostname)
- Uses gethostname crate to get system hostname if not provided

**Response Codes**:
- 202 Accepted - Events accepted and broadcasted
- 400 Bad Request - Invalid JSON or malformed events
- 401 Unauthorized - Missing/empty X-Source-ID or signature verification failed
- 429 Too Many Requests - Rate limit exceeded (includes Retry-After header)

### Server → Client (Event Broadcasting)

**Protocol**: WebSocket (upgraded from HTTP)
**URL**: `ws://<server-url>/ws` (or `wss://` for HTTPS)
**Authentication**: Bearer token in upgrade request headers
**Message Format**: JSON (Event)

**Flow**:
1. Client initiates WebSocket connection with Bearer token
2. Server validates token and establishes connection
3. Server broadcasts events as they arrive from monitors
4. Optional: Server filters events based on query parameters (source, type, project)
5. Client processes and stores events in Zustand state via `addEvent()`
6. Client UI renders session information from state

**Server Broadcasting** (`server/src/broadcast.rs`):
- EventBroadcaster wraps tokio broadcast channel
- 1000-event capacity for burst handling
- Thread-safe, cloneable for sharing across handlers
- SubscriberFilter enables selective delivery by event type, source, project

**WebSocket Upgrade** (`server/src/routes.rs`):
- GET /ws endpoint handles upgrade requests
- Validates bearer token before upgrade
- Returns 101 Switching Protocols on success
- Returns 401 Unauthorized on token validation failure

**Client-Side Handling** (Phase 7-10):
- WebSocket proxy configured in `client/vite.config.ts` (target: ws://localhost:8080)
- State management via `useEventStore` hook (Zustand)
- Event type guards for safe type access in `client/src/types/events.ts`
- ConnectionStatus transitions: disconnected → connecting → connected → reconnecting
- Token management via `TokenForm` component
- Connection control via `useWebSocket` hook
- Virtual scrolling display via EventStream component (Phase 8)
- Session management via SessionOverview component (Phase 10)

**Connection Details**:
- Address/port: Configured via `PORT` environment variable (default: 8080)
- Persistent connection model
- Automatic reconnection with exponential backoff (Phase 7)
- No message queuing (direct streaming)
- Events processed with selective subscriptions to prevent unnecessary re-renders

## HTTP Sender & Event Transmission

### Phase 6: Event Sender Module

**Module Location**: `monitor/src/sender.rs` (544 lines)

**Sender Features**:

1. **HTTP Client Configuration**
   - Built with `reqwest` Client
   - Connection pooling: 10 max idle connections per host
   - Request timeout: 30 seconds
   - Automatic redirect handling

2. **Event Buffering**
   - VecDeque-based buffer with FIFO eviction
   - Default capacity: 1000 events
   - Configurable via `buffer_size` parameter
   - Tracks buffer overflow events with warnings
   - Supports queuing before sending

3. **Exponential Backoff Retry**
   - Initial delay: 1 second
   - Maximum delay: 60 seconds
   - Jitter: ±25% per attempt
   - Max retry attempts: 10 per batch
   - Resets on successful send

4. **Rate Limit Handling**
   - Recognizes HTTP 429 (Too Many Requests)
   - Reads `Retry-After` header from server
   - Respects server-provided delay
   - Falls back to exponential backoff if no header

5. **Event Signing**
   - Signs JSON event payload with Ed25519
   - X-Signature header contains base64-encoded signature
   - X-Source-ID header contains monitor source identifier
   - Compatible with server `auth.rs` verification

6. **Batch Sending**
   - `send_batch()` for efficient transmission
   - Single HTTP request with event array or single event
   - JSON request body with event(s)
   - 202 Accepted response expected

7. **Buffer Management**
   - `queue(event)` - Add to buffer
   - `flush()` - Send all buffered events
   - `send(event)` - Send single event immediately
   - `buffer_len()` - Current buffer size
   - `is_empty()` - Check if buffer empty

8. **Graceful Shutdown**
   - `shutdown(timeout)` - Flushes remaining events
   - Returns count of unflushed events
   - Waits for timeout before giving up
   - Allows time for final retry attempts

**SenderConfig**:
```rust
pub struct SenderConfig {
    pub server_url: String,     // e.g., https://vibetea.fly.dev
    pub source_id: String,      // e.g., hostname
    pub buffer_size: usize,     // e.g., 1000
}
```

**SenderError Types**:
- `Http` - HTTP client error (network, TLS, etc.)
- `ServerError { status, message }` - Non-202 response
- `AuthFailed` - 401 Unauthorized (invalid signature)
- `RateLimited { retry_after_secs }` - 429 with delay
- `BufferOverflow { evicted_count }` - Events evicted
- `MaxRetriesExceeded { attempts }` - All retries failed
- `Json` - Event serialization error

**Connection Details**:
- Server URL from `VIBETEA_SERVER_URL` env var
- POST to `{server_url}/events` endpoint
- HTTPS recommended for production
- HTTP allowed for local development

## CLI & Key Management

### Phase 6: Monitor CLI

**Module Location**: `monitor/src/main.rs` (301 lines)

**Command Structure**:

1. **init Command**: Generate Ed25519 keypair
   ```bash
   vibetea-monitor init [--force]
   ```
   - Generates new keypair using `Crypto::generate()`
   - Saves to `~/.vibetea/` or `VIBETEA_KEY_PATH`
   - Displays public key for server registration
   - Prompts for overwrite confirmation (unless --force)
   - Provides copy-paste ready export command

2. **run Command**: Start monitor daemon
   ```bash
   vibetea-monitor run
   ```
   - Loads configuration from environment variables
   - Loads cryptographic keys from disk
   - Creates sender with buffering and retry
   - Initializes file watcher (future: Phase 7)
   - Waits for shutdown signal
   - Graceful shutdown with event flushing

3. **help Command**: Show documentation
   ```bash
   vibetea-monitor help
   vibetea-monitor --help
   vibetea-monitor -h
   ```
   - Displays usage information
   - Lists all available commands
   - Shows environment variables
   - Provides example commands

4. **version Command**: Show version
   ```bash
   vibetea-monitor version
   vibetea-monitor --version
   vibetea-monitor -V
   ```
   - Prints binary version from Cargo.toml

**CLI Features**:
- Manual argument parsing (no external CLI framework)
- Flag support: `--force`, `-f` for init overwrite
- Short and long option variants for help/version
- User prompts on stdout/stderr
- Structured error messages
- Exit codes: 0 on success, 1 on error

**Environment Variables Used**:

| Variable | Required | Default | Command |
|----------|----------|---------|---------|
| `VIBETEA_SERVER_URL` | Yes | - | run |
| `VIBETEA_SOURCE_ID` | No | hostname | run |
| `VIBETEA_KEY_PATH` | No | ~/.vibetea | init, run |
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | run |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | run |
| `VIBETEA_BASENAME_ALLOWLIST` | No | - | run |
| `VIBETEA_SUPABASE_URL` | No | - | run |
| `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | No | 60 | run |
| `VIBETEA_SUPABASE_RETRY_LIMIT` | No | 3 | run |
| `RUST_LOG` | No | info | all |

**Logging**:
- Structured logging via `tracing` crate
- Environment-based filtering (`RUST_LOG`)
- JSON output support
- Logs configuration, key loading, shutdown events
- Info level by default

**Signal Handling**:
- Listens for SIGINT (Ctrl+C)
- Listens for SIGTERM on Unix
- Cross-platform support via `tokio::signal`
- Graceful shutdown sequence on signal

**Key Registration Workflow**:
1. User runs: `vibetea-monitor init`
2. Binary displays public key
3. User copies to: `export VIBETEA_PUBLIC_KEYS="...:<public_key>"`
4. User adds to server configuration
5. User runs: `vibetea-monitor run`

## Development & Local Configuration

### Local Supabase Setup (Phase 3)

**Prerequisites**:
- Docker and Docker Compose installed
- Supabase CLI installed: `brew install supabase/tap/supabase`

**Initialization**:
```bash
supabase init                    # Creates supabase/ directory
supabase start                   # Starts local development environment
```

**Output from `supabase start`**:
```
Supabase local development environment started
API URL: http://127.0.0.1:54321
Graphql URL: http://127.0.0.1:54321/graphql/v1
DB URL: postgresql://postgres:postgres@127.0.0.1:54322/postgres
Studio URL: http://127.0.0.1:54323
Inbucket URL: http://127.0.0.1:54324
```

**Configuration File**: `supabase/config.toml`
- PostgreSQL 17 on port 54322
- PostgREST API on port 54321
- Studio web UI on port 54323
- Deno runtime version 2
- Email testing server on port 54324

**Environment Setup**: `supabase/.env.local`
- Copy `supabase/.env.local.example`
- Populate from `supabase start` output:
  ```
  SUPABASE_URL=http://127.0.0.1:54321
  SUPABASE_SERVICE_ROLE_KEY=<service-role-key-from-output>
  VIBETEA_SUBSCRIBER_TOKEN=dev-token-for-testing
  VIBETEA_PUBLIC_KEYS=dev-monitor:<public-key-from-monitor-init>
  ```

**Database Migrations**:
```bash
supabase db push              # Applies pending migrations
supabase migration list       # View applied migrations
```

**Edge Functions Deployment** (Phase 3):
```bash
supabase functions deploy     # Deploy all functions to remote
supabase functions serve      # Test functions locally
```

**Testing Edge Functions** (Phase 3):
```bash
deno test --allow-env --allow-net supabase/functions/ingest/index.test.ts
deno test --allow-env --allow-net supabase/functions/query/index.test.ts
deno test --allow-env --allow-net supabase/functions/_tests/rls.test.ts
```

**Studio Access**:
- Open http://127.0.0.1:54323 in browser
- View/edit database schema
- Test API endpoints
- Manage Edge Functions

## Error Handling & Validation

### Supabase Edge Function Error Handling (Phase 3)

**Validation Points**:
1. Authentication validation (Ed25519 signature or bearer token)
2. Request body validation (JSON parsing)
3. Event schema validation (ingest endpoint)
4. Query parameter validation (query endpoint)
5. Database RPC execution

**Error Response Format**:
```json
{
  "error": "error_code",
  "message": "Human-readable error description"
}
```

**Error Codes** (ingest endpoint):
- `missing_auth` - Missing X-Source-ID or X-Signature header
- `unknown_source` - X-Source-ID not registered in VIBETEA_PUBLIC_KEYS
- `invalid_signature` - Signature verification failed
- `invalid_request` - Request body not valid JSON or not array
- `empty_batch` - Request body is empty array
- `batch_too_large` - More than 1000 events in batch
- `invalid_event` - Event missing required field or invalid format
- `invalid_event_type` - eventType not in allowed list
- `source_mismatch` - event.source doesn't match X-Source-ID
- `internal_error` - Database insertion failed

**Error Codes** (query endpoint):
- `missing_auth` - Authorization header missing
- `invalid_token` - Bearer token invalid
- `invalid_days` - days parameter not 7 or 30
- `internal_error` - Database query failed

**Response Codes**:
- 200 OK - Successful response
- 400 Bad Request - Validation error (invalid JSON, empty batch, etc.)
- 401 Unauthorized - Authentication failed
- 422 Unprocessable Entity - Invalid event type or source mismatch
- 500 Internal Server Error - Database operation failed

## Future Integration Points

### Planned (Not Yet Integrated)

- **Client Supabase Integration**: Query historical event aggregates via `get_hourly_aggregates()` function
- **Database Retention Policies**: Automated event archival and purging
- **Backup & Disaster Recovery**: Automated backups and point-in-time recovery
- **Background Job Scheduling**: Data aggregation jobs
- **Event Search/Filtering**: Full-text search and advanced filtering UI
- **Analytics Dashboard**: Event trends, metrics, insights
- **Per-user RLS Policies**: Multi-tenant event isolation
- **Token Management**: Rotation, expiration, refresh mechanisms

## Configuration Quick Reference

### Supabase Environment Variables (Phase 3+)

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `SUPABASE_URL` | string | - | Yes | Supabase project URL (from `supabase start`) |
| `SUPABASE_SERVICE_ROLE_KEY` | string | - | Yes | Service role key for database access |
| `VIBETEA_PUBLIC_KEYS` | string | - | Yes* | Monitor public keys (source:key format) |
| `VIBETEA_SUBSCRIBER_TOKEN` | string | - | Yes* | Client query endpoint token |

*Used by Supabase Edge Functions for authentication

### Monitor Persistence Configuration (Phase 4)

| Variable | Type | Default | Required | Purpose |
|----------|------|---------|----------|---------|
| `VIBETEA_SUPABASE_URL` | string | - | No | Supabase edge function base URL |
| `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | integer | 60 | No | Seconds between batch submissions (min 1) |
| `VIBETEA_SUPABASE_RETRY_LIMIT` | integer | 3 | No | Max retry attempts (1-10 range) |

