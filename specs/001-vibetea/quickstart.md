# VibeTea Development Quickstart

**Feature**: 001-vibetea | **Date**: 2026-02-02

Quick reference for setting up and running VibeTea components locally.

---

## Prerequisites

| Tool | Version | Check |
|------|---------|-------|
| Rust | stable (1.75+) | `rustc --version` |
| Node.js | 20+ | `node --version` |
| pnpm | 8+ | `pnpm --version` |

**Install Rust**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Install pnpm**:
```bash
npm install -g pnpm
```

---

## Repository Structure

```
vibetea/
├── Cargo.toml           # Workspace root
├── monitor/             # Rust Monitor daemon
├── server/              # Rust Server
├── client/              # React/TypeScript dashboard
├── lefthook.yml         # Git hooks config
└── .github/workflows/   # CI pipeline
```

---

## Quick Commands

### All Components

```bash
# Install dependencies
cargo build                      # Rust (monitor + server)
cd client && pnpm install        # Client

# Run linting
cargo clippy --all-targets       # Rust
cd client && pnpm lint           # Client

# Run tests
cargo test                       # Rust
cd client && pnpm test           # Client

# Format code
cargo fmt                        # Rust
cd client && pnpm format         # Client
```

### Monitor

```bash
# Generate keypair (first time only)
cargo run -p vibetea-monitor -- init

# Run monitor (development)
VIBETEA_SERVER_URL=http://localhost:8080 \
VIBETEA_UNSAFE_NO_AUTH=true \
cargo run -p vibetea-monitor -- run

# Build release binary
cargo build --release -p vibetea-monitor
# Binary at: target/release/vibetea-monitor
```

### Server

```bash
# Run server (development, no auth)
VIBETEA_UNSAFE_NO_AUTH=true \
cargo run -p vibetea-server

# Run server (with auth)
VIBETEA_PUBLIC_KEYS="my-laptop:$(cat ~/.vibetea/key.pub | base64)" \
VIBETEA_SUBSCRIBER_TOKEN="dev-token-123" \
cargo run -p vibetea-server

# Build release binary
cargo build --release -p vibetea-server
# Binary at: target/release/vibetea-server
```

### Client

```bash
cd client

# Development server (hot reload)
pnpm dev

# Production build
pnpm build

# Preview production build
pnpm preview

# Type check
pnpm typecheck
```

---

## Development Workflow

### 1. Start Server (terminal 1)

```bash
VIBETEA_UNSAFE_NO_AUTH=true cargo run -p vibetea-server
```

Server runs at `http://localhost:8080`.

### 2. Start Client (terminal 2)

```bash
cd client && pnpm dev
```

Dashboard at `http://localhost:5173`.

### 3. Start Monitor (terminal 3)

```bash
VIBETEA_SERVER_URL=http://localhost:8080 \
VIBETEA_UNSAFE_NO_AUTH=true \
cargo run -p vibetea-monitor -- run
```

Monitor watches `~/.claude/projects/` for Claude Code sessions.

### 4. Use Claude Code

Open Claude Code in any project. Events should appear in the dashboard.

---

## Environment Variables

### Server

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VIBETEA_PUBLIC_KEYS` | Yes* | - | `source:base64_pubkey,...` |
| `VIBETEA_SUBSCRIBER_TOKEN` | Yes* | - | Client auth token |
| `PORT` | No | 8080 | HTTP port |
| `VIBETEA_UNSAFE_NO_AUTH` | No | false | Disable auth (dev only) |

*Not required if `VIBETEA_UNSAFE_NO_AUTH=true`

### Monitor

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VIBETEA_SERVER_URL` | Yes | - | Server URL |
| `VIBETEA_SOURCE_ID` | No | hostname | Monitor identifier |
| `VIBETEA_KEY_PATH` | No | ~/.vibetea | Keypair directory |
| `VIBETEA_CLAUDE_DIR` | No | ~/.claude | Claude Code directory |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer size |
| `VIBETEA_BASENAME_ALLOWLIST` | No | (all) | File extension filter |

### Client

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VITE_WS_URL` | No | ws://localhost:8080 | WebSocket URL |

---

## Testing

### Unit Tests

```bash
# All Rust tests
cargo test

# Specific crate
cargo test -p vibetea-monitor
cargo test -p vibetea-server

# Client tests
cd client && pnpm test
```

### Integration Tests

```bash
# Server integration tests (starts real server)
cargo test -p vibetea-server --test integration

# Monitor integration tests (requires mock server)
cargo test -p vibetea-monitor --test integration
```

### Manual Testing

**Test Server Health**:
```bash
curl http://localhost:8080/health
# {"status":"ok","connections":0}
```

**Test Event Ingestion** (unsafe mode):
```bash
curl -X POST http://localhost:8080/events \
  -H "Content-Type: application/json" \
  -d '{"id":"evt_test12345678901234567","source":"test","timestamp":"2026-02-02T00:00:00Z","type":"activity","payload":{"sessionId":"00000000-0000-0000-0000-000000000000"}}'
# Should return 202
```

**Test WebSocket** (unsafe mode):
```bash
websocat ws://localhost:8080/ws
# Receives events as JSON
```

---

## Git Hooks (Lefthook)

Pre-commit hooks run automatically:
- `cargo fmt --check` - Rust formatting
- `cargo clippy` - Rust linting
- `pnpm lint` - TypeScript linting
- `pnpm typecheck` - TypeScript type check

**Manual hook run**:
```bash
lefthook run pre-commit
```

---

## Troubleshooting

### Monitor not detecting Claude Code sessions

1. Check Claude Code directory exists: `ls ~/.claude/projects/`
2. Verify Monitor is watching: check logs for "Watching" message
3. Ensure Claude Code is active (writing to JSONL files)

### Events not appearing in dashboard

1. Check Server is running: `curl localhost:8080/health`
2. Check WebSocket connection: browser DevTools → Network → WS
3. Verify token matches (if auth enabled)

### Build errors

```bash
# Clean and rebuild
cargo clean && cargo build

# Update dependencies
cargo update
```

---

## Next Steps

After local development:

1. **Deploy Server**: See `docs/deployment.md` (Fly.io)
2. **Distribute Monitor**: Build cross-platform binaries via CI
3. **Host Client**: Deploy to CDN (Netlify/Vercel)

---

*Quickstart complete. For full documentation, see README.md.*
