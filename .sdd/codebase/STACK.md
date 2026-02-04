# Technology Stack

> **Purpose**: Document what executes in this codebase - languages, runtimes, frameworks, and critical dependencies.
> **Generated**: 2026-02-04
> **Last Updated**: 2026-02-04

## Languages & Runtimes

| Language | Version | Purpose |
|----------|---------|---------|
| TypeScript | 5.9.3 | Client-side application language |
| Rust | 2021 edition | Server-side and monitor application language |
| JavaScript/Deno | Latest | Supabase edge functions runtime |

## Frameworks

| Framework | Version | Purpose |
|-----------|---------|---------|
| React | 19.2.4 | Client UI library and component framework |
| Vite | 7.3.1 | Client build tool and dev server |
| Axum | 0.8 | Server HTTP framework and async runtime orchestration |
| Tokio | 1.43 | Async runtime for server and monitor applications |

## Critical Dependencies

| Package | Version | Purpose | Usage Scope |
|---------|---------|---------|-------------|
| @supabase/supabase-js | 2.94.0 | Client authentication and Supabase integration | Client authentication via GitHub OAuth |
| zustand | 5.0.11 | Client state management | Global app state for auth and UI |
| reqwest | 0.12 | HTTP client for Supabase API calls | Server JWT validation and public key fetching |
| ed25519-dalek | 2.1 | Ed25519 signature verification | Server event validation from monitors |
| serde/serde_json | 1.0 | Serialization and deserialization | Event serialization, config parsing, API responses |
| tower-http | 0.6 | HTTP middleware (CORS, tracing) | Server request handling and observability |
| tracing/tracing-subscriber | 0.3 | Structured logging and observability | Server and monitor diagnostics |

## Package Managers & Build Tools

| Tool | Version | Purpose |
|------|---------|---------|
| pnpm | Latest | Client package management |
| Cargo | Latest | Rust package management and build |
| npm | Latest | Dev tooling (scripts) |

## Runtime Environment

| Environment | Details |
|-------------|---------|
| Node.js | 18+ (inferred from package.json target) |
| Browser | Modern browsers supporting WebSocket, ES2020+ |
| Tokio async | Single-threaded event loop per worker |
| Deployment | Docker containers (optimized release builds with LTO) |
| OS Targets | Linux (primary), macOS (development), Windows (development) |

---

## Additional Stack Components

### Cryptography Layer
- **ed25519-dalek** 2.1 with RFC 8032 compliant verification
- **subtle** 2.6 for constant-time comparisons (timing attack mitigation)
- **rand** 0.9 for cryptographically secure random token generation
- **base64** 0.22 for token encoding/decoding

### Session Management
- In-memory session store with 5-minute TTL
- 32-byte random tokens, base64-url encoded (43 characters)
- Capacity limit of 10,000 concurrent sessions
- Thread-safe with RwLock-based interior mutability

### Styling & UI
- Tailwind CSS 4.1.18 with Vite integration
- Framer Motion 12.31.0 for animations
- Recharts 3.7.0 for data visualization
- TanStack React Virtual for list virtualization

### Testing & Quality
- Vitest 4.0.18 (client tests)
- Tokio-test 0.4 and WireMock 0.6 (server tests)
- serial_test 3.2 (for environment-dependent test isolation)
- ESLint 9.39.2 with TypeScript support
- Prettier 3.8.1 for code formatting

---

## What Does NOT Belong Here

- Directory structure → STRUCTURE.md
- System design patterns → ARCHITECTURE.md
- External service integrations → INTEGRATIONS.md
- Dev tools (linting, formatting) → CONVENTIONS.md
- Test frameworks → TESTING.md

---

*This document captures only what executes. Reflect Phase 2 (Supabase Authentication Foundational) additions including session store, Supabase client, and edge functions.*
