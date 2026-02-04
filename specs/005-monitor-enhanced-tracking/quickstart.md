# Quickstart: Monitor Enhanced Data Tracking

**Date**: 2026-02-03
**Branch**: `005-monitor-enhanced-tracking`
**Status**: Phase 1 Design

## Prerequisites

- Rust 2021 edition toolchain
- Node.js 20+ with pnpm
- Access to `~/.claude/` directory (Claude Code installation)

## Development Setup

### 1. Clone and Switch Branch

```bash
git clone https://github.com/aaronbassett/VibeTea.git
cd VibeTea
git checkout 005-monitor-enhanced-tracking
```

### 2. Build Monitor

```bash
cd monitor
cargo build
cargo clippy
cargo fmt --check
```

### 3. Build Server

```bash
cd server
cargo build
```

### 4. Build Client

```bash
cd client
pnpm install
pnpm build
```

## Testing Enhanced Tracking

### Create Test Data

For local testing, create mock files in a test Claude directory:

```bash
export VIBETEA_CLAUDE_DIR="$HOME/.vibetea-test-claude"
mkdir -p "$VIBETEA_CLAUDE_DIR"/{todos,file-history,projects}
```

#### Mock stats-cache.json

```bash
cat > "$VIBETEA_CLAUDE_DIR/stats-cache.json" << 'EOF'
{
  "totalSessions": 10,
  "totalMessages": 150,
  "totalToolUsage": 500,
  "longestSession": "01:30:00",
  "hourCounts": {
    "9": 20, "10": 35, "11": 40, "14": 25, "15": 30
  },
  "modelUsage": {
    "claude-opus-4-5-20251101": {
      "inputTokens": 100000,
      "outputTokens": 25000,
      "cacheReadInputTokens": 50000,
      "cacheCreationInputTokens": 10000
    },
    "claude-sonnet-4-20250514": {
      "inputTokens": 200000,
      "outputTokens": 50000,
      "cacheReadInputTokens": 100000,
      "cacheCreationInputTokens": 20000
    }
  }
}
EOF
```

#### Mock history.jsonl

```bash
cat > "$VIBETEA_CLAUDE_DIR/history.jsonl" << 'EOF'
{"display": "/commit", "timestamp": 1738567268363, "project": "/home/user/project", "sessionId": "test-session-001"}
{"display": "/sdd:plan", "timestamp": 1738567368363, "project": "/home/user/project", "sessionId": "test-session-001"}
{"display": "/worktrees:status", "timestamp": 1738567468363, "project": "/home/user/project", "sessionId": "test-session-002"}
EOF
```

#### Mock todo file

```bash
mkdir -p "$VIBETEA_CLAUDE_DIR/todos"
cat > "$VIBETEA_CLAUDE_DIR/todos/test-session-001-agent-test-session-001.json" << 'EOF'
[
  {"content": "Task 1", "status": "completed", "activeForm": null},
  {"content": "Task 2", "status": "in_progress", "activeForm": "Working..."},
  {"content": "Task 3", "status": "pending", "activeForm": null}
]
EOF
```

#### Mock file-history

```bash
mkdir -p "$VIBETEA_CLAUDE_DIR/file-history/test-session-001"
echo "Line 1" > "$VIBETEA_CLAUDE_DIR/file-history/test-session-001/abcd1234efgh5678@v1"
echo -e "Line 1\nLine 2" > "$VIBETEA_CLAUDE_DIR/file-history/test-session-001/abcd1234efgh5678@v2"
```

#### Mock project session

```bash
mkdir -p "$VIBETEA_CLAUDE_DIR/projects/-home-user-project"
cat > "$VIBETEA_CLAUDE_DIR/projects/-home-user-project/test-session-001.jsonl" << 'EOF'
{"type": "user", "message": "Hello"}
{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "Task", "input": {"subagent_type": "devs:rust-dev", "description": "Test task"}}]}, "timestamp": "2026-02-03T10:00:00Z"}
EOF
```

### Run Monitor

```bash
# From repository root
cd monitor

# Initialize (generates keypair)
cargo run -- init

# Run with test directory
VIBETEA_CLAUDE_DIR="$HOME/.vibetea-test-claude" cargo run -- run
```

### Run Server

```bash
# In a separate terminal
cd server
cargo run
```

### Run Client (Development)

```bash
# In a separate terminal
cd client
pnpm dev
```

## Running Tests

### Monitor Tests

```bash
cd monitor
cargo test --test-threads=1
```

### Enhanced Tracking Tests

```bash
cd monitor
cargo test enhanced_tracking --test-threads=1
```

### Server Tests

```bash
cd server
cargo test --test-threads=1
```

### Client Tests

```bash
cd client
pnpm test
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VIBETEA_CLAUDE_DIR` | `~/.claude` | Override Claude directory for testing |
| `VIBETEA_SERVER_URL` | `http://localhost:8080` | Server endpoint |
| `VIBETEA_SOURCE_ID` | hostname | Unique monitor identifier |
| `VIBETEA_BUFFER_SIZE` | `100` | Event buffer size |
| `VIBETEA_DEBOUNCE_MS` | `200` | File change debounce (stats-cache.json) |

## Debugging

### Enable Verbose Logging

```bash
RUST_LOG=debug cargo run -- run
```

### Check File Watching

```bash
# Linux: Check inotify watches
cat /proc/sys/fs/inotify/max_user_watches

# macOS: Check FSEvents
log show --predicate 'subsystem == "com.apple.FSEvents"' --last 5m
```

### Inspect Event Stream

```bash
# Connect to WebSocket and watch events
websocat "ws://localhost:8080/ws?token=YOUR_TOKEN"
```

## Verification Checklist

- [ ] `cargo clippy` passes on monitor crate
- [ ] `cargo fmt --check` passes
- [ ] `cargo test -p vibetea-monitor --test-threads=1` passes
- [ ] Stats-cache.json changes emit events
- [ ] history.jsonl appends emit skill invocation events
- [ ] Todo file changes emit progress events
- [ ] File-history versions >= 2 emit diff events
- [ ] Project session activity detected
- [ ] Client receives and displays new event types

## Next Steps

After setup:

1. Run `/sdd:tasks` to generate implementation tasks
2. Implement tracker modules in priority order:
   1. `stats_tracker.rs` (stats-cache.json)
   2. `skill_tracker.rs` (history.jsonl)
   3. `agent_tracker.rs` (Task tool parsing)
   4. `todo_tracker.rs` (todos/*.json)
   5. `file_history_tracker.rs` (file-history/)
   6. `project_tracker.rs` (projects/)
3. Extend client with new event type handlers
4. Add integration tests for each tracker
