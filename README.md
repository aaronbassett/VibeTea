# VibeTea

[![CI](https://img.shields.io/github/actions/workflow/status/aaronbassett/VibeTea/ci.yml?style=flat-square)](https://github.com/aaronbassett/VibeTea/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg?style=flat-square)](https://www.rust-lang.org/)
[![Node](https://img.shields.io/badge/node-20%2B-green.svg?style=flat-square)](https://nodejs.org/)

VibeTea is a real-time event aggregation and broadcast system for AI coding assistants. It consolidates activity streams from multiple AI agents (Claude Code, Cursor, Copilot, etc.) into a unified WebSocket feed, enabling developers to build dashboards, analytics tools, and integrations that provide visibility into their AI-assisted development workflow.

**Privacy is paramount.** VibeTea broadcasts only structural metadata (event types, tool categories, timestamps) and never transmits code, prompts, file contents, or any sensitive information.

## Architecture

VibeTea follows a hub-and-spoke architecture where the Server acts as a central event bus:

```
+--------------+    +--------------+    +--------------+
|   Claude     |    |   Cursor     |    |   Copilot    |
|   Monitor    |    |   Monitor    |    |   Monitor    |
+------+-------+    +------+-------+    +------+-------+
       |                   |                   |
       +-------------------+-------------------+
                           |
                           v
                  +--------+--------+
                  |    VibeTea      |
                  |     Server      |
                  +--------+--------+
                           |
       +-------------------+-------------------+
       v                   v                   v
+--------------+    +--------------+    +--------------+
|  Dashboard   |    |   Custom     |    |    CLI       |
|   Client     |    | Integration  |    |   Client     |
+--------------+    +--------------+    +--------------+
```

## Components

| Component | Description | Tech Stack |
|-----------|-------------|------------|
| **Monitor** | Lightweight daemon watching AI agent activity | Rust |
| **Server** | Event hub receiving and broadcasting events | Rust, Axum, Tokio |
| **Client** | Real-time dashboard for visualizing events | React, TypeScript, Vite |

## Quick Start

### Prerequisites

- Rust 1.75+ with Cargo
- Node.js 20+ with pnpm
- A running VibeTea Server instance

### Running the Server

```bash
# Clone the repository
git clone https://github.com/aaronbassett/VibeTea.git
cd VibeTea

# Build and run the server
cargo run --package vibetea-server --release
```

The server starts on `http://localhost:3000` by default.

### Running the Monitor

```bash
# Generate a keypair (first time only)
cargo run --package vibetea-monitor --release -- keygen

# Set required environment variables
export VIBETEA_SERVER_URL="https://localhost:3000"

# Run the monitor
cargo run --package vibetea-monitor --release -- run
```

The `keygen` command creates Ed25519 keys in `~/.vibetea/` and outputs the public key to register with the server.

### Running the Client Dashboard

```bash
cd client

# Install dependencies
pnpm install

# Start development server
pnpm dev
```

The dashboard will be available at `http://localhost:5173`.

## Configuration

### Monitor Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `VIBETEA_SERVER_URL` | Required | Server URL (e.g., `https://vibetea.fly.dev`) |
| `VIBETEA_SOURCE_ID` | hostname | Monitor identifier (must match key registration) |
| `VIBETEA_KEY_PATH` | `~/.vibetea` | Directory containing `key.priv` and `key.pub` |
| `VIBETEA_CLAUDE_DIR` | `~/.claude` | Claude Code config directory |
| `VIBETEA_BUFFER_SIZE` | 1000 | Events to buffer during disconnect |
| `VIBETEA_BASENAME_ALLOWLIST` | (all) | Comma-separated file extensions to include |

### Server Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `VIBETEA_HOST` | `0.0.0.0` | Host to bind to |
| `VIBETEA_PORT` | `3000` | Port to listen on |
| `VIBETEA_PUBLIC_KEYS` | Required | Monitor public keys. Format: `source1:pubkey1,source2:pubkey2` |
| `VIBETEA_AUTH_TOKEN` | Required | Bearer token for WebSocket client authentication |

### Authentication

VibeTea uses two authentication mechanisms:

**Monitors → Server (Ed25519 Signatures)**
- Monitors sign event payloads with Ed25519 private keys
- Server verifies signatures using registered public keys
- Headers: `X-Source-ID` (monitor identifier), `X-Signature` (base64-encoded signature)
- Generate keys: `vibetea-monitor keygen`
- Register public key via `VIBETEA_PUBLIC_KEYS` on the server

**Clients → Server (Bearer Token)**
- Dashboard and WebSocket clients use bearer token authentication
- Token passed via `?token=` query parameter on WebSocket connections
- Configured via `VIBETEA_AUTH_TOKEN` on both server and client

## API Reference

### Server Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/events` | POST | Ingest events from monitors |
| `/ws` | GET | WebSocket subscription endpoint |
| `/health` | GET | Health check with connection stats |

### Event Schema

```json
{
  "id": "evt_abc123",
  "source": "macbook-pro",
  "timestamp": "2025-01-15T14:30:00Z",
  "type": "tool",
  "payload": {
    "sessionId": "sess_xyz789",
    "project": "vibetea",
    "tool": "file_read",
    "status": "completed",
    "context": "server.rs"
  }
}
```

**Event Types:** `session`, `activity`, `tool`, `agent`, `summary`, `error`

## Development

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

### Running Tests

```bash
# Rust tests (must run single-threaded due to env var tests)
cargo test --workspace -- --test-threads=1

# Client tests
cd client && pnpm test
```

### Code Quality

```bash
# Rust formatting and linting
cargo fmt --all
cargo clippy --all-targets -- -D warnings

# Client formatting and linting
cd client
pnpm lint
pnpm format
pnpm typecheck
```

## Privacy

VibeTea implements strict privacy controls. The Monitor only transmits:

**Allowed:**
- Event type and status
- Tool category (e.g., "file_read", "terminal")
- File basenames (not full paths)
- Token counts
- Project names
- Timestamps and session IDs

**Never transmitted:**
- File contents, code, or diffs
- User prompts or assistant responses
- Full file paths
- Bash commands (only descriptions)
- Search queries, URLs, or API responses
- Error messages or tool output

## Security

For security concerns or vulnerability reports, please see [SECURITY.md](SECURITY.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework for Rust
- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [React](https://react.dev/) - UI library
- [Vite](https://vitejs.dev/) - Frontend build tool
- [TailwindCSS](https://tailwindcss.com/) - CSS framework
