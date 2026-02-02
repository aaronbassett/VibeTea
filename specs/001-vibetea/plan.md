# Implementation Plan: VibeTea

**Branch**: `001-vibetea` | **Date**: 2026-02-02 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-vibetea/spec.md`

## Summary

VibeTea is a real-time AI coding assistant activity monitoring system with a hub-and-spoke architecture. The system consists of three components:

1. **Monitor** (Rust): Daemon that watches Claude Code session files, extracts metadata events, strips all sensitive content via a privacy pipeline, and forwards events to the Server
2. **Server** (Rust): HTTP/WebSocket event hub using axum that receives signed events from Monitors and broadcasts to authenticated Clients with filtering support
3. **Client** (TypeScript/React): Real-time dashboard displaying event stream, activity heatmap, session overview, and usage statistics

Privacy is paramount - the Monitor's privacy pipeline ensures no source code, prompts, file contents, or sensitive data is ever transmitted.

## Technical Context

**Languages/Versions**:
- Monitor & Server: Rust 2021 edition (stable toolchain)
- Client: TypeScript 5.x with React 18+

**Primary Dependencies**:
- Rust: tokio (async runtime), axum (HTTP/WS server), notify (file watching), ed25519-dalek (signing), serde (JSON), reqwest (HTTP client), thiserror/anyhow (errors)
- Client: React, Zustand (state management), @tanstack/react-virtual (virtualization), Vite (build), Vitest (testing)

**Storage**: None - events are transient (in-memory only, no persistence)

**Testing**:
- Rust: cargo test (unit + integration)
- Client: Vitest (component + WebSocket mock tests)

**Target Platforms**:
- Server: Fly.io (Docker container, single binary)
- Client: CDN (static files - Netlify/Vercel/Cloudflare)
- Monitor: Local binary (Linux x86_64-musl, macOS x86_64/aarch64, Windows x86_64-msvc)

**Project Type**: Multi-component (3 separate projects: monitor, server, client)

**Performance Goals**:
- Event latency (end-to-end): < 100 ms
- Server concurrent connections: 100+
- Client time to interactive: < 2 seconds (3G)
- Monitor CPU (idle): < 1%
- Monitor memory: < 5 MB
- Monitor binary: < 10 MB

**Constraints**:
- Privacy: Never log/transmit source code, prompts, file contents, full paths
- Rate limit: 100 events/sec per source, 1000 events/sec global
- Buffer capacity: Monitor 1000 events, Server 100 per-client
- Single-instance v1 (no horizontal scaling)

**Scale/Scope**: Single-user/small team use cases (no multi-tenancy in v1)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Implementation Approach |
|-----------|--------|------------------------|
| **I. Privacy by Design** | ✅ PASS | Privacy pipeline strips all sensitive data before transmission; tests explicitly verify nothing sensitive logged |
| **II. Unix Philosophy** | ✅ PASS | 3 separate components (Monitor, Server, Client) with clear responsibilities; JSON communication |
| **III. Keep It Simple (KISS)** | ✅ PASS | No speculative features; standard patterns (REST, WebSocket, file watching) |
| **IV. Event-Driven** | ✅ PASS | Monitor emits events → Server broadcasts → Client subscribes; no polling |
| **V. Test What Matters** | ✅ PASS | Critical paths tested; privacy guarantees have explicit tests |
| **VI. Fail Fast & Loud** | ✅ PASS | Exit on invalid config; clear error messages; graceful degradation documented |
| **VII. Modularity** | ✅ PASS | Independent components; explicit interfaces; no circular dependencies |

**Language Standards Compliance**:
- Rust: clippy + rustfmt configured; thiserror/anyhow for errors; tokio documented as async runtime
- TypeScript: strict mode; Prettier + ESLint; no `any` policy

**No violations requiring justification.**

## Project Structure

### Documentation (this feature)

```text
specs/001-vibetea/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (API schemas)
└── tasks.md             # Phase 2 output (/sdd:tasks)
```

### Source Code (repository root)

```text
# Cargo workspace with 3 members

Cargo.toml               # Workspace root
monitor/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point (init, run commands)
│   ├── lib.rs           # Public API
│   ├── watcher.rs       # File system watcher (notify)
│   ├── parser.rs        # JSONL parser for Claude Code events
│   ├── privacy.rs       # Privacy pipeline (data stripping)
│   ├── sender.rs        # HTTP client + buffering + retry
│   ├── crypto.rs        # Ed25519 keypair generation/signing
│   └── config.rs        # Environment variable configuration

### Ed25519 Key Lifecycle

**Key Generation** (`vibetea init` command):
1. Check if `~/.vibetea/key.priv` exists; if yes, prompt to overwrite or abort
2. Generate Ed25519 keypair using `ed25519-dalek` with `rand::rngs::OsRng`
3. Save private key to `~/.vibetea/key.priv` (raw 32 bytes, 0600 permissions)
4. Save public key to `~/.vibetea/key.pub` (base64 encoded, 0644 permissions)
5. Display public key to stdout for user to register with Server

**Key Usage** (Monitor runtime):
1. Load private key from `VIBETEA_KEY_PATH` (default: `~/.vibetea`)
2. Sign each event batch with private key before POST to Server
3. Include signature in `X-Signature` header (base64 encoded)

**Key Registration** (Server configuration):
1. User adds Monitor's public key to `VIBETEA_PUBLIC_KEYS` env var
2. Format: `source-id:base64-pubkey` (e.g., `laptop:MCowBQ...`)

**v1 Scope**: No automatic key rotation. Users must manually regenerate keys and update server configuration if compromise is suspected.
└── tests/
    ├── privacy_test.rs  # Verify nothing sensitive transmitted
    └── integration/     # Mock server tests

server/
├── Cargo.toml
├── src/
│   ├── main.rs          # Server entry point
│   ├── lib.rs           # Public API
│   ├── routes.rs        # axum routes (POST /events, GET /ws, GET /health)
│   ├── auth.rs          # Ed25519 verification, token validation
│   ├── broadcast.rs     # WebSocket broadcast channel
│   ├── rate_limit.rs    # Per-source rate limiting
│   └── config.rs        # Environment variable configuration
└── tests/
    ├── integration/     # Full HTTP/WS tests
    └── auth_test.rs     # Signature verification tests

client/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── src/
│   ├── main.tsx         # React entry point
│   ├── App.tsx          # Main layout
│   ├── components/
│   │   ├── EventStream.tsx      # Live event feed
│   │   ├── Heatmap.tsx          # Activity heatmap
│   │   ├── SessionOverview.tsx  # Session cards
│   │   ├── StatsPanel.tsx       # Statistics
│   │   ├── ConnectionStatus.tsx # Status indicator
│   │   └── TokenForm.tsx        # Auth token input
│   ├── hooks/
│   │   ├── useWebSocket.ts      # WebSocket connection + reconnect
│   │   └── useEventStore.ts     # Event state management
│   ├── types/
│   │   └── events.ts            # TypeScript interfaces
│   └── utils/
│       └── formatting.ts        # Timestamp, duration formatting
└── tests/
    ├── components/      # Component tests
    └── hooks/           # Hook tests with mock WebSocket

# Shared configuration
lefthook.yml             # Git hooks (cross-language)
.github/
└── workflows/
    └── ci.yml           # Build, test, lint all components
```

**Structure Decision**: Cargo workspace at root with 3 members (monitor, server, client). This enables shared Rust types between monitor and server while keeping client as a separate Node.js project. The workspace approach allows `cargo build --release` to produce both binaries efficiently.

## Complexity Tracking

> No violations requiring justification - all patterns are standard and simple.

## Implementation Phases

### Phase 0: Research (Complete)

Research questions to resolve before design:
1. Claude Code JSONL format and event types
2. Ed25519 signing best practices in Rust
3. axum WebSocket broadcast patterns
4. React state management for real-time events
5. Vite configuration for production builds

### Phase 1: Design & Contracts

Deliverables:
- `research.md` - Research findings and decisions
- `data-model.md` - Event schema, entity definitions
- `contracts/` - OpenAPI spec for Server API
- `quickstart.md` - Development setup guide

### Phase 2: Development Environment (Complete)

Deliverables:
- ✅ Cargo workspace configuration (`Cargo.toml`, `monitor/Cargo.toml`, `server/Cargo.toml`)
- ✅ Rust tooling (clippy, rustfmt via lefthook)
- ✅ TypeScript tooling (ESLint, Prettier, Vitest in `client/`)
- ✅ Lefthook git hooks (`lefthook.yml`)
- ✅ GitHub Actions CI workflow (`.github/workflows/ci.yml`) with privacy validation scan
- ✅ Privacy validation in CI: workflow includes step to scan test output and logs for sensitive data patterns

---

## Generated Artifacts

| Artifact | Path | Description |
|----------|------|-------------|
| Specification | `specs/001-vibetea/spec.md` | Feature specification with user stories |
| Plan | `specs/001-vibetea/plan.md` | Implementation plan (this file) |
| Research | `specs/001-vibetea/research.md` | Technology decisions |
| Data Model | `specs/001-vibetea/data-model.md` | Event schema and entities |
| API Contract | `specs/001-vibetea/contracts/openapi.yaml` | OpenAPI 3.1 spec |
| Quickstart | `specs/001-vibetea/quickstart.md` | Development setup guide |
| Cargo Workspace | `Cargo.toml` | Workspace root |
| Monitor Config | `monitor/Cargo.toml` | Monitor crate dependencies |
| Server Config | `server/Cargo.toml` | Server crate dependencies |
| Client Config | `client/package.json` | Client dependencies |
| Vite Config | `client/vite.config.ts` | Build configuration |
| TypeScript Config | `client/tsconfig.json` | TypeScript settings |
| Tailwind Config | `client/tailwind.config.js` | Styling configuration |
| Lefthook | `lefthook.yml` | Git hooks (Rust + TypeScript) |
| CI Workflow | `.github/workflows/ci.yml` | GitHub Actions pipeline |

---

*Planning complete. Run `/sdd:tasks` to generate implementation tasks.*
