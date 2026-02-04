# Feature Specification: Supabase Authentication

**Feature Branch**: `002-supabase-auth`
**Created**: 2026-02-04
**Status**: Draft
**Input**: User description: "Supabase Authentication - Replace shared bearer token with GitHub OAuth via Supabase for clients, move monitor public keys from env var to Supabase table with periodic refresh"

## Overview

Replace the current shared bearer token authentication for clients with GitHub OAuth via Supabase. Additionally, move monitor public key management from environment variables to a Supabase database table with periodic server-side refresh.

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Client GitHub Authentication (Priority: P1)

As a client user, I want to authenticate using my GitHub account so that I have a secure, individual identity instead of sharing a single token with all users.

**Why this priority**: This is the core authentication change. Without GitHub OAuth working, the entire feature has no value. It replaces the insecure shared token model with proper user authentication.

**Independent Test**: Can be fully tested by logging in with GitHub, receiving a session token, and connecting to the WebSocket. Delivers secure individual user authentication.

**Acceptance Scenarios**:

1. **Given** I am not authenticated, **When** I visit the dashboard, **Then** I am redirected to the login page
2. **Given** I am on the login page, **When** I click "Sign in with GitHub", **Then** I am redirected to GitHub's OAuth flow
3. **Given** I complete GitHub OAuth, **When** I am redirected back, **Then** I am authenticated and redirected to the dashboard
4. **Given** I am authenticated, **When** I visit the dashboard, **Then** I see the real-time event stream

---

### User Story 2 - Server Session Token Exchange (Priority: P1)

As an authenticated client, I need the server to validate my Supabase JWT and provide a short-lived session token so that I can connect to the WebSocket securely.

**Why this priority**: This is the bridge between Supabase authentication and WebSocket access. Without it, authenticated users cannot access real-time events.

**Independent Test**: Can be tested by calling the session endpoint with a valid Supabase JWT and verifying a session token is returned that works for WebSocket connection.

**Acceptance Scenarios**:

1. **Given** I have a valid Supabase JWT, **When** I request a session token from the server, **Then** I receive a session token with a 5-minute TTL
2. **Given** I have an invalid or expired Supabase JWT, **When** I request a session token, **Then** I receive a 401 Unauthorized response
3. **Given** I have a valid session token, **When** I connect to the WebSocket, **Then** the connection succeeds and the token TTL is extended
4. **Given** I have an expired session token, **When** I connect to the WebSocket, **Then** I receive a 401 and must re-authenticate

---

### User Story 3 - Client Reconnection with Token Refresh (Priority: P2)

As a connected client, when my WebSocket connection drops, I want the application to automatically reconnect and refresh tokens as needed so that I don't have to manually re-authenticate.

**Why this priority**: Essential for user experience but depends on P1 stories being complete. Users shouldn't need to manually handle disconnections.

**Independent Test**: Can be tested by simulating a WebSocket disconnect and verifying automatic reconnection with existing token, then simulating server restart and verifying automatic token refresh.

**Acceptance Scenarios**:

1. **Given** my WebSocket disconnects, **When** my session token is still valid, **Then** the client automatically reconnects using the existing token
2. **Given** my WebSocket disconnects, **When** my session token has expired, **Then** the client automatically requests a new session token and reconnects
3. **Given** my WebSocket disconnects, **When** my Supabase session has expired, **Then** I am redirected to the login page

---

### User Story 4 - Monitor Public Key Management via Supabase (Priority: P2)

As an administrator, I want to manage monitor public keys in Supabase instead of environment variables so that I can add or remove monitors without restarting the server.

**Why this priority**: Independent of client auth but improves operational flexibility. Can be implemented and tested separately.

**Independent Test**: Can be tested by adding a public key to the Supabase table and verifying the server accepts events signed by that key within 30 seconds.

**Acceptance Scenarios**:

1. **Given** I add a public key to the Supabase table, **When** the server refreshes (within 30 seconds), **Then** the monitor with that key can submit events
2. **Given** I remove a public key from the Supabase table, **When** the server refreshes, **Then** the monitor with that key is rejected
3. **Given** the Supabase endpoint is temporarily unavailable, **When** the server tries to refresh, **Then** the server keeps existing keys and retries later

---

### User Story 5 - Server Startup Resilience (Priority: P3)

As a server operator, I want the server to handle transient network errors on startup so that brief Supabase connectivity issues don't prevent deployment.

**Why this priority**: Operational resilience improvement. The core functionality works without this, but it improves deployment reliability.

**Independent Test**: Can be tested by starting the server with Supabase temporarily unavailable and verifying retry behavior.

**Acceptance Scenarios**:

1. **Given** Supabase is available on startup, **When** the server starts, **Then** public keys are loaded successfully
2. **Given** Supabase is temporarily unavailable, **When** the server starts, **Then** the server retries up to 5 times with exponential backoff
3. **Given** all 5 retry attempts fail, **When** the server exhausts retries, **Then** the server exits with a clear error message

---

### Edge Cases

- What happens when a user's Supabase session is revoked while they're connected via WebSocket? (Connection stays open until next reconnect)
- What happens when the server restarts while clients are connected? (Clients reconnect, get 401, refresh token automatically)
- What happens when Supabase is down during a token validation request? (Client receives 502/503, retries with backoff)
- What happens when the public keys endpoint returns malformed JSON? (Server logs warning, keeps existing keys)

## Requirements *(mandatory)*

### Functional Requirements

#### Client Authentication
- **FR-001**: System MUST redirect unauthenticated users to a dedicated login page
- **FR-002**: System MUST provide GitHub OAuth authentication via Supabase
- **FR-003**: System MUST allow any authenticated GitHub user to access the dashboard (no allowlist)
- **FR-004**: System MUST store Supabase session using the Supabase client SDK

#### Session Token Exchange
- **FR-005**: Server MUST expose an endpoint to exchange Supabase JWTs for short-lived session tokens
- **FR-006**: Server MUST validate Supabase JWTs by calling Supabase's `GET {SUPABASE_URL}/auth/v1/user` endpoint with a 5-second timeout; return 503 if timeout or unreachable
- **FR-007**: Server MUST generate opaque, cryptographically random session tokens
- **FR-008**: Server MUST store session tokens in memory with a 5-minute TTL
- **FR-009**: Server MUST extend session token TTL by 5 minutes once upon successful WebSocket connection establishment (not on every message)

#### WebSocket Authentication
- **FR-010**: Server MUST accept session tokens as query parameters on WebSocket connections
- **FR-011**: Server MUST reject WebSocket connections with invalid or expired session tokens
- **FR-012**: Client MUST automatically refresh session tokens on 401 responses during reconnection using exponential backoff (1s, 2s, 4s); after 3 failed attempts, redirect to login page

#### Monitor Public Key Management
- **FR-013**: Server MUST fetch monitor public keys from Supabase edge function `GET {SUPABASE_URL}/functions/v1/public-keys` on startup
- **FR-014**: Server MUST refresh monitor public keys every 30 seconds; new keys become active within 60 seconds of being added to Supabase (one refresh cycle)
- **FR-015**: Server MUST NOT require authentication for the public keys endpoint
- **FR-016**: Edge function MUST return JSON: `{"keys": [{"source_id": "string", "public_key": "string"}]}`; public keys are base64-encoded Ed25519 keys
- **FR-017**: Server MUST keep existing public keys if a refresh fails and retry on next interval

#### Server Resilience
- **FR-018**: Server MUST retry Supabase connection on startup up to 5 times
- **FR-019**: Server MUST use exponential backoff with jitter: `delay = min(2^attempt * 100ms + random(0, 100ms), 10s)` for attempts 1-5
- **FR-020**: Server MUST exit with an error if all startup retries fail

#### Session Token Details
- **FR-021**: Session tokens MUST be 32 bytes of cryptographically secure random data, base64-url encoded without padding (resulting in 43 characters)
- **FR-022**: Session token store MUST have a maximum capacity of 10,000 tokens; when capacity is reached, return 503 Service Unavailable with message "Session capacity exceeded"
- **FR-023**: Server MUST periodically clean up expired session tokens (lazy cleanup on access + sweep every 60 seconds)
- **FR-024**: Session token expiry validation MUST include a 30-second grace period for clock skew; grace period applies ONLY to WebSocket connection validation, NOT to token exchange endpoint

#### Token Exchange API
- **FR-025**: Token exchange endpoint MUST be `POST /auth/session`
- **FR-026**: Token exchange request MUST include Supabase JWT in `Authorization: Bearer <jwt>` header
- **FR-027**: Token exchange response MUST include session_token and expires_in fields
- **FR-028**: Server MUST return 401 for invalid/expired JWTs, 503 if Supabase is unreachable

#### Error Handling
- **FR-029**: Invalid session token on WebSocket MUST return 401 Unauthorized
- **FR-030**: Supabase API unavailable during token exchange MUST return 503 Service Unavailable
- **FR-031**: All auth error responses MUST follow existing ErrorResponse format

### Key Entities

- **Supabase User Session**: Represents an authenticated GitHub user; managed by Supabase SDK; contains access_token (JWT), refresh_token, user metadata
- **Server Session Token**: Short-lived opaque token; maps to a validated Supabase user; 5-minute TTL extended on WebSocket connection
- **Allowed Public Key**: Monitor identifier and Ed25519 public key; stored in Supabase table; fetched periodically by server

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete GitHub authentication and reach the dashboard in under 30 seconds
- **SC-002**: WebSocket connection establishes within 2 seconds of session token acquisition
- **SC-003**: Automatic reconnection after disconnect completes within 5 seconds (with valid tokens)
- **SC-004**: New monitor public keys become active within 60 seconds of being added to Supabase (server refreshes every 30s)
- **SC-005**: Server startup succeeds within 30 seconds when Supabase endpoints respond within 200ms
- **SC-006**: Server handles 100 concurrent authenticated WebSocket connections maintained for 60 seconds while accepting/delivering messages at 10 msg/sec without dropped connections
- **SC-007**: Session token exchange endpoint responds within 1 second (including Supabase validation)

## Assumptions

- Supabase project is already created with GitHub OAuth provider configured
- Server has network access to Supabase endpoints
- Client is served from a domain that can redirect to/from Supabase OAuth flow
- Existing monitors continue using Ed25519 signature authentication (unchanged)
- No user-level authorization/allowlist required initially (any GitHub user can access)

## Development Standards

### Pre-commit Hooks
- **DS-001**: Project MUST use lefthook for git hooks management
- **DS-002**: Pre-commit hooks MUST run formatting checks (prettier for TypeScript, cargo fmt for Rust)
- **DS-003**: Pre-commit hooks MUST run linting (ESLint for TypeScript, cargo clippy for Rust)

### CI/CD Requirements
- **DS-004**: GitHub Actions workflow MUST include tests for new auth endpoints
- **DS-005**: CI MUST validate new environment variables are documented
- **DS-006**: CI MUST test both authenticated and unauthenticated request paths

### Environment Configuration
- **DS-007**: New environment variables MUST be documented in README and .env.example
- **DS-008**: Server MUST validate required Supabase environment variables on startup
- **DS-009**: Client MUST validate required Supabase environment variables at build time
- **DS-010**: Missing required environment variables MUST produce clear error messages

## Known Limitations

- **Single-server deployment only**: Session tokens are stored in memory, not suitable for horizontal scaling
- **Server restart invalidates sessions**: All clients must re-authenticate after server restart
- **Token in URL**: Session token passed as query parameter may appear in server access logs

## Out of Scope

- User allowlist/blocklist functionality
- Multiple OAuth providers (only GitHub for now)
- User roles or permissions
- Audit logging of authentication events
- Rate limiting on authentication endpoints
- Session token persistence across server restarts
- Distributed session storage (Redis)
- GitHub organization membership validation
