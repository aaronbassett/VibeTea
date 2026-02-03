# Contract: TUI Application Interface

**Module**: `monitor/src/tui/app.rs`
**Date**: 2026-02-03

## Overview

The `App` struct is the main entry point for the TUI, managing the application lifecycle, event handling, and state transitions.

## Interface

### App Struct

```rust
/// Main TUI application
pub struct App {
    state: AppState,
    event_handler: EventHandler,
    tui: Tui,
}

impl App {
    /// Create new application with detected theme and symbols
    pub fn new() -> io::Result<Self>;

    /// Create application with custom configuration
    pub fn with_config(config: AppConfig) -> io::Result<Self>;

    /// Run the application main loop
    pub async fn run(&mut self) -> Result<(), TuiError>;

    /// Check if application should quit
    pub fn should_quit(&self) -> bool;

    /// Get current screen
    pub fn current_screen(&self) -> Screen;

    /// Get immutable reference to state (for rendering)
    pub fn state(&self) -> &AppState;
}
```

### AppConfig

```rust
/// Configuration for TUI application
pub struct AppConfig {
    /// Tick rate for state updates (default: 60ms)
    pub tick_rate: Duration,
    /// Render rate for screen updates (default: 60ms)
    pub render_rate: Duration,
    /// Maximum events in display buffer (default: 1000)
    pub max_events: usize,
    /// Theme override (default: auto-detect)
    pub theme: Option<Theme>,
    /// Symbols override (default: auto-detect)
    pub symbols: Option<Symbols>,
    /// Existing keys path (default: ~/.vibetea)
    pub key_path: PathBuf,
    /// Server URL
    pub server_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(60),
            render_rate: Duration::from_millis(60),
            max_events: 1000,
            theme: None,
            symbols: None,
            key_path: default_key_path(),
            server_url: String::new(),
        }
    }
}
```

### State Management Methods

```rust
impl App {
    /// Handle keyboard input, returns true if quit requested
    pub fn handle_input(&mut self, key: KeyEvent) -> bool;

    /// Handle tick event (timers, animations)
    pub fn tick(&mut self);

    /// Handle terminal resize
    pub fn resize(&mut self, width: u16, height: u16);

    /// Add event to display buffer
    pub fn add_event(&mut self, event: crate::types::Event);

    /// Update connection status
    pub fn set_connection_status(&mut self, status: ConnectionStatus);

    /// Update sender metrics
    pub fn update_metrics(&mut self, metrics: SenderMetrics);

    /// Transition from setup to dashboard
    pub fn complete_setup(&mut self) -> Result<Credentials, SetupError>;
}
```

### Error Types

```rust
/// TUI-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TuiError {
    #[error("terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("terminal too small: {width}x{height} (minimum: 80x24)")]
    TerminalTooSmall { width: u16, height: u16 },

    #[error("setup failed: {0}")]
    Setup(#[from] SetupError),

    #[error("event channel closed")]
    ChannelClosed,
}

/// Setup-specific errors
#[derive(Debug, thiserror::Error)]
pub enum SetupError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("key generation failed: {0}")]
    KeyGeneration(#[from] crate::crypto::CryptoError),

    #[error("cancelled by user")]
    Cancelled,
}
```

## Usage Example

```rust
use vibetea_monitor::tui::{App, AppConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig {
        server_url: "https://vibetea.example.com".to_string(),
        ..Default::default()
    };

    let mut app = App::with_config(config)?;
    app.run().await?;

    Ok(())
}
```

## Integration Points

### With Existing Monitor Components

```rust
// In main.rs run_tui() function
async fn run_tui(config: Config) -> Result<()> {
    let mut app = App::with_config(AppConfig {
        server_url: config.server_url.clone(),
        key_path: config.key_path.clone(),
        ..Default::default()
    })?;

    // Run setup phase
    let credentials = loop {
        if let Some(event) = app.event_handler.next().await {
            match event {
                TuiEvent::Render => app.tui.draw(|f| ui::render(f, app.state()))?,
                TuiEvent::Key(key) => {
                    if app.handle_input(key) {
                        return Ok(()); // User cancelled
                    }
                    if app.current_screen() == Screen::Dashboard {
                        break app.state().dashboard.credentials.clone();
                    }
                }
                _ => {}
            }
        }
    };

    // Initialize sender with credentials
    let crypto = if app.state().setup.key_option == KeyOption::GenerateNew {
        let c = Crypto::generate();
        c.save(&config.key_path)?;
        c
    } else {
        Crypto::load(&config.key_path)?
    };

    let sender = Sender::new(SenderConfig::new(
        config.server_url,
        credentials.session_name,
        config.buffer_size,
    ), crypto);

    // Run main dashboard loop with watcher
    // ... existing watcher integration
}
```

## Lifecycle

1. **Initialization**: Create App, set up panic hook, enter alternate screen
2. **Setup Phase**: Display form, collect configuration, validate inputs
3. **Transition**: Generate/load keys, create sender, start watcher
4. **Dashboard Phase**: Display events, handle input, update stats
5. **Shutdown**: Restore terminal on quit, signal, or panic

## Thread Safety

- `App` is not `Send` or `Sync` (contains terminal handle)
- All TUI operations run on a single async task
- Event channels bridge TUI and background tasks (watcher, sender)
