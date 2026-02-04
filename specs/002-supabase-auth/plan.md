<!--
==============================================================================
PLAN TEMPLATE
==============================================================================

PURPOSE:
  Defines technical implementation plans with architecture decisions, file
  structure, and constitution compliance. Bridges specification (WHAT) to
  tasks (HOW).

WHEN USED:
  - By /sdd:plan command when creating implementation plans
  - After spec is created and approved
  - Sets technical context for the entire feature

CUSTOMIZATION:
  - Add project-specific technical context fields
  - Customize complexity tracking for your constitution
  - Add architecture decision sections relevant to your domain
  - Override by creating .sdd/templates/plan-template.md in your repo

LEARN MORE:
  See plugins/sdd/skills/sdd-infrastructure/references/template-guide.md
  for detailed documentation and examples.

==============================================================================
-->

# Implementation Plan: Supabase Authentication

**Branch**: `002-supabase-auth` | **Date**: 2026-02-04 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-supabase-auth/spec.md`

## Summary

Replace the current shared bearer token authentication for clients with GitHub OAuth via Supabase. The server will validate Supabase JWTs and issue short-lived session tokens for WebSocket access. Additionally, monitor public key management will move from environment variables to a Supabase table with periodic server-side refresh.

**Technical Approach**:
- Client: Add Supabase JS SDK for GitHub OAuth flow
- Server: New `/auth/session` endpoint to exchange Supabase JWTs for session tokens
- Server: Validate JWTs by calling Supabase `/auth/v1/user` endpoint
- Server: In-memory session token store with 5-minute TTL
- Server: Periodic public key refresh from Supabase edge function
- Client: Automatic token refresh on 401 during WebSocket reconnection

## Technical Context

**Language/Version**:
- Server: Rust 2021 edition (existing)
- Client: TypeScript 5.x with React 19 (existing)

**Primary Dependencies**:
- Server (existing): axum 0.8, tokio 1.43, ed25519-dalek 2.1, reqwest 0.12
- Server (new): `jsonwebtoken` for JWT parsing (optional - can validate via Supabase API)
- Client (existing): React 19, Zustand 5.0, Vite 7.3
- Client (new): `@supabase/supabase-js` for auth SDK

**Storage**:
- Server: In-memory session token store (HashMap with TTL cleanup)
- Server: Supabase PostgreSQL for monitor public keys
- Client: Supabase SDK handles session storage (localStorage)

**Testing**:
- Server: cargo test with `--test-threads=1` for env var tests
- Client: Vitest with React Testing Library

**Target Platform**:
- Server: Linux (Docker on Fly.io)
- Client: Browser (ES2020+)

**Project Type**: Web application (frontend + backend)

**Performance Goals**:
- Session token exchange: <1 second (including Supabase validation)
- WebSocket connection: <2 seconds after token acquisition
- Public key refresh: 30-second intervals
- Support 100 concurrent authenticated WebSocket connections

**Constraints**:
- Session tokens: 5-minute TTL (extended on WebSocket connection)
- In-memory storage: Single-server deployment only
- Server restart invalidates all sessions (acceptable per spec)
- 30-second grace period for clock skew on token validation

**Scale/Scope**:
- Any authenticated GitHub user can access (no allowlist)
- Single-server deployment initially
- Max session token capacity configurable to prevent memory exhaustion

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Privacy by Design (I) ✅ PASS
- No logging of user credentials, JWTs, or session tokens in plaintext
- Supabase handles GitHub OAuth securely
- Session tokens are opaque random bytes, not user data
- No tracking of user identity beyond authentication
- **Action**: Ensure session token generation/validation doesn't log tokens

### Unix Philosophy (II) ✅ PASS
- Server remains the event hub - auth is a new concern, not a new component
- Client handles OAuth flow separately from event display
- Supabase edge function handles public key serving (separate from server core)
- Token exchange is a well-defined interface

### Keep It Simple (III) ✅ PASS with monitoring
- **Simplest approach**: Validate JWT via Supabase API (no local JWT parsing)
- **Risk**: Adding Supabase dependency increases complexity
- **Mitigation**: Using Supabase JS SDK (well-supported), edge function is simple
- **Alternative rejected**: Custom OAuth implementation far more complex

### Event-Driven Communication (IV) ✅ PASS
- WebSocket connections remain event-driven
- Auth is a prerequisite, not a change to event flow
- Session tokens extend existing bearer token pattern

### Test What Matters (V) ✅ PASS with test matrix

**Required Test Coverage Matrix**:

| Test Category | Test Cases | Location |
|---------------|------------|----------|
| JWT Validation | Valid JWT → 200, Invalid JWT → 401, Supabase down → 503 | server/tests/auth_test.rs |
| Session Token Generation | 100 tokens all unique, 32-byte entropy validation | server/tests/session_test.rs |
| Token TTL & Grace Period | Token at T+5min-29s passes, T+5min+31s fails | server/tests/session_test.rs |
| Public Key Refresh | Startup retry (5 attempts), periodic refresh success/failure | server/tests/supabase_test.rs |
| Privacy Compliance | No JWTs or session tokens in any log output (RUST_LOG=trace) | server/tests/auth_privacy_test.rs |
| Client Token Refresh | 401 triggers refresh, 3 failures redirects to login | client/tests/auth.test.ts |

**Privacy tests**: Verify no tokens logged (Constitution I compliance)

### Fail Fast & Loud (VI) ✅ PASS
- Invalid JWT → 401 with clear message
- Expired token → 401 with clear message
- Supabase unavailable → 503 with retry guidance
- Clear error messages defined in spec (FR-028-031)

### Modularity & Clear Boundaries (VII) ✅ PASS
- Auth module separate from routes
- Session store is a new independent module
- Public key management separate from signature verification

### Rust Standards ✅ PASS
- Continue using clippy, rustfmt
- Result types for all fallible operations
- thiserror for error types

### TypeScript Standards ✅ PASS
- Strict TypeScript
- Supabase SDK types
- ESLint, Prettier

### No Violations Requiring Justification

## Project Structure

### Documentation (this feature)

```text
specs/002-supabase-auth/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (OpenAPI specs)
└── tasks.md             # Phase 2 output (/sdd:tasks command)
```

### Source Code (repository root)

```text
server/
├── src/
│   ├── main.rs           # Server entry point (add session cleanup task)
│   ├── lib.rs            # Public library interface
│   ├── routes.rs         # HTTP routes (add POST /auth/session, modify GET /ws)
│   ├── auth.rs           # Auth module (add JWT validation, session token generation)
│   ├── session.rs        # NEW: Session token store with TTL
│   ├── supabase.rs       # NEW: Supabase API client (public key fetch)
│   ├── config.rs         # Config (add Supabase URL, JWT secret)
│   ├── broadcast.rs      # Event broadcaster (unchanged)
│   ├── rate_limit.rs     # Rate limiting (unchanged)
│   ├── error.rs          # Error types (add auth errors)
│   └── types.rs          # Shared types (unchanged)
└── tests/
    ├── session_test.rs   # NEW: Session token tests
    └── auth_test.rs      # Extend auth tests

client/
├── src/
│   ├── main.tsx          # Entry point (unchanged)
│   ├── App.tsx           # Root component (add auth routing)
│   ├── pages/            # NEW: Page components
│   │   ├── Login.tsx     # NEW: Login page with GitHub OAuth
│   │   └── Dashboard.tsx # NEW: Main dashboard (moved from App.tsx)
│   ├── components/
│   │   ├── ConnectionStatus.tsx  # Existing
│   │   ├── EventStream.tsx       # Existing
│   │   ├── Heatmap.tsx           # Existing
│   │   ├── SessionOverview.tsx   # Existing
│   │   └── TokenForm.tsx         # MODIFY: Remove or repurpose
│   ├── hooks/
│   │   ├── useEventStore.ts      # Existing
│   │   ├── useWebSocket.ts       # MODIFY: Use session token
│   │   ├── useSessionTimeouts.ts # Existing
│   │   └── useAuth.ts            # NEW: Supabase auth hook
│   ├── services/
│   │   └── supabase.ts           # NEW: Supabase client setup
│   ├── types/
│   │   └── events.ts             # Existing
│   └── utils/
│       └── formatting.ts         # Existing
└── tests/
    └── auth.test.ts              # NEW: Auth flow tests

supabase/
├── functions/
│   └── public-keys/             # NEW: Edge function for monitor public keys
│       └── index.ts
└── migrations/
    └── 001_public_keys.sql      # NEW: Create monitor_public_keys table
```

**Structure Decision**: Web application with frontend + backend. Using existing VibeTea structure with new auth modules added to both server and client. Supabase edge function hosted externally.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations requiring justification. Complexity is appropriate for the feature:

| Addition | Justification | Simpler Alternative Rejected |
|----------|---------------|------------------------------|
| Supabase dependency | Required for GitHub OAuth - implementing OAuth from scratch would be far more complex and error-prone | Rolling our own OAuth implementation |
| Session token store | Required for decoupling Supabase auth from WebSocket - validates once, then uses fast local token | Validating Supabase JWT on every WebSocket message |
| Public keys edge function | Allows monitor key management without server restart | Environment variables (rejected per spec) |
