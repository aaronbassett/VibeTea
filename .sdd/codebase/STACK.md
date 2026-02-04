# Technology Stack

**Status**: Phase 5 Implementation Complete - Skill/slash command tracking via history.jsonl
**Generated**: 2026-02-04
**Last Updated**: 2026-02-04

## Languages & Runtimes

| Component | Language   | Version | Purpose |
|-----------|-----------|---------|---------|
| Monitor   | Rust      | 2021    | Native file watching, JSONL parsing, privacy filtering, event signing, skill tracking, HTTP transmission |
| Server    | Rust      | 2021    | Async HTTP/WebSocket server for event distribution and rate limiting |
| Client    | TypeScript | 5.x     | Type-safe React UI for session visualization and real-time monitoring |

## Frameworks & Core Libraries

### Rust (Monitor & Server)

| Package            | Version | Purpose | Used By |
|--------------------|---------|---------|----------|
| tokio              | 1.43    | Async runtime with full features (threads, signals, timers) | Server, Monitor |
| axum               | 0.8     | HTTP/WebSocket server framework with routing and middleware | Server |
| tower              | 0.5     | Composable middleware and service abstractions | Server |
| tower-http         | 0.6     | HTTP utilities (CORS, tracing, compression) | Server |
| reqwest            | 0.12    | HTTP client library with connection pooling and timeouts | Monitor sender, Server tests |
| serde              | 1.0     | Serialization/deserialization with derive macros | All |
| serde_json         | 1.0     | JSON format handling and streaming | All |
| ed25519-dalek      | 2.1     | Ed25519 cryptographic signing and verification | Server auth, Monitor crypto |
| uuid               | 1.11    | Unique identifiers (v4, v5) for events and sessions | Server, Monitor |
| chrono             | 0.4     | Timestamp handling with serde support | Server, Monitor |
| thiserror          | 2.0     | Derive macros for error types | Server, Monitor |
| anyhow             | 1.0     | Flexible error handling and context | Server, Monitor |
| tracing            | 0.1     | Structured logging framework | Server, Monitor |
| tracing-subscriber | 0.3     | Logging implementation with JSON and env-filter | Server, Monitor |
| notify             | 8.0     | Cross-platform file system watching (inotify/FSEvents) | Monitor watcher, skill tracker |
| base64             | 0.22    | Base64 encoding/decoding for signatures and keys | Server, Monitor |
| rand               | 0.9     | Cryptographically secure random number generation | Server, Monitor |
| directories        | 6.0     | Platform-specific standard directory paths | Monitor config |
| gethostname        | 1.0     | System hostname retrieval for monitor source ID | Monitor config |
| subtle             | 2.6     | Constant-time comparison to prevent timing attacks | Server auth |
| futures-util       | 0.3     | WebSocket stream utilities and async helpers | Server |
| futures            | 0.3     | Futures trait and utilities for async coordination | Monitor |
| lru                | 0.12    | LRU cache for session tracking | Monitor stats tracker |
| clap               | 4.5     | CLI argument parsing with derive macros | Monitor CLI |
| serial_test        | 3.2     | Serial test execution for env var isolation | Monitor, Server tests |

### TypeScript/JavaScript (Client)

| Package                    | Version  | Purpose |
|---------------------------|----------|---------|
| React                      | ^19.2.4  | UI framework for component-based architecture |
| React DOM                  | ^19.2.4  | DOM rendering and lifecycle management |
| TypeScript                 | ^5.9.3   | Static type checking and transpilation |
| Vite                       | ^7.3.1   | Build tool and dev server with HMR |
| Tailwind CSS               | ^4.1.18  | Utility-first CSS framework for styling |
| Zustand                    | ^5.0.11  | Lightweight state management without boilerplate |
| @tanstack/react-virtual    | ^3.13.18 | Virtual scrolling for efficient rendering of 1000+ events |
| @vitejs/plugin-react       | ^5.1.3   | React Fast Refresh for HMR in Vite |
| @tailwindcss/vite          | ^4.1.18  | Tailwind CSS Vite plugin for CSS compilation |
| vite-plugin-compression2   | ^2.4.0   | Brotli compression for optimized production builds |

## Build Tools & Package Managers

| Tool     | Version  | Purpose |
|----------|----------|---------|
| cargo    | -        | Rust package manager and build system with workspaces |
| pnpm     | -        | Node.js package manager with monorepo support |
| rustfmt  | -        | Rust code formatter enforcing consistent style |
| clippy   | -        | Rust linter for code quality |
| prettier | ^3.8.1   | Code formatter for TypeScript and CSS |
| ESLint   | ^9.39.2  | Linter for JavaScript/TypeScript code quality |

## Testing Infrastructure

### Rust Testing
| Package      | Version | Purpose |
|--------------|---------|---------|
| tokio-test   | 0.4     | Tokio testing utilities for async tests |
| tempfile     | 3.15    | Temporary file/directory management for tests |
| wiremock     | 0.6     | HTTP mocking for integration tests |

### TypeScript Testing
| Package                | Version  | Purpose |
|------------------------|----------|---------|
| Vitest                 | ^4.0.18  | Unit and component testing framework |
| @testing-library/react | ^16.3.2  | React component testing utilities |
| @testing-library/jest-dom | ^6.9.1 | DOM assertion helpers |
| jsdom                  | ^28.0.0  | Full DOM implementation for tests |
| happy-dom              | ^20.5.0  | Lightweight DOM for faster tests |

## Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` (workspace) | Rust workspace configuration with shared dependencies and edition settings |
| `server/Cargo.toml` | Server package manifest with axum HTTP framework |
| `monitor/Cargo.toml` | Monitor package manifest with crypto, file watching, CLI, and skill tracking |
| `client/vite.config.ts` | Vite build configuration with React, Tailwind, compression, WebSocket proxy |
| `client/tsconfig.json` | TypeScript strict mode configuration (ES2020 target) |
| `client/eslint.config.js` | ESLint flat config with TypeScript support |

## Runtime Environment

| Aspect | Details |
|--------|---------|
| Server Runtime | Rust binary compiled with tokio async runtime |
| Client Runtime | Browser (ES2020+ compatible) with modern module support |
| Monitor Runtime | Native binary (Linux ELF, macOS Mach-O, Windows PE) with CLI |
| Node.js | Required for development and client builds only (not production) |
| Async Model | Tokio for Rust, Promises/async-await for TypeScript |
| WebSocket Support | Native support via axum (server), browser APIs (client) |
| File System Monitoring | Rust notify crate for cross-platform inotify/FSEvents support |
| CLI Support | Manual command parsing for monitor (init, run, help, version) |

## Communication Protocols & Formats

| Interface | Protocol | Format | Auth Method |
|-----------|----------|--------|------------|
| Monitor → Server | HTTPS POST | JSON | Ed25519 signature in X-Signature header |
| Server → Client | WebSocket (ws/wss) | JSON | Bearer token in query parameter |
| Client → Server | WebSocket (ws/wss) | JSON | Bearer token |
| Monitor → File System | Native file I/O | JSONL | N/A (local file access with permissions) |

## Data Serialization

| Component | Serialization | Notes |
|-----------|---------------|-------|
| Server/Monitor | serde (Rust) | JSON with snake_case for serde rename |
| Client | TypeScript/JSON | camelCase for API contracts |
| Events | serde_json | Standardized event schema across all components |
| Claude Code Sessions | JSONL (JSON Lines) | Privacy-first parsing extracting metadata only |
| History File | JSONL (JSON Lines) | One JSON object per line, append-only file |
| Cryptographic Keys | Base64 + raw bytes | Public keys base64-encoded, private keys raw 32-byte seeds |

## Build Output

| Component | Output | Format | Deployment |
|-----------|--------|--------|-----------|
| Server | Binary | ELF (Linux) | Docker container on Fly.io with minimal base |
| Monitor | Binary | ELF/Mach-O/PE | Cross-platform standalone executable |
| Client | Static files | JavaScript + CSS with Brotli compression | CDN (Netlify/Vercel/Cloudflare) |

## Module Organization

### Client (`client/src`)
- `components/` - React components
  - `ConnectionStatus.tsx` - Visual WebSocket connection indicator
  - `TokenForm.tsx` - Token management with localStorage persistence
  - `EventStream.tsx` - Virtual scrolling for 1000+ events with auto-scroll
  - `Heatmap.tsx` - Activity heatmap with 7/30-day views and color scale
  - `SessionOverview.tsx` - Session cards with real-time activity indicators
- `hooks/` - Custom React hooks
  - `useEventStore.ts` - Zustand store for event state with session timeout management
  - `useWebSocket.ts` - WebSocket management with auto-reconnect
  - `useSessionTimeouts.ts` - Periodic session state transitions
- `types/events.ts` - Discriminated union event types matching server schema
- `utils/formatting.ts` - Timestamp and duration formatting (5 functions)
- `__tests__/` - Test suite with 33+ test cases

### Server (`server/src`)
- `config.rs` - Configuration from environment (ports, keys, tokens)
- `auth.rs` - Ed25519 signature verification with constant-time comparison
- `broadcast.rs` - Event broadcasting via tokio channels with filtering
- `rate_limit.rs` - Per-source token bucket rate limiting (100 events/sec)
- `routes.rs` - HTTP endpoints (POST /events, GET /ws, GET /health)
- `error.rs` - Comprehensive error types and handling
- `types.rs` - Event types and data models
- `main.rs` - Server binary entry point

### Monitor (`monitor/src`)
- `config.rs` - Configuration from environment variables
- `types.rs` - Event types and definitions
- `parser.rs` - Claude Code JSONL parser with privacy-first extraction
- `watcher.rs` - File system watcher with position tracking
- `privacy.rs` - Privacy pipeline for event sanitization
- `crypto.rs` - Ed25519 keypair generation, loading, and signing
- `sender.rs` - HTTP client with buffering, retry, and rate limit handling
- `main.rs` - CLI entry point (init, run commands)
- `trackers/` - Specialized tracking modules
  - `agent_tracker.rs` - Task tool agent spawn detection
  - `stats_tracker.rs` - Token and session statistics
  - `skill_tracker.rs` - Skill/slash command tracking from history.jsonl (1837 lines)
- `utils/` - Utility functions
  - `tokenize.rs` - Skill name extraction from display strings

## Deployment Targets

| Component | Target | Container | Notes |
|-----------|--------|-----------|-------|
| Server | Fly.io | Docker | Single Rust binary with minimal base image |
| Client | CDN | Static files | Optimized builds with Brotli compression |
| Monitor | Local | Native binary | Users download and run locally |

## Phase 5 - Skill Tracker Implementation

**Completion Date**: 2026-02-03

### New Module: `monitor/src/trackers/skill_tracker.rs` (1837 lines)

**Core Types**:
- `SkillInvocationEvent` - Event emitted when user invokes a skill
- `HistoryEntry` - Parsed entry from history.jsonl file
- `SkillTracker` - File watcher for `~/.claude/history.jsonl`
- `SkillTrackerConfig` - Configuration option for startup behavior
- `SkillTrackerError` - Comprehensive error handling

**Parsing Functions**:
- `parse_history_entry()` - Parses single JSON line with validation
- `parse_history_entries()` - Parses multiple lines with lenient skipping
- `create_skill_invocation_event()` - Constructs event from entry

**File Watching**:
- Monitors `~/.claude/history.jsonl` with `notify` crate
- Tail-like behavior with atomic byte offset tracking
- Handles file truncation gracefully
- Events emitted via mpsc channel

**Test Coverage**: 60+ comprehensive tests validating parsing, file operations, async behavior

### New Module: `monitor/src/utils/tokenize.rs`

**Skill Name Extraction**:
- `extract_skill_name()` - Parses skill command name from display
- Handles colons: `/sdd:plan` → `sdd:plan`
- Handles hyphens: `/review-pr` → `review-pr`
- Handles quoted names and arguments

### Enhanced Integration

**Main Event Loop** (`monitor/src/main.rs`):
- SkillTracker initialization with error handling
- Skill event channel processing
- Integration with event sender

**Event Types** (`monitor/src/types.rs`):
- `EventType::SkillInvocation` - New enum variant
- `EventPayload::SkillInvocation` - Payload wrapper

**Privacy-First Approach**:
- Only skill name extracted from display
- Command arguments intentionally omitted
- Project path and timestamp included for context

## Key Features & Capabilities

### Architecture
- Distributed event system: Monitor → Server → Client
- Privacy-by-design throughout pipeline
- Cryptographic authentication for event integrity
- Efficient file watching with position tracking
- Virtual scrolling for high-volume event display

### Monitoring Capabilities
- Claude Code session lifecycle tracking (start/end)
- Tool invocation tracking with context
- Task tool agent spawn detection and tracking
- Skill/slash command invocation tracking (Phase 5)
- Token usage and statistics accumulation
- Real-time activity heatmaps

### Reliability
- Exponential backoff retry with jitter
- Rate limiting protection
- Graceful shutdown with event flushing
- Lenient JSONL parsing
- Auto-reconnection with backoff

### Security
- Ed25519 signatures on all events
- Constant-time signature verification
- Bearer token authentication for clients
- File permission enforcement (0600 private keys)
- Privacy pipeline stripping sensitive data

---

*This document captures production technologies, frameworks, and dependencies. Version specifications reflect compatibility constraints.*
