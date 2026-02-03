# Implementation Plan: Monitor TUI Interface

**Branch**: `003-monitor-tui` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-monitor-tui/spec.md`

## Summary

Transform the VibeTea monitor from a headless CLI into an interactive terminal user interface (TUI) that presents a setup form on first launch, followed by a real-time dashboard displaying server connection status, streaming event log, session credentials, and event statistics. The TUI will use Ratatui for rendering and integrate with the existing async Tokio event loop.

## Technical Context

**Language/Version**: Rust 2021 edition (1.75+)
**Primary Dependencies**: Ratatui 0.29+ (TUI framework), Crossterm 0.28+ (terminal backend), Tokio 1.43 (async runtime)
**Storage**: Filesystem for Ed25519 keypairs (`~/.vibetea/key.priv`, `~/.vibetea/key.pub`)
**Testing**: cargo test with `--test-threads=1` for environment variable isolation
**Target Platform**: Linux (primary), macOS, Windows terminals supporting VT100 escape sequences
**Project Type**: Single Rust binary (monitor component)
**Performance Goals**: 60ms tick rate, <5% CPU idle, handle 100+ events/sec without dropping frames
**Constraints**: Minimum 80x24 terminal, graceful degradation for smaller windows, terminal state restoration on all exit paths
**Scale/Scope**: Single-user local application monitoring Claude Code sessions

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Privacy by Design | ✅ Pass | TUI displays only non-sensitive data already processed by privacy pipeline. No additional logging of sensitive data. |
| II. Unix Philosophy | ✅ Pass | Monitor monitors - TUI adds presentation layer without changing core responsibility. Clear separation of TUI from event processing. |
| III. KISS/YAGNI | ✅ Pass | Using established Ratatui library, not building custom rendering. Features directly match user stories. |
| IV. Event-Driven | ✅ Pass | TUI subscribes to event stream, doesn't poll. Non-blocking input handling. |
| V. Test What Matters | ✅ Pass | Focus on state machine transitions, input validation, credential display accuracy. |
| VI. Fail Fast & Loud | ✅ Pass | Terminal restoration on panic, clear error messages in setup form, connection status prominently displayed. |
| VII. Modularity | ✅ Pass | TUI layer separate from existing sender/crypto/watcher modules. New tui module with clear API boundaries. |

## Project Structure

### Documentation (this feature)

```text
specs/003-monitor-tui/
├── plan.md              # This file
├── research.md          # Phase 0: TUI library research, terminal compatibility
├── data-model.md        # Phase 1: TUI state machines, component props
├── quickstart.md        # Phase 1: Development setup guide
├── contracts/           # Phase 1: Component interfaces
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (repository root)

```text
monitor/
├── src/
│   ├── main.rs              # Updated: TUI mode as default, preserve init/run subcommands
│   ├── lib.rs               # Module exports (add tui module)
│   ├── config.rs            # Existing configuration
│   ├── watcher.rs           # Existing file watcher
│   ├── parser.rs            # Existing JSONL parser
│   ├── privacy.rs           # Existing privacy pipeline
│   ├── crypto.rs            # Existing Ed25519 crypto
│   ├── sender.rs            # Updated: Add observable metrics
│   ├── types.rs             # Existing event types
│   ├── error.rs             # Updated: TUI-specific errors
│   └── tui/                 # NEW: TUI module
│       ├── mod.rs           # TUI module exports
│       ├── app.rs           # Application state machine
│       ├── ui.rs            # Ratatui rendering functions
│       ├── widgets/         # Custom widget implementations
│       │   ├── mod.rs
│       │   ├── logo.rs      # VibeTea ASCII logo
│       │   ├── setup_form.rs # Session name/key setup form
│       │   ├── event_stream.rs # Scrolling event log
│       │   ├── credentials.rs # Session name + public key display
│       │   └── stats_footer.rs # Event statistics
│       ├── input.rs         # Keyboard event handling
│       └── terminal.rs      # Terminal setup/restoration
└── tests/
    ├── tui_state_test.rs    # State machine tests
    └── tui_render_test.rs   # Widget rendering tests (optional)
```

**Structure Decision**: Single Rust binary with new `tui` submodule. TUI code isolated in its own module to preserve existing monitor functionality. The `widgets` subdirectory organizes TUI components for maintainability.

## Complexity Tracking

No constitution violations requiring justification. Design follows existing patterns.

## Learnings from Previous Retros

Based on `specs/001-vibetea/retro/`:

| Learning | Application |
|----------|-------------|
| **Phase 6**: Signal handling requires `tokio::select!` for cross-platform | Apply same pattern for TUI shutdown handling |
| **Phase 6**: `VecDeque` for bounded buffers with FIFO eviction | Use for event stream display buffer |
| **Phase 6**: Simple CLI enum pattern without clap | Extend existing Command enum with TUI mode |
| **Phase 8**: Unicode emoji icons with escape sequences | Use similar patterns for TUI icons/status indicators |
| **Phase 10**: Activity level calculation patterns | Inform event stream scroll/highlight behavior |
| **Phase 10**: Pure render behavior concerns | Keep TUI state calculations deterministic |

## Technical Decisions

### TUI Library Selection: Ratatui

**Decision**: Use Ratatui with Crossterm backend

**Rationale**:
- De facto standard for Rust TUIs, active maintenance (successor to tui-rs)
- Crossterm backend provides cross-platform support (Linux, macOS, Windows)
- Immediate-mode API fits well with async event loop
- Existing VibeTea patterns (event-driven, Tokio async) integrate cleanly

**Alternatives Considered**:
- cursive: Higher abstraction but less control over rendering
- termion: Linux-only, ruled out by Windows compatibility need

### Async Integration

**Decision**: Use non-blocking input polling with `crossterm::event::poll()`

**Rationale**:
- Integrates with existing Tokio event loop
- Allows handling terminal input, network events, and file events in single loop
- Uses `tokio::select!` to multiplex event sources

**Pattern**:
```rust
loop {
    tokio::select! {
        // Check for terminal input with short timeout
        _ = tokio::time::sleep(Duration::from_millis(50)) => {
            if crossterm::event::poll(Duration::from_millis(10))? {
                // Handle input
            }
        }
        // Handle events from watcher/sender
        event = event_rx.recv() => { /* update state */ }
        // Handle shutdown signal
        _ = shutdown_rx.recv() => break,
    }
}
```

### State Machine Design

**Decision**: Two-phase state machine (Setup → Dashboard)

**States**:
1. `Setup` - Form state with current field, session name, key choice
2. `Dashboard` - Running state with connection status, event buffer, stats

**Transitions**:
- `Setup` → `Dashboard`: On form completion
- `Dashboard` → Exit: On quit command or signal

### Terminal Restoration

**Decision**: RAII pattern with panic hook

**Rationale**:
- Terminal struct implements Drop to restore terminal state
- Custom panic hook ensures restoration even on panics
- Uses alternate screen buffer to preserve shell history

**Pattern**:
```rust
struct Terminal {
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
}
```

## Phase 0: Research Topics

1. **Ratatui integration with Tokio**: Confirm non-blocking input patterns work with existing async architecture
2. **Terminal capability detection**: How to detect terminal size and feature support
3. **Unicode support**: Determine safe Unicode characters for cross-terminal compatibility
4. **Color palette**: Select colors that work with both light and dark terminals
5. **Testing strategy**: Approaches for testing TUI state logic without full rendering

## Phase 1: Design Artifacts

### data-model.md Contents

- `SetupFormState`: Current field, session name, key generation choice, validation errors
- `DashboardState`: Connection status, event buffer, statistics, credentials
- `AppState`: Enum wrapping Setup and Dashboard states
- `TuiEvent`: Input events, tick events, shutdown events
- `EventStreamEntry`: Formatted event for display

### contracts/ Contents

- `tui/app.rs` interface: `App::new()`, `App::handle_input()`, `App::tick()`, `App::render()`
- `sender.rs` metrics interface: `SenderMetrics { queued, sent, failed }`
- Widget interfaces for each component

### quickstart.md Contents

- Development setup with `cargo run -p vibetea-monitor`
- TUI mode testing: `cargo run -p vibetea-monitor` (default)
- Headless mode: `cargo run -p vibetea-monitor -- run`
- Key generation: `cargo run -p vibetea-monitor -- init`

## Phase 2: Local Development Environment

### Existing Tooling Status

- ✅ `cargo fmt` - Rust formatting
- ✅ `cargo clippy` - Rust linting
- ✅ `cargo test` - Rust testing
- ✅ lefthook - Pre-commit hooks

### New Dependencies

```toml
[dependencies]
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
```

### Development Commands

```bash
# Run TUI (default mode)
cargo run -p vibetea-monitor

# Run headless (existing behavior)
cargo run -p vibetea-monitor -- run

# Initialize keys
cargo run -p vibetea-monitor -- init

# Run tests
cargo test -p vibetea-monitor
cargo test -p vibetea-monitor tui
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Terminal compatibility issues | Medium | Medium | Test on common terminals (iTerm2, Windows Terminal, Alacritty, tmux/screen) |
| Async integration complexity | Low | High | Follow established Ratatui + Tokio patterns from community examples |
| Performance under high event load | Medium | Medium | Throttle rendering updates, use bounded display buffer |
| Terminal state not restored | Low | High | RAII pattern, panic hook, signal handlers all restore state |

## Next Steps

1. **Phase 0**: Generate `research.md` with Ratatui/Crossterm research
2. **Phase 1**: Generate `data-model.md`, `contracts/`, `quickstart.md`
3. **Phase 2**: Verify development tooling (already complete)
4. Run `/sdd:tasks` to generate implementation tasks
