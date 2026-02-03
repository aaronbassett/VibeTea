# Data Model: Monitor TUI Interface

**Feature**: 003-monitor-tui
**Date**: 2026-02-03

## Overview

The TUI data model consists of two primary state machines (Setup and Dashboard) connected by a top-level application state. The design separates pure state logic from rendering for testability.

## Core State Types

### AppState

Top-level application state machine.

```rust
/// Application state machine
pub struct AppState {
    /// Current screen being displayed
    pub screen: Screen,
    /// Setup form state (populated when screen == Setup)
    pub setup: SetupFormState,
    /// Dashboard state (populated when screen == Dashboard)
    pub dashboard: DashboardState,
    /// Flag indicating user requested exit
    pub should_quit: bool,
    /// Theme configuration
    pub theme: Theme,
    /// Symbol set (unicode or ascii)
    pub symbols: Symbols,
}

/// Screen enum for state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Setup,
    Dashboard,
}
```

### SetupFormState

State for the initial configuration form (User Stories 1, 6, 7).

```rust
/// Setup form state
#[derive(Debug, Clone, Default)]
pub struct SetupFormState {
    /// Current field with focus
    pub focused_field: SetupField,
    /// Session name input value
    pub session_name: String,
    /// Key generation option
    pub key_option: KeyOption,
    /// Validation error message (if any)
    pub validation_error: Option<String>,
    /// Whether keys already exist
    pub keys_exist: bool,
    /// Default session name (hostname)
    pub default_session_name: String,
}

/// Fields in the setup form
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupField {
    #[default]
    SessionName,
    KeyOption,
    Submit,
}

/// Key generation options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyOption {
    #[default]
    UseExisting,
    GenerateNew,
}

impl KeyOption {
    pub fn toggle(&self) -> Self {
        match self {
            KeyOption::UseExisting => KeyOption::GenerateNew,
            KeyOption::GenerateNew => KeyOption::UseExisting,
        }
    }
}
```

### DashboardState

State for the main monitoring dashboard (User Stories 2, 3, 4, 5, 8).

```rust
/// Dashboard state
#[derive(Debug, Clone)]
pub struct DashboardState {
    /// Server connection status
    pub connection_status: ConnectionStatus,
    /// Event stream display buffer
    pub event_buffer: EventBuffer,
    /// Event statistics
    pub stats: EventStats,
    /// Session credentials
    pub credentials: Credentials,
    /// Scroll state for event stream
    pub scroll: ScrollState,
    /// Last resize dimensions (for graceful degradation)
    pub terminal_size: (u16, u16),
}

/// Connection status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

/// Event statistics (FR-011)
#[derive(Debug, Clone, Copy, Default)]
pub struct EventStats {
    /// Total events processed since startup
    pub total: u64,
    /// Events successfully sent to server
    pub sent: u64,
    /// Events that failed to send
    pub failed: u64,
    /// Events currently queued in buffer
    pub queued: usize,
}

/// Session credentials for display (FR-009, FR-010)
#[derive(Debug, Clone, Default)]
pub struct Credentials {
    /// Session name (source_id)
    pub session_name: String,
    /// Base64-encoded Ed25519 public key
    pub public_key: String,
}

/// Scroll state for event stream
#[derive(Debug, Clone, Default)]
pub struct ScrollState {
    /// Current scroll offset (0 = newest at bottom visible)
    pub offset: usize,
    /// Whether auto-scroll is enabled (scroll to newest)
    pub auto_scroll: bool,
    /// Total events in buffer (cached for scroll calculations)
    pub total_events: usize,
    /// Visible area height (set from terminal size)
    pub visible_height: usize,
}
```

## Event Types

### TuiEvent

Internal events for the TUI event loop.

```rust
/// Events that drive the TUI loop
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Periodic tick for animations/timers (60ms default)
    Tick,
    /// Trigger a render cycle
    Render,
    /// Terminal input event
    Key(crossterm::event::KeyEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// File watcher event (forwarded from existing watcher)
    WatchEvent(crate::types::Event),
    /// Sender metrics update
    MetricsUpdate(EventStats),
    /// Connection status change
    ConnectionChange(ConnectionStatus),
}
```

### DisplayEvent

Formatted event for stream display (FR-008, FR-022, FR-023).

```rust
/// Event formatted for display in the stream
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    /// Event ID (for keying/deduplication)
    pub id: String,
    /// Formatted timestamp (HH:MM:SS)
    pub timestamp: String,
    /// Event type for icon/color selection
    pub event_type: DisplayEventType,
    /// Summary text (truncated to fit width)
    pub summary: String,
    /// Age in seconds (for highlighting recent events)
    pub age_secs: u64,
}

/// Event type categories for display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEventType {
    Session,
    Activity,
    Tool,
    Agent,
    Summary,
    Error,
}

impl From<crate::types::EventType> for DisplayEventType {
    fn from(et: crate::types::EventType) -> Self {
        match et {
            crate::types::EventType::Session => Self::Session,
            crate::types::EventType::Activity => Self::Activity,
            crate::types::EventType::Tool => Self::Tool,
            crate::types::EventType::Agent => Self::Agent,
            crate::types::EventType::Summary => Self::Summary,
            crate::types::EventType::Error => Self::Error,
        }
    }
}
```

## Buffers

### EventBuffer

Bounded FIFO buffer for display events (FR-024).

```rust
use std::collections::VecDeque;

/// Maximum events to retain in display buffer (configurable)
pub const MAX_DISPLAY_EVENTS: usize = 1000;

/// Bounded FIFO buffer for display events
#[derive(Debug, Clone, Default)]
pub struct EventBuffer {
    events: VecDeque<DisplayEvent>,
    max_size: usize,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_DISPLAY_EVENTS),
            max_size: MAX_DISPLAY_EVENTS,
        }
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Add event, evicting oldest if full
    pub fn push(&mut self, event: DisplayEvent) {
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Get event at index (0 = oldest)
    pub fn get(&self, index: usize) -> Option<&DisplayEvent> {
        self.events.get(index)
    }

    /// Iterate over events (oldest first)
    pub fn iter(&self) -> impl Iterator<Item = &DisplayEvent> {
        self.events.iter()
    }

    /// Iterate in reverse (newest first)
    pub fn iter_rev(&self) -> impl Iterator<Item = &DisplayEvent> {
        self.events.iter().rev()
    }

    /// Number of events in buffer
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
    }
}
```

## Theme Configuration

### Theme

Color and style configuration.

```rust
use ratatui::style::{Color, Modifier, Style};

/// Theme configuration for the TUI
#[derive(Debug, Clone)]
pub struct Theme {
    // Status indicators
    pub status_connected: Style,
    pub status_disconnected: Style,
    pub status_connecting: Style,
    pub status_reconnecting: Style,

    // Event stream
    pub event_timestamp: Style,
    pub event_type_session: Style,
    pub event_type_activity: Style,
    pub event_type_tool: Style,
    pub event_type_agent: Style,
    pub event_type_summary: Style,
    pub event_type_error: Style,
    pub event_recent: Style,

    // Statistics
    pub stat_total: Style,
    pub stat_sent: Style,
    pub stat_failed: Style,
    pub stat_queued: Style,

    // Form
    pub input_focused: Style,
    pub input_unfocused: Style,
    pub input_error: Style,
    pub label: Style,

    // Layout
    pub border: Style,
    pub border_focused: Style,
    pub title: Style,
    pub text_primary: Style,
    pub text_secondary: Style,
    pub text_muted: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            // Status indicators (NFR-006: color-blind safe with symbols)
            status_connected: Style::default().fg(Color::Green),
            status_disconnected: Style::default().fg(Color::Red),
            status_connecting: Style::default().fg(Color::Yellow),
            status_reconnecting: Style::default().fg(Color::Yellow),

            // Event stream
            event_timestamp: Style::default().fg(Color::DarkGray),
            event_type_session: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            event_type_activity: Style::default().fg(Color::Blue),
            event_type_tool: Style::default().fg(Color::Cyan),
            event_type_agent: Style::default().fg(Color::Yellow),
            event_type_summary: Style::default().fg(Color::Green),
            event_type_error: Style::default().fg(Color::Red),
            event_recent: Style::default().add_modifier(Modifier::BOLD),

            // Statistics (FR-012: failed visually distinguished)
            stat_total: Style::default().fg(Color::White),
            stat_sent: Style::default().fg(Color::Green),
            stat_failed: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            stat_queued: Style::default().fg(Color::Yellow),

            // Form
            input_focused: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            input_unfocused: Style::default().fg(Color::Gray),
            input_error: Style::default().fg(Color::Red),
            label: Style::default().fg(Color::White),

            // Layout
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default().fg(Color::Cyan),
            title: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            text_primary: Style::default(),
            text_secondary: Style::default().fg(Color::Gray),
            text_muted: Style::default().fg(Color::DarkGray),
        }
    }
}

impl Theme {
    /// Create monochrome theme for NO_COLOR support
    pub fn monochrome() -> Self {
        Self {
            status_connected: Style::default().add_modifier(Modifier::BOLD),
            status_disconnected: Style::default().add_modifier(Modifier::DIM),
            status_connecting: Style::default().add_modifier(Modifier::ITALIC),
            status_reconnecting: Style::default().add_modifier(Modifier::ITALIC),

            event_timestamp: Style::default().add_modifier(Modifier::DIM),
            event_type_session: Style::default().add_modifier(Modifier::BOLD),
            event_type_activity: Style::default(),
            event_type_tool: Style::default(),
            event_type_agent: Style::default().add_modifier(Modifier::ITALIC),
            event_type_summary: Style::default(),
            event_type_error: Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            event_recent: Style::default().add_modifier(Modifier::BOLD),

            stat_total: Style::default(),
            stat_sent: Style::default(),
            stat_failed: Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            stat_queued: Style::default(),

            input_focused: Style::default().add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            input_unfocused: Style::default().add_modifier(Modifier::DIM),
            input_error: Style::default().add_modifier(Modifier::BOLD),
            label: Style::default(),

            border: Style::default(),
            border_focused: Style::default().add_modifier(Modifier::BOLD),
            title: Style::default().add_modifier(Modifier::BOLD),
            text_primary: Style::default(),
            text_secondary: Style::default().add_modifier(Modifier::DIM),
            text_muted: Style::default().add_modifier(Modifier::DIM),
        }
    }

    /// Create theme based on environment
    pub fn from_env() -> Self {
        if std::env::var("NO_COLOR").is_ok() {
            Self::monochrome()
        } else {
            Self::default()
        }
    }
}
```

### Symbols

Unicode/ASCII symbol sets.

```rust
/// Symbol set for status indicators
#[derive(Debug, Clone, Copy)]
pub struct Symbols {
    pub connected: &'static str,
    pub disconnected: &'static str,
    pub connecting: &'static str,
    pub reconnecting: &'static str,
    pub success: &'static str,
    pub failure: &'static str,
    pub arrow: &'static str,
    pub bullet: &'static str,
}

pub const UNICODE_SYMBOLS: Symbols = Symbols {
    connected: "●",
    disconnected: "○",
    connecting: "◔",
    reconnecting: "◐",
    success: "✓",
    failure: "✗",
    arrow: "→",
    bullet: "•",
};

pub const ASCII_SYMBOLS: Symbols = Symbols {
    connected: "[*]",
    disconnected: "[ ]",
    connecting: "[.]",
    reconnecting: "[~]",
    success: "[+]",
    failure: "[x]",
    arrow: "->",
    bullet: "*",
};

impl Symbols {
    /// Detect and return appropriate symbol set
    pub fn detect() -> Self {
        if std::env::var("TERM")
            .map(|t| t.contains("linux") || t.contains("vt100"))
            .unwrap_or(false)
        {
            ASCII_SYMBOLS
        } else {
            UNICODE_SYMBOLS
        }
    }
}
```

## Validation Rules

### Session Name Validation (FR-014, FR-026)

```rust
/// Session name validation result
pub type ValidationResult = Result<(), String>;

/// Validate session name per FR-026
pub fn validate_session_name(name: &str) -> ValidationResult {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Session name cannot be empty".to_string());
    }

    if trimmed.len() > 64 {
        return Err(format!("Session name too long ({}/64 characters)", trimmed.len()));
    }

    // Allow alphanumeric, hyphens, underscores only
    if !trimmed.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err("Session name can only contain letters, numbers, hyphens, and underscores".to_string());
    }

    // Don't start with hyphen or underscore
    if trimmed.starts_with('-') || trimmed.starts_with('_') {
        return Err("Session name cannot start with hyphen or underscore".to_string());
    }

    Ok(())
}
```

## State Transitions

### Setup Form Flow

```
┌─────────────────┐
│   Initialize    │
│  (load config)  │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  SessionName    │<─────┐
│    Field        │      │ Tab/Arrow
└────────┬────────┘      │
         │ Tab/Enter     │
         v               │
┌─────────────────┐      │
│   KeyOption     │──────┘
│    Field        │
└────────┬────────┘
         │ Tab/Enter
         v
┌─────────────────┐
│     Submit      │
│    Button       │
└────────┬────────┘
         │ Enter (if valid)
         v
┌─────────────────┐
│   Dashboard     │
│    Screen       │
└─────────────────┘
```

### Dashboard Event Flow

```
         ┌─────────────────────────────────────────┐
         │                                         │
         v                                         │
┌─────────────────┐                               │
│  File Watch     │ ─────> Process Event ─────────┤
│   Event         │                               │
└─────────────────┘                               │
                                                  │
┌─────────────────┐                               │
│  Connection     │ ─────> Update Status ─────────┤
│   Change        │                               │
└─────────────────┘                               │
                                                  │
┌─────────────────┐                               │
│   Metrics       │ ─────> Update Stats ──────────┤
│   Update        │                               │
└─────────────────┘                               │
                                                  │
┌─────────────────┐                               │
│  Key Input      │ ─────> Handle Action ─────────┤
│   Event         │                               │
└─────────────────┘                               │
                                                  │
┌─────────────────┐                               │
│    Render       │ <─────────────────────────────┘
│    Cycle        │
└─────────────────┘
```

## Related Documents

- [spec.md](./spec.md) - Feature specification
- [plan.md](./plan.md) - Implementation plan
- [research.md](./research.md) - Technology research
- [contracts/](./contracts/) - API contracts
