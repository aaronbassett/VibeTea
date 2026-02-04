# Research: Monitor TUI Interface

**Feature**: 003-monitor-tui
**Date**: 2026-02-03

## TUI Library Selection

### Decision: Ratatui 0.29+ with Crossterm backend

**Rationale**:
- De facto standard for Rust TUIs, successor to tui-rs with active maintenance
- Crossterm backend provides cross-platform support (Linux, macOS, Windows)
- Immediate-mode API integrates naturally with async event loops
- Built-in responsive layout system adapts to terminal size changes
- Extensive documentation and community examples

**Alternatives Considered**:

| Library | Pros | Cons | Decision |
|---------|------|------|----------|
| cursive | Higher-level API, widget library | Less control over rendering, heavier | Rejected |
| termion | Lightweight, pure Rust | Linux-only, no Windows support | Rejected (Windows needed) |
| tui-rs | Predecessor, stable | Unmaintained, superseded by Ratatui | Rejected |

**Dependencies to Add**:

```toml
[dependencies]
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
```

## Async Integration Pattern

### Decision: EventStream with tokio::select!

The `crossterm::event::EventStream` provides a futures-compatible stream that integrates cleanly with Tokio:

```rust
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEventKind};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;

pub enum TuiEvent {
    Tick,
    Render,
    Key(crossterm::event::KeyEvent),
    Resize(u16, u16),
    WatchEvent(crate::types::Event),
    Metrics(SenderMetrics),
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<TuiEvent>,
    _task: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, render_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        let task = tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);
            let mut render_interval = tokio::time::interval(render_rate);

            loop {
                tokio::select! {
                    _ = tick_interval.tick() => {
                        let _ = tx.send(TuiEvent::Tick);
                    }
                    _ = render_interval.tick() => {
                        let _ = tx.send(TuiEvent::Render);
                    }
                    maybe_event = reader.next().fuse() => {
                        if let Some(Ok(evt)) = maybe_event {
                            match evt {
                                CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => {
                                    let _ = tx.send(TuiEvent::Key(key));
                                }
                                CrosstermEvent::Resize(w, h) => {
                                    let _ = tx.send(TuiEvent::Resize(w, h));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });

        Self { rx, _task: task }
    }

    pub async fn next(&mut self) -> Option<TuiEvent> {
        self.rx.recv().await
    }
}
```

**Key Points**:
- Windows sends both Press and Release key events; filter for `KeyEventKind::Press`
- Use `fuse()` on stream to handle completion properly
- Tick events drive state updates (60ms default per NFR-001)
- Render events trigger terminal draws (can be separate from tick rate)

## Terminal Capability Detection

### Size Detection

```rust
use crossterm::terminal::size;

pub fn check_minimum_size() -> Result<(), String> {
    let (cols, rows) = size().map_err(|e| format!("Cannot query terminal size: {}", e))?;

    const MIN_COLS: u16 = 80;
    const MIN_ROWS: u16 = 24;

    if cols < MIN_COLS || rows < MIN_ROWS {
        return Err(format!(
            "Terminal too small: {}x{} (minimum: {}x{}). Please resize and try again.",
            cols, rows, MIN_COLS, MIN_ROWS
        ));
    }
    Ok(())
}
```

### Resize Handling

Ratatui automatically queries terminal size on each `draw()` call. Handle resize events to update app state if needed:

```rust
TuiEvent::Resize(w, h) => {
    if w < 80 || h < 24 {
        app_state.show_size_warning = true;
    } else {
        app_state.show_size_warning = false;
    }
}
```

## Unicode and Character Support

### Safe Box-Drawing Characters

Use Ratatui's built-in border types for maximum compatibility:

| Border Type | Characters | Compatibility |
|-------------|------------|---------------|
| `BorderType::Plain` | `┌ ─ ┐ │ └ ┘` | Excellent (WGL4) |
| `BorderType::Rounded` | `╭ ─ ╮ │ ╰ ╯` | Good (Unicode 1.1) |
| `BorderType::Double` | `╔ ═ ╗ ║ ╚ ╝` | Excellent (WGL4) |

**Recommendation**: Use `Plain` or `Rounded` borders.

### Status Indicator Symbols

```rust
pub struct Symbols {
    pub connected: &'static str,
    pub disconnected: &'static str,
    pub reconnecting: &'static str,
    pub success: &'static str,
    pub failure: &'static str,
    pub arrow: &'static str,
}

pub const UNICODE_SYMBOLS: Symbols = Symbols {
    connected: "●",      // U+25CF BLACK CIRCLE
    disconnected: "○",   // U+25CB WHITE CIRCLE
    reconnecting: "◐",   // U+25D0 CIRCLE WITH LEFT HALF BLACK
    success: "✓",        // U+2713 CHECK MARK
    failure: "✗",        // U+2717 BALLOT X
    arrow: "→",          // U+2192 RIGHTWARDS ARROW
};

pub const ASCII_SYMBOLS: Symbols = Symbols {
    connected: "[*]",
    disconnected: "[ ]",
    reconnecting: "[~]",
    success: "[+]",
    failure: "[x]",
    arrow: "->",
};

pub fn detect_unicode_support() -> bool {
    std::env::var("TERM")
        .map(|t| !t.contains("linux") && !t.contains("vt100"))
        .unwrap_or(true)
}
```

## Color Palette

### Decision: ANSI Named Colors

Use ANSI named colors that adapt to user's terminal theme:

```rust
use ratatui::style::{Color, Style, Modifier};

pub struct Theme {
    pub status_connected: Style,
    pub status_disconnected: Style,
    pub status_reconnecting: Style,
    pub event_timestamp: Style,
    pub event_type: Style,
    pub error_count: Style,
    pub border: Style,
    pub text_primary: Style,
    pub text_secondary: Style,
    pub input_focused: Style,
    pub input_unfocused: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            status_connected: Style::default().fg(Color::Green),
            status_disconnected: Style::default().fg(Color::Red),
            status_reconnecting: Style::default().fg(Color::Yellow),
            event_timestamp: Style::default().fg(Color::DarkGray),
            event_type: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            error_count: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::DarkGray),
            text_primary: Style::default(),
            text_secondary: Style::default().fg(Color::DarkGray),
            input_focused: Style::default().fg(Color::Cyan),
            input_unfocused: Style::default().fg(Color::Gray),
        }
    }
}

impl Theme {
    pub fn monochrome() -> Self {
        Self {
            status_connected: Style::default().add_modifier(Modifier::BOLD),
            status_disconnected: Style::default().add_modifier(Modifier::DIM),
            status_reconnecting: Style::default().add_modifier(Modifier::ITALIC),
            event_timestamp: Style::default().add_modifier(Modifier::DIM),
            event_type: Style::default().add_modifier(Modifier::BOLD),
            error_count: Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            border: Style::default(),
            text_primary: Style::default(),
            text_secondary: Style::default().add_modifier(Modifier::DIM),
            input_focused: Style::default().add_modifier(Modifier::BOLD),
            input_unfocused: Style::default().add_modifier(Modifier::DIM),
        }
    }

    pub fn new() -> Self {
        if std::env::var("NO_COLOR").is_ok() {
            Self::monochrome()
        } else {
            Self::default()
        }
    }
}
```

### Color-Blind Safety (NFR-006)

Combine symbols with colors for redundant information:

```rust
fn render_status(status: ConnectionStatus, symbols: &Symbols) -> Span {
    match status {
        ConnectionStatus::Connected => Span::styled(
            format!("{} Connected", symbols.connected),
            theme.status_connected
        ),
        ConnectionStatus::Disconnected => Span::styled(
            format!("{} Disconnected", symbols.disconnected),
            theme.status_disconnected
        ),
        ConnectionStatus::Reconnecting => Span::styled(
            format!("{} Reconnecting...", symbols.reconnecting),
            theme.status_reconnecting
        ),
    }
}
```

## Terminal Restoration

### Decision: RAII with Panic Hook

Per FR-019, terminal must be restored on all exit paths:

```rust
use std::io::{self, stdout, Stdout};
use std::panic;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub fn new() -> io::Result<Self> {
        Self::install_panic_hook();

        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_terminal();
            original_hook(panic_info);
        }));
    }

    fn restore_terminal() -> io::Result<()> {
        disable_raw_mode()?;
        execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = Self::restore_terminal() {
            eprintln!("Failed to restore terminal: {}", e);
        }
    }
}
```

**Benefits**:
- RAII ensures cleanup on normal exit
- Panic hook ensures cleanup on panic
- Signal handlers (existing `wait_for_shutdown()`) trigger normal loop exit, then Drop runs

## Testing Strategy

### State Logic Testing

Separate state from rendering for unit testability:

```rust
// state.rs - pure logic, no TUI dependencies
pub struct AppState {
    pub current_screen: Screen,
    pub setup_form: SetupFormState,
    pub dashboard: DashboardState,
    pub should_quit: bool,
}

impl AppState {
    pub fn handle_key(&mut self, key: KeyEvent) -> StateAction {
        // Pure state transitions
    }

    pub fn add_event(&mut self, event: EventEntry) {
        // Buffer management
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};

    #[test]
    fn test_quit_on_q() {
        let mut state = AppState::default();
        state.current_screen = Screen::Dashboard;

        let key = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::empty());
        state.handle_key(key);

        assert!(state.should_quit);
    }

    #[test]
    fn test_session_name_validation() {
        let state = SetupFormState::default();

        assert!(validate_session_name("valid-name_123").is_ok());
        assert!(validate_session_name("").is_err());
        assert!(validate_session_name(&"a".repeat(100)).is_err());
        assert!(validate_session_name("invalid name!").is_err());
    }
}
```

### Visual Testing (Optional)

Use TestBackend for snapshot testing:

```rust
#[cfg(test)]
mod render_tests {
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn test_dashboard_renders() {
        let state = DashboardState::test_fixture();
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| render_dashboard(f, &state)).unwrap();

        // Assert on buffer contents or use snapshot testing
        let buffer = terminal.backend().buffer();
        assert!(buffer.get(0, 0).symbol() != " "); // Header not empty
    }
}
```

## tmux/screen Compatibility

### Key Points

1. **Resize events**: tmux forwards resize events; handled normally
2. **Colors**: ANSI named colors work in all multiplexers
3. **Escape key delay**: Users may need `set -sg escape-time 10` in tmux.conf

### Detection and Guidance

```rust
fn is_multiplexer() -> bool {
    std::env::var("TMUX").is_ok() || std::env::var("STY").is_ok()
}

fn log_multiplexer_tip() {
    if is_multiplexer() {
        tracing::info!("Running in terminal multiplexer. If you experience input lag, add 'set -sg escape-time 10' to ~/.tmux.conf");
    }
}
```

## VibeTea ASCII Logo

### Logo Design

```rust
pub const VIBETEA_LOGO: &[&str] = &[
    r" _   _ _ _        _____            ",
    r"| | | (_) |      |_   _|           ",
    r"| | | |_| |__   ___| | ___  __ _   ",
    r"| | | | | '_ \ / _ \ |/ _ \/ _` |  ",
    r"\ \_/ / | |_) |  __/ |  __/ (_| |  ",
    r" \___/|_|_.__/ \___|_/\___|\__,_|  ",
];

pub const VIBETEA_LOGO_SMALL: &[&str] = &[
    r"VibeTea",
];

pub fn get_logo(width: u16) -> &'static [&'static str] {
    if width >= 40 {
        VIBETEA_LOGO
    } else {
        VIBETEA_LOGO_SMALL
    }
}
```

## Event Stream Display Buffer

### Decision: VecDeque with FIFO Eviction

Per FR-024, maintain bounded display buffer:

```rust
use std::collections::VecDeque;

const MAX_DISPLAY_EVENTS: usize = 1000;

pub struct EventBuffer {
    events: VecDeque<DisplayEvent>,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_DISPLAY_EVENTS),
        }
    }

    pub fn push(&mut self, event: DisplayEvent) {
        if self.events.len() >= MAX_DISPLAY_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn iter(&self) -> impl Iterator<Item = &DisplayEvent> {
        self.events.iter()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }
}
```

## Input Handling

### Form Navigation Keys

```rust
pub fn handle_setup_input(state: &mut SetupFormState, key: KeyEvent) -> FormAction {
    match key.code {
        KeyCode::Tab | KeyCode::Down => FormAction::NextField,
        KeyCode::BackTab | KeyCode::Up => FormAction::PrevField,
        KeyCode::Enter => FormAction::Submit,
        KeyCode::Esc => FormAction::Cancel,
        KeyCode::Char(c) => {
            if state.focused_field == SetupField::SessionName {
                state.session_name.push(c);
                FormAction::Update
            } else {
                FormAction::None
            }
        }
        KeyCode::Backspace => {
            if state.focused_field == SetupField::SessionName {
                state.session_name.pop();
                FormAction::Update
            } else {
                FormAction::None
            }
        }
        KeyCode::Left | KeyCode::Right => {
            if state.focused_field == SetupField::KeyOption {
                state.key_option = state.key_option.toggle();
                FormAction::Update
            } else {
                FormAction::None
            }
        }
        _ => FormAction::None,
    }
}
```

### Dashboard Navigation Keys

```rust
pub fn handle_dashboard_input(state: &mut DashboardState, key: KeyEvent) -> DashAction {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => DashAction::Quit,
        KeyCode::Up | KeyCode::Char('k') => {
            state.scroll_up();
            state.auto_scroll = false;
            DashAction::Update
        }
        KeyCode::Down | KeyCode::Char('j') => {
            state.scroll_down();
            DashAction::Update
        }
        KeyCode::PageUp => {
            state.scroll_page_up();
            state.auto_scroll = false;
            DashAction::Update
        }
        KeyCode::PageDown => {
            state.scroll_page_down();
            DashAction::Update
        }
        KeyCode::Home | KeyCode::Char('g') => {
            state.scroll_to_top();
            state.auto_scroll = false;
            DashAction::Update
        }
        KeyCode::End | KeyCode::Char('G') => {
            state.scroll_to_bottom();
            state.auto_scroll = true;
            DashAction::Update
        }
        _ => DashAction::None,
    }
}
```

## Sender Metrics Integration

### FR-025: Observable Metrics

Add metrics struct to sender module:

```rust
// In sender.rs
pub struct SenderMetrics {
    pub queued: usize,
    pub sent: u64,
    pub failed: u64,
}

impl Sender {
    pub fn metrics(&self) -> SenderMetrics {
        SenderMetrics {
            queued: self.buffer.len(),
            sent: self.sent_count,
            failed: self.failed_count,
        }
    }
}
```

## Summary

| Topic | Decision |
|-------|----------|
| TUI Library | Ratatui 0.29+ with Crossterm backend |
| Async Pattern | EventStream with tokio::select! |
| Colors | ANSI named colors with NO_COLOR support |
| Symbols | Unicode with ASCII fallback |
| Border Style | Plain or Rounded for compatibility |
| Terminal Restoration | RAII + panic hook pattern |
| Event Buffer | VecDeque with 1000-event limit |
| Testing | Separated state logic for unit testing |
