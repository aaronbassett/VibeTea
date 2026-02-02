# Technology Stack

**Status**: Phase 3 Implementation - Authentication, broadcasting, and rate limiting modules complete
**Last Updated**: 2026-02-02

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, event capture and signing |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization |

## Frameworks & Runtime Libraries

### Rust (Monitor & Server)

| Package            | Version | Purpose | Used By |
|--------------------|---------|---------|----------|
| tokio              | 1.43    | Async runtime with full features | Server, Monitor |
| axum               | 0.8     | HTTP/WebSocket server framework | Server |
| tower              | 0.5     | Composable middleware | Server |
| tower-http         | 0.6     | HTTP utilities (CORS, tracing) | Server |
| reqwest            | 0.12    | HTTP client library | Monitor, Server (tests) |
| serde              | 1.0     | Serialization/deserialization | All |
| serde_json         | 1.0     | JSON serialization | All |
| ed25519-dalek      | 2.1     | Ed25519 cryptographic signing | Server, Monitor |
| uuid               | 1.11    | Unique identifiers for events | Server, Monitor |
| chrono             | 0.4     | Timestamp handling | Server, Monitor |
| thiserror          | 2.0     | Error type derivation | Server, Monitor |
| anyhow             | 1.0     | Flexible error handling | Server, Monitor |
| tracing            | 0.1     | Structured logging framework | Server, Monitor |
| tracing-subscriber | 0.3     | Logging implementation (JSON, env-filter) | Server, Monitor |
| notify             | 8.0     | File system watching | Monitor |
| base64             | 0.22    | Base64 encoding/decoding | Server, Monitor |
| rand               | 0.9     | Random number generation | Server, Monitor |
| directories        | 6.0     | Standard directory paths | Monitor |
| gethostname        | 1.0     | System hostname retrieval | Monitor |
| subtle             | 2.6     | Constant-time comparison for cryptography | Server (auth) |
| futures-util       | 0.3     | WebSocket stream utilities | Server |

### TypeScript/JavaScript (Client)

| Package                    | Version  | Purpose |
|---------------------------|----------|---------|
| React                      | ^19.2.4  | UI framework |
| React DOM                  | ^19.2.4  | DOM rendering |
| TypeScript                 | ^5.9.3   | Language and type checking |
| Vite                       | ^7.3.1   | Build tool and dev server |
| Tailwind CSS               | ^4.1.18  | Utility-first CSS framework |
| Zustand                    | ^5.0.11  | Lightweight state management |
| @tanstack/react-virtual    | ^3.13.18 | Virtual scrolling for large lists |
| @vitejs/plugin-react       | ^5.1.3   | React Fast Refresh for Vite |
| @tailwindcss/vite          | ^4.1.18  | Tailwind CSS Vite plugin |
| vite-plugin-compression2   | ^2.4.0   | Brotli compression for builds |

## Build Tools & Package Managers

| Tool     | Version  | Purpose |
|----------|----------|---------|
| cargo    | -        | Rust package manager and build system |
| pnpm     | -        | Node.js package manager (client) |
| rustfmt  | -        | Rust code formatter |
| clippy   | -        | Rust linter |
| prettier | ^3.8.1   | Code formatter (TypeScript) |
| ESLint   | ^9.39.2  | JavaScript/TypeScript linter |

## Development & Testing

### Rust Testing
| Package      | Version | Purpose |
|--------------|---------|---------|
| tokio-test   | 0.4     | Tokio testing utilities |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit/component testing framework |
| @testing-library/react | ^16.3.2  | React testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM matchers for testing |
| jsdom                  | ^28.0.0  | DOM implementation for Node.js |

## Configuration Files

| File | Framework | Purpose |
|------|-----------|---------|
| `client/vite.config.ts` | Vite | Build configuration, WebSocket proxy to server on port 8080 |
| `client/tsconfig.json` | TypeScript | Strict mode, ES2020 target |
| `Cargo.toml` (workspace) | Cargo | Rust workspace configuration and shared dependencies |
| `server/Cargo.toml` | Cargo | Server package configuration |
| `monitor/Cargo.toml` | Cargo | Monitor package configuration |

## Runtime Environment

| Aspect | Details |
|--------|---------|
| Server Runtime | Rust binary (tokio async) |
| Client Runtime | Browser (ES2020+) |
| Monitor Runtime | Native binary (Linux/macOS/Windows) |
| Node.js | Required for development and client build only |
| Async Model | Tokio (Rust), Promises (TypeScript) |
| WebSocket Support | Native (server-side via axum, client-side via browser) |
| WebSocket Proxy | Vite dev server proxies /ws to localhost:8080 |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature |
| Server → Client | WebSocket | JSON | Bearer token |
| Client → Server | WebSocket | JSON | Bearer token |

## Data Serialization

| Component | Serialization | Notes |
|-----------|---------------|-------|
| Server/Monitor | serde (Rust) | JSON with snake_case for env configs |
| Client | TypeScript/JSON | camelCase for API contracts |
| Events | serde_json | Standardized event schema across components |

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io |
| Monitor | Binary | ELF/Mach-O/PE | Standalone executable for users |
| Client | Static files | JS + CSS (Brotli compressed) | CDN (Netlify/Vercel/Cloudflare) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
- `hooks/useEventStore.ts` - Zustand store for WebSocket event state with selective subscriptions
- `types/events.ts` - Event type definitions with discriminated union types matching Rust schema
- `utils/` - Utility functions
- `App.tsx` - Root component
- `main.tsx` - Entry point
- `index.css` - Global styles

### Server (`server/src`)
- `config.rs` - Environment variable parsing and validation (public keys, subscriber token, port)
- `auth.rs` - Ed25519 signature verification and token validation with constant-time comparison
- `broadcast.rs` - Event broadcaster using tokio broadcast channels with subscriber filtering
- `rate_limit.rs` - Per-source token bucket rate limiting (100 events/sec default)
- `routes.rs` - HTTP endpoints (POST /events, GET /ws, GET /health)
- `error.rs` - Error types and handling
- `types.rs` - Event types and data models
- `lib.rs` - Public library interface
- `main.rs` - Server entry point

### Monitor (`monitor/src`)
- `config.rs` - Configuration from environment variables (server URL, source ID, key path, buffer size)
- `error.rs` - Error types
- `types.rs` - Event types
- `lib.rs` - Public interface
- `main.rs` - Monitor entry point

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary, minimal base image |
| Client | CDN | Static files | Optimized builds with compression |
| Monitor | Local | Native binary | Users download and run locally |

## Phase 3 Additions

**Server Authentication Module** (`server/src/auth.rs`):
- Ed25519 signature verification with `subtle::ConstantTimeEq` for timing-resistant comparison
- Token validation for WebSocket clients
- Comprehensive AuthError enum with detailed error types
- Support for unknown sources, invalid signatures, base64 encoding failures, malformed keys
- Unsafe mode support for development (skips signature verification when enabled)

**Server Broadcasting Module** (`server/src/broadcast.rs`):
- EventBroadcaster wrapping tokio broadcast channels
- SubscriberFilter for filtering events by type, source, and project
- 1000-event channel capacity for handling burst traffic
- Thread-safe, cloneable broadcaster for sharing across async tasks

**Server Rate Limiting Module** (`server/src/rate_limit.rs`):
- Per-source token bucket rate limiting
- 100 events/second default rate with 100-token burst capacity
- Per-source configuration with automatic stale entry cleanup
- Returns Retry-After value for rate-limited clients

**Server Routes Module** (`server/src/routes.rs`):
- AppState struct containing config, broadcaster, rate limiter, and uptime tracking
- POST /events endpoint with signature verification and event broadcasting
- GET /ws endpoint with WebSocket upgrade and token validation
- GET /health endpoint with uptime reporting
- Rate limiting applied per source via X-Source-ID header
- Comprehensive error handling with HTTP status codes (401, 429, 400, 202, 101)

**Integration Tests** (`server/tests/unsafe_mode_test.rs`):
- 18 comprehensive integration tests for unsafe mode functionality
- Tests for POST /events without signature, with invalid/malformed signatures
- Tests for GET /ws upgrade without token and with invalid tokens
- Tests for event broadcasting and ordering
- Tests for rate limiting persistence in unsafe mode
- Tests for source ID requirement and event validation

**New Dependency**: `subtle` 2.6 for constant-time comparison in signature verification
**New Dependency**: `futures-util` 0.3 for WebSocket stream handling

## Not Yet Implemented

- Database/persistence layer
- Advanced state management patterns (beyond Context + Zustand)
- Session persistence beyond memory
- Request/response logging to external services
- Enhanced error tracking
- Automatic reconnection on WebSocket disconnection
- Per-user authentication tokens
- Token rotation and expiration
