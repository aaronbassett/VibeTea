# Contract: TUI Widget Interfaces

**Module**: `monitor/src/tui/widgets/`
**Date**: 2026-02-03

## Overview

Widget modules provide reusable rendering functions for TUI components. Each widget takes immutable state and renders to a Ratatui Frame area.

## Logo Widget

**File**: `monitor/src/tui/widgets/logo.rs`

```rust
/// Render VibeTea ASCII logo in header area
///
/// Automatically scales based on available width:
/// - Full logo for width >= 40
/// - Compact text for width < 40
pub fn render_logo(frame: &mut Frame, area: Rect, theme: &Theme);

/// Get logo height for layout calculations
pub fn logo_height(area_width: u16) -> u16;
```

**Logo Content**:
```
 _   _ _ _        _____
| | | (_) |      |_   _|
| | | |_| |__   ___| | ___  __ _
| | | | | '_ \ / _ \ |/ _ \/ _` |
\ \_/ / | |_) |  __/ |  __/ (_| |
 \___/|_|_.__/ \___|_/\___|\__,_|
```

## Setup Form Widget

**File**: `monitor/src/tui/widgets/setup_form.rs`

```rust
/// Render the setup form
///
/// Displays:
/// - Session name input field with default placeholder
/// - Key option selector (Use existing / Generate new)
/// - Submit button
/// - Validation error message (if any)
pub fn render_setup_form(
    frame: &mut Frame,
    area: Rect,
    state: &SetupFormState,
    theme: &Theme,
);

/// Minimum dimensions for setup form
pub const SETUP_FORM_MIN_WIDTH: u16 = 50;
pub const SETUP_FORM_MIN_HEIGHT: u16 = 12;
```

**Layout**:
```
┌─────────────── Setup ───────────────┐
│                                      │
│  Session Name                        │
│  ┌─────────────────────────────────┐│
│  │ my-macbook                      ││
│  └─────────────────────────────────┘│
│  (hostname)                          │
│                                      │
│  Key Option                          │
│  [●] Use existing key                │
│  [ ] Generate new key                │
│                                      │
│  [Submit] ← Enter to continue        │
│                                      │
│  ⚠ Error message here (if any)      │
│                                      │
└──────────────────────────────────────┘
```

## Event Stream Widget

**File**: `monitor/src/tui/widgets/event_stream.rs`

```rust
/// Render scrollable event stream
///
/// Displays events with:
/// - Timestamp (HH:MM:SS)
/// - Event type icon
/// - Summary text (truncated with ellipsis)
///
/// Supports:
/// - Auto-scroll to newest (default)
/// - Manual scrolling (arrow keys pause auto-scroll)
/// - Recent event highlighting (< 5 seconds old)
pub fn render_event_stream(
    frame: &mut Frame,
    area: Rect,
    state: &DashboardState,
    theme: &Theme,
    symbols: &Symbols,
);

/// Calculate visible event range for given area height
pub fn visible_event_range(
    scroll: &ScrollState,
    area_height: u16,
) -> std::ops::Range<usize>;
```

**Event Row Format**:
```
│ HH:MM:SS  🔧  Tool Read completed: main.rs         │
│ HH:MM:SS  🚀  Session started: my-project          │
│ HH:MM:SS  ⚠️   Error: Connection timeout           │
```

**Event Type Icons**:
| Type | Unicode | ASCII | Color |
|------|---------|-------|-------|
| Session | 🚀 | [S] | Magenta |
| Activity | 💬 | [A] | Blue |
| Tool | 🔧 | [T] | Cyan |
| Agent | 🤖 | [G] | Yellow |
| Summary | 📋 | [M] | Green |
| Error | ⚠️ | [!] | Red |

## Credentials Widget

**File**: `monitor/src/tui/widgets/credentials.rs`

```rust
/// Render credentials panel below event stream
///
/// Displays:
/// - Session name
/// - Public key (base64, suitable for copy-paste)
///
/// Handles narrow terminals by wrapping/truncating key display
pub fn render_credentials(
    frame: &mut Frame,
    area: Rect,
    credentials: &Credentials,
    theme: &Theme,
);

/// Minimum width for credentials panel
pub const CREDENTIALS_MIN_WIDTH: u16 = 50;
```

**Layout**:
```
┌────────────── Credentials ──────────────┐
│ Session: my-macbook                      │
│ Public Key: MCowBQYDK2VwAyEA...          │
│                                          │
│ Add this key to VIBETEA_PUBLIC_KEYS on   │
│ the server to authenticate this monitor. │
└──────────────────────────────────────────┘
```

## Stats Footer Widget

**File**: `monitor/src/tui/widgets/stats_footer.rs`

```rust
/// Render statistics footer
///
/// Displays counts for:
/// - Total events processed
/// - Events sent successfully
/// - Events failed (highlighted if > 0)
/// - Events currently queued
pub fn render_stats_footer(
    frame: &mut Frame,
    area: Rect,
    stats: &EventStats,
    theme: &Theme,
);
```

**Layout**:
```
│ Total: 1,234  Sent: 1,200  Failed: 34  Queued: 12 │
```

## Connection Status Widget

**File**: `monitor/src/tui/widgets/connection_status.rs`

```rust
/// Render connection status in header
///
/// Displays status with icon and text:
/// - ● Connected (green)
/// - ○ Disconnected (red)
/// - ◐ Reconnecting... (yellow)
/// - ◔ Connecting... (yellow)
pub fn render_connection_status(
    frame: &mut Frame,
    area: Rect,
    status: ConnectionStatus,
    theme: &Theme,
    symbols: &Symbols,
);
```

## Header Widget

**File**: `monitor/src/tui/widgets/header.rs`

```rust
/// Render combined header with logo and status
///
/// Layout adapts to width:
/// - Wide (>= 80): Logo left, status right
/// - Narrow (< 80): Status only
pub fn render_header(
    frame: &mut Frame,
    area: Rect,
    status: ConnectionStatus,
    theme: &Theme,
    symbols: &Symbols,
);

/// Calculate header height for layout
pub fn header_height(area_width: u16) -> u16;
```

**Wide Layout**:
```
┌──────────────────────────────────────────────────────────────────────────────┐
│  _   _ _ _        _____                                    ● Connected      │
│ | | | (_) |      |_   _|                                                    │
│ | | | |_| |__   ___| | ___  __ _                                           │
│ | | | | | '_ \ / _ \ |/ _ \/ _` |                                          │
│ \ \_/ / | |_) |  __/ |  __/ (_| |                                          │
│  \___/|_|_.__/ \___|_/\___|\__,_|                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Narrow Layout**:
```
┌──────────────────────────────────────┐
│ VibeTea               ● Connected    │
└──────────────────────────────────────┘
```

## Size Warning Widget

**File**: `monitor/src/tui/widgets/size_warning.rs`

```rust
/// Render terminal size warning overlay
///
/// Displayed when terminal is below minimum 80x24
pub fn render_size_warning(
    frame: &mut Frame,
    area: Rect,
    current_size: (u16, u16),
    theme: &Theme,
);
```

**Layout**:
```
┌─────────── Terminal Too Small ───────────┐
│                                          │
│  Current size: 60x20                     │
│  Minimum size: 80x24                     │
│                                          │
│  Please resize your terminal to continue │
│                                          │
└──────────────────────────────────────────┘
```

## Common Widget Patterns

### Area Subdivision

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Split area vertically into header, body, footer
fn split_dashboard(area: Rect) -> (Rect, Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height(area.width)),  // Header
            Constraint::Min(10),                             // Event stream
            Constraint::Length(5),                           // Credentials
            Constraint::Length(1),                           // Stats footer
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2], chunks[3])
}
```

### Styled Text Builder

```rust
use ratatui::text::{Line, Span};

fn status_line(status: ConnectionStatus, theme: &Theme, symbols: &Symbols) -> Line<'static> {
    let (symbol, text, style) = match status {
        ConnectionStatus::Connected => (
            symbols.connected,
            "Connected",
            theme.status_connected,
        ),
        ConnectionStatus::Disconnected => (
            symbols.disconnected,
            "Disconnected",
            theme.status_disconnected,
        ),
        // ...
    };

    Line::from(vec![
        Span::styled(format!("{} ", symbol), style),
        Span::styled(text, style),
    ])
}
```
