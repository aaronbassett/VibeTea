# Quickstart: Monitor TUI Development

**Feature**: 003-monitor-tui
**Date**: 2026-02-03

## Prerequisites

- Rust 1.75+ with 2021 edition
- A VibeTea server instance (or use `VIBETEA_UNSAFE_NO_AUTH=true` for local testing)

## Getting Started

### 1. Install Dependencies

The TUI requires two new dependencies. Add to `monitor/Cargo.toml`:

```toml
[dependencies]
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
```

Install with:

```bash
cd /home/ubuntu/Projects/VibeTea
cargo build -p vibetea-monitor
```

### 2. Run the TUI

```bash
# Default mode: TUI (new behavior)
cargo run -p vibetea-monitor

# Headless mode: existing CLI behavior
cargo run -p vibetea-monitor -- run

# Initialize keys (unchanged)
cargo run -p vibetea-monitor -- init
```

### 3. Environment Setup

For local development without a real server:

```bash
# No authentication (development only)
export VIBETEA_UNSAFE_NO_AUTH=true
export VIBETEA_SERVER_URL=http://localhost:8080

# Start server in another terminal
cargo run -p vibetea-server
```

For production-like testing:

```bash
# Generate keys
cargo run -p vibetea-monitor -- init

# Get your public key
cat ~/.vibetea/key.pub

# Add to server config (in server terminal)
export VIBETEA_PUBLIC_KEYS="my-monitor:$(cat ~/.vibetea/key.pub)"
cargo run -p vibetea-server
```

## Development Commands

### Build

```bash
# Build all workspace members
cargo build

# Build only monitor with verbose output
cargo build -p vibetea-monitor -v
```

### Test

```bash
# Run all monitor tests
cargo test -p vibetea-monitor

# Run TUI-specific tests
cargo test -p vibetea-monitor tui

# Run with output (for debugging)
cargo test -p vibetea-monitor -- --nocapture

# Run sequentially (if tests interfere)
cargo test -p vibetea-monitor -- --test-threads=1
```

### Lint and Format

```bash
# Check formatting
cargo fmt --check

# Fix formatting
cargo fmt

# Run clippy
cargo clippy -p vibetea-monitor -- -D warnings

# Run all checks (pre-commit)
lefthook run pre-commit
```

## Project Structure

```
monitor/
├── src/
│   ├── main.rs          # Entry point, CLI parsing
│   ├── lib.rs           # Module exports
│   ├── config.rs        # Configuration
│   ├── crypto.rs        # Ed25519 operations
│   ├── sender.rs        # HTTP sender (add metrics)
│   ├── watcher.rs       # File watching
│   ├── parser.rs        # JSONL parsing
│   ├── privacy.rs       # Privacy pipeline
│   ├── types.rs         # Event types
│   ├── error.rs         # Error types
│   └── tui/             # NEW: TUI module
│       ├── mod.rs       # Module exports
│       ├── app.rs       # Application state
│       ├── ui.rs        # Main render function
│       ├── input.rs     # Input handling
│       ├── terminal.rs  # Terminal setup
│       └── widgets/     # Widget components
│           ├── mod.rs
│           ├── logo.rs
│           ├── setup_form.rs
│           ├── event_stream.rs
│           ├── credentials.rs
│           └── stats_footer.rs
└── tests/
    └── tui_state_test.rs
```

## TUI Navigation

### Setup Screen

| Key | Action |
|-----|--------|
| Tab / ↓ | Next field |
| Shift+Tab / ↑ | Previous field |
| Enter | Submit (on Submit button) |
| Esc | Cancel / Exit |
| ← → | Toggle key option |
| Backspace | Delete character |

### Dashboard Screen

| Key | Action |
|-----|--------|
| q / Esc | Quit |
| ↑ / k | Scroll up |
| ↓ / j | Scroll down |
| PageUp | Scroll page up |
| PageDown | Scroll page down |
| Home / g | Scroll to top |
| End / G | Scroll to bottom (resume auto-scroll) |

## Debugging

### Terminal Issues

If the terminal is not restored properly:

```bash
# Reset terminal
reset

# Or
stty sane
```

### Logging

TUI mode suppresses normal stderr logging. To enable:

```bash
# Redirect logs to file
RUST_LOG=debug cargo run -p vibetea-monitor 2> /tmp/monitor.log

# In another terminal, watch logs
tail -f /tmp/monitor.log
```

### Testing in tmux/screen

```bash
# Recommended tmux settings
# Add to ~/.tmux.conf:
set -sg escape-time 10
set -g focus-events on
```

## Common Issues

### Terminal Too Small

The TUI requires minimum 80x24 characters. Resize your terminal or use headless mode:

```bash
cargo run -p vibetea-monitor -- run
```

### Colors Look Wrong

If colors appear incorrect, check your terminal's color theme or set:

```bash
export NO_COLOR=1
cargo run -p vibetea-monitor
```

### Input Lag in tmux

Add to `~/.tmux.conf`:

```
set -sg escape-time 10
```

Then reload: `tmux source ~/.tmux.conf`

### Panic Doesn't Restore Terminal

This should not happen with proper panic hook setup. If it does:

```bash
reset
```

Report the issue with the panic message.

## Related Documentation

- [spec.md](./spec.md) - Feature specification
- [plan.md](./plan.md) - Implementation plan
- [data-model.md](./data-model.md) - State and type definitions
- [contracts/](./contracts/) - API interfaces
- [research.md](./research.md) - Technology research
