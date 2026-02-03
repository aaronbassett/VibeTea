//! Application state and event management for the VibeTea Monitor TUI.
//!
//! This module contains the core application state, event types, and business logic
//! that drive the TUI. The main types are:
//!
//! - [`AppState`]: Application state machine managing screens and quit state
//! - [`Screen`]: Current screen being displayed (Setup or Dashboard)
//! - [`TuiEvent`]: Events that drive the TUI event loop
//! - [`EventHandler`]: Async event loop using `tokio::select!` to multiplex event sources
//! - [`EventStats`]: Metrics for sender event throughput (placeholder)
//! - [`ConnectionStatus`]: WebSocket connection state (placeholder)
//!
//! # Architecture
//!
//! The TUI uses an event-driven architecture where all state changes are triggered
//! by [`TuiEvent`] variants. The [`EventHandler`] runs an async loop that:
//!
//! 1. Polls for terminal input (keyboard, resize) with short timeouts
//! 2. Generates periodic tick events for animations and timers
//! 3. Listens for shutdown signals to terminate gracefully
//!
//! Events are sent to the main application via an MPSC channel, where they are
//! processed to update state followed by render cycles.
//!
//! # Example
//!
//! ```ignore
//! use tokio::sync::mpsc;
//! use vibetea_monitor::tui::app::EventHandler;
//!
//! let (event_tx, mut event_rx) = mpsc::channel(100);
//! let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
//!
//! // Spawn the event handler
//! let handler = EventHandler::new(event_tx, shutdown_rx);
//! tokio::spawn(handler.run());
//!
//! // Process events in the main loop
//! while let Some(event) = event_rx.recv().await {
//!     // Handle the event...
//! }
//! ```

use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use ratatui::style::{Color, Modifier, Style};
use tokio::sync::{mpsc, oneshot};

use crate::types::Event;

// =============================================================================
// Screen and Application State Types
// =============================================================================

/// Current screen being displayed in the TUI.
///
/// The monitor TUI operates as a simple state machine with two main screens:
///
/// - **Setup**: Initial configuration screen where users can specify server URL,
///   session path, and other connection parameters
/// - **Dashboard**: Main monitoring view showing real-time session events,
///   connection status, and metrics
///
/// # Default
///
/// The default screen is [`Screen::Setup`], as users typically need to configure
/// the monitor before viewing the dashboard.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::Screen;
///
/// let screen = Screen::default();
/// assert_eq!(screen, Screen::Setup);
///
/// let screen = Screen::Dashboard;
/// assert_ne!(screen, Screen::Setup);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Setup screen for initial configuration.
    ///
    /// Displayed when the application starts, allowing users to configure
    /// server URL, session path, and other connection parameters.
    #[default]
    Setup,

    /// Main dashboard screen for monitoring.
    ///
    /// Shows real-time session events, connection status, and metrics
    /// once the monitor is configured and connected.
    Dashboard,
}

/// Form field that can receive focus in the setup screen.
///
/// The setup form has three focusable fields that the user can navigate between
/// using Tab/Shift+Tab or arrow keys. The focus state determines which field
/// receives keyboard input and is visually highlighted.
///
/// # Field Order
///
/// The natural tab order is:
/// 1. [`SetupField::SessionName`] - Session name text input
/// 2. [`SetupField::KeyOption`] - Key generation option selector
/// 3. [`SetupField::Submit`] - Submit button
///
/// # Default
///
/// The default focus is [`SetupField::SessionName`], as this is the first
/// interactive element users encounter in the form.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::SetupField;
///
/// let field = SetupField::default();
/// assert_eq!(field, SetupField::SessionName);
///
/// // All fields are distinct
/// assert_ne!(SetupField::SessionName, SetupField::KeyOption);
/// assert_ne!(SetupField::KeyOption, SetupField::Submit);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SetupField {
    /// Session name text input field.
    ///
    /// When focused, keyboard input is captured as text for the session name.
    /// This is the first field in tab order and the default focus.
    #[default]
    SessionName,

    /// Key generation option selector.
    ///
    /// When focused, arrow keys or space toggle between key options.
    /// This is the second field in tab order.
    KeyOption,

    /// Submit button.
    ///
    /// When focused, Enter or space activates the form submission.
    /// This is the last field in tab order.
    Submit,
}

/// Key generation option for the setup form.
///
/// Determines whether to use existing keys from `~/.vibetea` or generate a new
/// keypair for this session. The default behavior follows FR-004: if existing
/// keys are detected, default to [`KeyOption::UseExisting`]; otherwise default
/// to [`KeyOption::GenerateNew`].
///
/// # Toggle Behavior
///
/// The [`KeyOption::toggle()`] method allows cycling between options, which is
/// useful for keyboard-based selection in the TUI.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::KeyOption;
///
/// // Default is GenerateNew
/// let option = KeyOption::default();
/// assert_eq!(option, KeyOption::GenerateNew);
///
/// // Toggle switches between options
/// let toggled = option.toggle();
/// assert_eq!(toggled, KeyOption::UseExisting);
///
/// // Toggle again returns to original
/// let toggled_again = toggled.toggle();
/// assert_eq!(toggled_again, KeyOption::GenerateNew);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KeyOption {
    /// Use existing keys from `~/.vibetea`.
    ///
    /// This option is available when the monitor detects existing key files
    /// in the VibeTea configuration directory. Using existing keys maintains
    /// session continuity and avoids unnecessary key regeneration.
    UseExisting,

    /// Generate a new keypair.
    ///
    /// Creates a fresh Ed25519 keypair for this session. This is the default
    /// option when no existing keys are found (FR-004).
    #[default]
    GenerateNew,
}

impl KeyOption {
    /// Toggles between key options.
    ///
    /// Switches [`KeyOption::UseExisting`] to [`KeyOption::GenerateNew`] and
    /// vice versa. This is useful for keyboard-based selection where users
    /// press space or arrow keys to change the option.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::KeyOption;
    ///
    /// let option = KeyOption::GenerateNew;
    /// assert_eq!(option.toggle(), KeyOption::UseExisting);
    ///
    /// let option = KeyOption::UseExisting;
    /// assert_eq!(option.toggle(), KeyOption::GenerateNew);
    /// ```
    #[must_use]
    pub fn toggle(self) -> Self {
        match self {
            KeyOption::UseExisting => KeyOption::GenerateNew,
            KeyOption::GenerateNew => KeyOption::UseExisting,
        }
    }
}

/// State for the setup form screen.
///
/// Contains form field values, validation state, and focus tracking for the
/// initial configuration form. This form collects the session name and key
/// generation preference before transitioning to the dashboard.
///
/// # Form Fields
///
/// - **Session Name**: Unique identifier for this monitoring session (FR-003).
///   Defaults to the system hostname. Limited to 64 characters, alphanumeric
///   plus `-` and `_` only (FR-026).
/// - **Key Option**: Whether to use existing keys or generate new ones (FR-004).
///   Defaults to "Use existing" if keys are found, otherwise "Generate new".
///
/// # Validation
///
/// The `session_name_error` field contains any validation error message for the
/// session name. Validation is performed on input and before form submission.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{SetupFormState, SetupField, KeyOption};
///
/// // Create default state
/// let state = SetupFormState::default();
/// assert!(state.session_name.is_empty());
/// assert_eq!(state.focused_field, SetupField::SessionName);
/// assert_eq!(state.key_option, KeyOption::GenerateNew);
///
/// // Create with specific values
/// let state = SetupFormState {
///     session_name: "my-session".to_string(),
///     session_name_error: None,
///     key_option: KeyOption::UseExisting,
///     focused_field: SetupField::Submit,
///     existing_keys_found: true,
/// };
/// assert_eq!(state.session_name, "my-session");
/// assert!(state.existing_keys_found);
/// ```
#[derive(Debug, Clone, Default)]
pub struct SetupFormState {
    /// Current session name input value.
    ///
    /// Defaults to empty string initially; should be populated with hostname
    /// during initialization (FR-003). Limited to 64 characters maximum,
    /// containing only alphanumeric characters, hyphens, and underscores (FR-026).
    pub session_name: String,

    /// Validation error message for the session name, if any.
    ///
    /// Set when the session name fails validation (e.g., contains invalid
    /// characters, exceeds 64 characters, or is empty). `None` indicates
    /// the current value is valid or hasn't been validated yet.
    pub session_name_error: Option<String>,

    /// Selected key generation option.
    ///
    /// Determines whether to use existing keys from `~/.vibetea` or generate
    /// a new keypair. Per FR-004, this defaults to [`KeyOption::UseExisting`]
    /// if existing keys are detected, otherwise [`KeyOption::GenerateNew`].
    pub key_option: KeyOption,

    /// Currently focused form field.
    ///
    /// Determines which field receives keyboard input and is visually
    /// highlighted. Defaults to [`SetupField::SessionName`].
    pub focused_field: SetupField,

    /// Whether existing keys were detected in `~/.vibetea`.
    ///
    /// Used to determine the default value for `key_option` (FR-004) and
    /// whether to show the "Use existing" option as available.
    pub existing_keys_found: bool,
}

/// State for the dashboard screen.
///
/// Contains all state needed to render and update the dashboard view,
/// including event streams, metrics, and UI state.
///
/// # Note
///
/// This is a placeholder type. The full implementation will be added in a later
/// task with fields for event history, scroll position, metrics, and panel state.
#[derive(Debug, Clone, Default)]
pub struct DashboardState {
    // Placeholder - fields will be added in future tasks
}

/// Theme configuration for the TUI.
///
/// Defines colors and styles used throughout the interface for consistent
/// visual presentation. The theme covers four main areas:
///
/// - **Status indicators**: Colors for connection states (connected, disconnected, etc.)
/// - **Event stream**: Styles for different event types and timestamps
/// - **Statistics**: Styles for metrics display (sent, failed, queued counts)
/// - **Layout and form**: Borders, titles, inputs, and text styles
///
/// # Color-Blind Safety
///
/// The theme uses symbols in addition to colors to ensure accessibility for
/// color-blind users (NFR-006). Status indicators always have accompanying
/// text or symbols that don't rely solely on color.
///
/// # NO_COLOR Support
///
/// For environments where colors should be disabled (per the `NO_COLOR` standard),
/// use [`Theme::monochrome()`] or [`Theme::from_env()`] which will automatically
/// detect the `NO_COLOR` environment variable.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::Theme;
///
/// // Default colorful theme
/// let theme = Theme::default();
///
/// // Monochrome theme for NO_COLOR support
/// let mono_theme = Theme::monochrome();
///
/// // Auto-detect based on environment
/// let env_theme = Theme::from_env();
/// ```
#[derive(Debug, Clone)]
pub struct Theme {
    // Status indicators
    /// Style for connected status (default: green).
    pub status_connected: Style,
    /// Style for disconnected status (default: red).
    pub status_disconnected: Style,
    /// Style for connecting status (default: yellow).
    pub status_connecting: Style,
    /// Style for reconnecting status (default: yellow).
    pub status_reconnecting: Style,

    // Event stream
    /// Style for event timestamps (default: dark gray).
    pub event_timestamp: Style,
    /// Style for session events (default: magenta bold).
    pub event_type_session: Style,
    /// Style for activity events (default: blue).
    pub event_type_activity: Style,
    /// Style for tool events (default: cyan).
    pub event_type_tool: Style,
    /// Style for agent events (default: yellow).
    pub event_type_agent: Style,
    /// Style for summary events (default: green).
    pub event_type_summary: Style,
    /// Style for error events (default: red).
    pub event_type_error: Style,
    /// Style for recent events (default: bold).
    pub event_recent: Style,

    // Statistics
    /// Style for total event count (default: white).
    pub stat_total: Style,
    /// Style for sent event count (default: green).
    pub stat_sent: Style,
    /// Style for failed event count (default: red bold).
    pub stat_failed: Style,
    /// Style for queued event count (default: yellow).
    pub stat_queued: Style,

    // Form
    /// Style for focused input fields (default: cyan bold).
    pub input_focused: Style,
    /// Style for unfocused input fields (default: gray).
    pub input_unfocused: Style,
    /// Style for input error states (default: red).
    pub input_error: Style,
    /// Style for form labels (default: white).
    pub label: Style,

    // Layout
    /// Style for unfocused borders (default: dark gray).
    pub border: Style,
    /// Style for focused borders (default: cyan).
    pub border_focused: Style,
    /// Style for titles (default: white bold).
    pub title: Style,
    /// Style for primary text (default: reset/terminal default).
    pub text_primary: Style,
    /// Style for secondary text (default: gray).
    pub text_secondary: Style,
    /// Style for muted/deemphasized text (default: dark gray).
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
            event_type_session: Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
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
            input_focused: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            input_unfocused: Style::default().fg(Color::Gray),
            input_error: Style::default().fg(Color::Red),
            label: Style::default().fg(Color::White),

            // Layout
            border: Style::default().fg(Color::DarkGray),
            border_focused: Style::default().fg(Color::Cyan),
            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
            text_primary: Style::default(),
            text_secondary: Style::default().fg(Color::Gray),
            text_muted: Style::default().fg(Color::DarkGray),
        }
    }
}

impl Theme {
    /// Creates a monochrome theme for `NO_COLOR` support.
    ///
    /// This theme uses only modifiers (bold, dim, italic, underlined) without
    /// any color codes. It complies with the [NO_COLOR standard](https://no-color.org/)
    /// for terminals where color output is disabled.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::Theme;
    ///
    /// let theme = Theme::monochrome();
    /// // All styles use modifiers instead of colors
    /// ```
    #[must_use]
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

    /// Creates a theme based on the environment.
    ///
    /// Checks the `NO_COLOR` environment variable and returns:
    /// - [`Theme::monochrome()`] if `NO_COLOR` is set (to any value)
    /// - [`Theme::default()`] otherwise
    ///
    /// This follows the [NO_COLOR standard](https://no-color.org/) for respecting
    /// user preferences regarding terminal colors.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::Theme;
    ///
    /// // Returns monochrome if NO_COLOR is set, colorful otherwise
    /// let theme = Theme::from_env();
    /// ```
    #[must_use]
    pub fn from_env() -> Self {
        if std::env::var("NO_COLOR").is_ok() {
            Self::monochrome()
        } else {
            Self::default()
        }
    }
}

/// Symbol set for the TUI (unicode or ASCII).
///
/// Provides a consistent set of symbols for rendering UI elements.
/// Unicode symbols provide a richer visual experience on modern terminals,
/// while ASCII symbols ensure compatibility with limited terminals.
///
/// # Symbol Sets
///
/// Two predefined symbol sets are available:
///
/// - [`UNICODE_SYMBOLS`]: Rich unicode characters for modern terminals
/// - [`ASCII_SYMBOLS`]: Plain ASCII for maximum compatibility
///
/// Use [`Symbols::detect()`] to automatically select the appropriate symbol
/// set based on the terminal environment.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{Symbols, UNICODE_SYMBOLS, ASCII_SYMBOLS};
///
/// // Use unicode symbols explicitly
/// let symbols = UNICODE_SYMBOLS;
/// assert_eq!(symbols.connected, "●");
///
/// // Use ASCII symbols for compatibility
/// let symbols = ASCII_SYMBOLS;
/// assert_eq!(symbols.connected, "[*]");
///
/// // Auto-detect based on terminal
/// let symbols = Symbols::detect();
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Symbols {
    /// Symbol for connected status.
    pub connected: &'static str,
    /// Symbol for disconnected status.
    pub disconnected: &'static str,
    /// Symbol for connecting status.
    pub connecting: &'static str,
    /// Symbol for reconnecting status.
    pub reconnecting: &'static str,
    /// Symbol for success/completion.
    pub success: &'static str,
    /// Symbol for failure/error.
    pub failure: &'static str,
    /// Arrow symbol for navigation/direction.
    pub arrow: &'static str,
    /// Bullet point symbol for lists.
    pub bullet: &'static str,
}

/// Unicode symbol set for modern terminals.
///
/// This symbol set uses rich unicode characters that render nicely on most
/// modern terminal emulators. It provides a more visually appealing experience
/// but may not display correctly on limited terminals (e.g., Linux console,
/// VT100 emulators).
///
/// # Symbols
///
/// | Symbol | Character | Description |
/// |--------|-----------|-------------|
/// | `connected` | ● | Filled circle for active connection |
/// | `disconnected` | ○ | Empty circle for no connection |
/// | `connecting` | ◔ | Quarter-filled circle for pending |
/// | `reconnecting` | ◐ | Half-filled circle for retry |
/// | `success` | ✓ | Check mark for success |
/// | `failure` | ✗ | X mark for failure |
/// | `arrow` | → | Right arrow for navigation |
/// | `bullet` | • | Bullet point for lists |
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

/// ASCII symbol set for maximum compatibility.
///
/// This symbol set uses only plain ASCII characters, ensuring compatibility
/// with all terminals including the Linux console, VT100 emulators, and
/// environments with limited unicode support.
///
/// # Symbols
///
/// | Symbol | Characters | Description |
/// |--------|------------|-------------|
/// | `connected` | `[*]` | Asterisk in brackets for active |
/// | `disconnected` | `[ ]` | Empty brackets for no connection |
/// | `connecting` | `[.]` | Dot in brackets for pending |
/// | `reconnecting` | `[~]` | Tilde in brackets for retry |
/// | `success` | `[+]` | Plus in brackets for success |
/// | `failure` | `[x]` | X in brackets for failure |
/// | `arrow` | `->` | ASCII arrow for navigation |
/// | `bullet` | `*` | Asterisk for lists |
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
    /// Detects and returns the appropriate symbol set for the current terminal.
    ///
    /// This method checks the `TERM` environment variable to determine if the
    /// terminal supports unicode. If the terminal is identified as a limited
    /// environment (e.g., `linux` console, `vt100`), ASCII symbols are returned.
    /// Otherwise, unicode symbols are used.
    ///
    /// # Detection Logic
    ///
    /// Returns [`ASCII_SYMBOLS`] if:
    /// - `TERM` contains "linux" (Linux console)
    /// - `TERM` contains "vt100" (VT100 emulator)
    ///
    /// Returns [`UNICODE_SYMBOLS`] otherwise, including when:
    /// - `TERM` is not set
    /// - `TERM` contains other values (xterm, screen, tmux, etc.)
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::Symbols;
    ///
    /// let symbols = Symbols::detect();
    /// // Will be unicode on most modern terminals
    /// ```
    #[must_use]
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

impl Default for Symbols {
    /// Returns the auto-detected symbol set.
    ///
    /// This is equivalent to calling [`Symbols::detect()`].
    fn default() -> Self {
        Self::detect()
    }
}

/// Application state machine for the VibeTea Monitor TUI.
///
/// Manages the current screen, form state, dashboard state, and application-wide
/// settings like theme and symbol set. This is the central state container that
/// gets updated in response to [`TuiEvent`]s and drives the rendering logic.
///
/// # State Machine
///
/// The application operates as a simple state machine:
///
/// ```text
/// +-------+     user confirms     +-----------+
/// | Setup | ------------------->  | Dashboard |
/// +-------+                       +-----------+
///     ^                                |
///     |       user goes back           |
///     +--------------------------------+
/// ```
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::AppState;
///
/// let mut state = AppState::new();
/// assert!(state.is_setup());
/// assert!(!state.should_quit());
///
/// state.quit();
/// assert!(state.should_quit());
/// ```
#[derive(Debug, Clone, Default)]
pub struct AppState {
    /// Current screen being displayed.
    pub screen: Screen,

    /// Setup form state (populated when screen == Setup).
    pub setup: SetupFormState,

    /// Dashboard state (populated when screen == Dashboard).
    pub dashboard: DashboardState,

    /// Flag indicating user requested exit.
    pub should_quit: bool,

    /// Theme configuration.
    pub theme: Theme,

    /// Symbol set (unicode or ASCII).
    pub symbols: Symbols,
}

impl AppState {
    /// Creates a new `AppState` with default values.
    ///
    /// The application starts on the Setup screen with default theme and symbols.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::AppState;
    ///
    /// let state = AppState::new();
    /// assert!(state.is_setup());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the current screen is Setup.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{AppState, Screen};
    ///
    /// let mut state = AppState::new();
    /// assert!(state.is_setup());
    ///
    /// state.screen = Screen::Dashboard;
    /// assert!(!state.is_setup());
    /// ```
    #[must_use]
    pub fn is_setup(&self) -> bool {
        self.screen == Screen::Setup
    }

    /// Returns `true` if the current screen is Dashboard.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{AppState, Screen};
    ///
    /// let mut state = AppState::new();
    /// assert!(!state.is_dashboard());
    ///
    /// state.screen = Screen::Dashboard;
    /// assert!(state.is_dashboard());
    /// ```
    #[must_use]
    pub fn is_dashboard(&self) -> bool {
        self.screen == Screen::Dashboard
    }

    /// Returns `true` if the application should quit.
    ///
    /// The main event loop should check this flag to determine when to exit.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::AppState;
    ///
    /// let state = AppState::new();
    /// assert!(!state.should_quit());
    /// ```
    #[must_use]
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Signals that the application should quit.
    ///
    /// Sets the `should_quit` flag to `true`. The main event loop should
    /// check this flag and initiate graceful shutdown.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::AppState;
    ///
    /// let mut state = AppState::new();
    /// assert!(!state.should_quit());
    ///
    /// state.quit();
    /// assert!(state.should_quit());
    /// ```
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

// =============================================================================
// Event Types and Statistics
// =============================================================================

/// Statistics for event sending throughput.
///
/// Tracks metrics about events being sent to the VibeTea server,
/// including counts, rates, and timing information.
///
/// # Note
///
/// This is a placeholder type. The full implementation will be added
/// in a later task with fields for event counts, throughput rates,
/// success/failure tracking, and timing statistics.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EventStats {
    /// Total number of events sent successfully.
    pub events_sent: u64,
    /// Total number of events that failed to send.
    pub events_failed: u64,
}

/// Connection status for the WebSocket link to the VibeTea server.
///
/// Represents the current state of the connection to the server,
/// allowing the TUI to display appropriate status indicators.
///
/// # Note
///
/// This is a placeholder type. Additional variants and associated data
/// (such as error details, retry counts, or latency measurements) may be
/// added in later tasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    /// Not connected to the server.
    #[default]
    Disconnected,
    /// Currently attempting to establish a connection.
    Connecting,
    /// Successfully connected and ready to send events.
    Connected,
    /// Connection attempt failed or connection was lost.
    Error,
}

/// Events that drive the TUI event loop.
///
/// The TUI operates on an event-driven model where all state changes
/// are triggered by incoming events. This enum defines all possible
/// event types that the main loop can process.
///
/// # Event Sources
///
/// - **Tick**: Generated by an internal timer for animations and periodic updates
/// - **Render**: Triggered when a redraw is needed
/// - **Key**: Forwarded from terminal input handling
/// - **Resize**: Forwarded from terminal resize signals
/// - **WatchEvent**: Forwarded from the file watcher monitoring session logs
/// - **MetricsUpdate**: Received from the sender component with throughput stats
/// - **ConnectionChange**: Received when WebSocket connection state changes
///
/// # Examples
///
/// ```ignore
/// use vibetea_monitor::tui::app::TuiEvent;
/// use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
///
/// // Handle a key press event
/// let event = TuiEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
///
/// // Handle a resize event
/// let event = TuiEvent::Resize(120, 40);
/// ```
#[derive(Debug, Clone)]
pub enum TuiEvent {
    /// Periodic tick for animations and timers.
    ///
    /// Generated at a fixed interval (default 60ms) to drive animations,
    /// update timers, and perform periodic state checks.
    Tick,

    /// Trigger a render cycle.
    ///
    /// Indicates that the UI should be redrawn. This may be triggered
    /// after state changes or when the terminal needs refreshing.
    Render,

    /// Terminal input event.
    ///
    /// Represents a key press or key combination from the user.
    /// The TUI processes these to handle navigation, commands, and input.
    Key(KeyEvent),

    /// Terminal resize event.
    ///
    /// Contains the new terminal dimensions (columns, rows) after a resize.
    /// The TUI should recalculate layouts and redraw when this is received.
    Resize(u16, u16),

    /// File watcher event.
    ///
    /// Forwarded from the existing file watcher when new session events
    /// are detected in the monitored log files.
    WatchEvent(Event),

    /// Sender metrics update.
    ///
    /// Contains updated statistics about event sending throughput,
    /// success rates, and other sender-related metrics.
    MetricsUpdate(EventStats),

    /// Connection status change.
    ///
    /// Indicates that the WebSocket connection state has changed,
    /// allowing the TUI to update status indicators accordingly.
    ConnectionChange(ConnectionStatus),
}

/// Default tick rate for the event handler (60ms = ~16 FPS).
///
/// This value provides smooth animations while balancing CPU usage.
/// A faster tick rate (lower value) provides smoother animations but
/// consumes more CPU cycles polling for events.
pub const DEFAULT_TICK_RATE_MS: u64 = 60;

/// Default poll timeout for checking terminal input (10ms).
///
/// This short timeout allows the event loop to remain responsive to
/// shutdown signals while efficiently batching terminal events.
const DEFAULT_POLL_TIMEOUT_MS: u64 = 10;

/// Handles terminal input and generates periodic tick events.
///
/// The `EventHandler` runs an async event loop that:
///
/// 1. Polls for terminal input (key presses, resize events) with a short timeout
/// 2. Generates [`TuiEvent::Tick`] events at a configurable interval
/// 3. Sends all events to the main application via an MPSC channel
/// 4. Terminates gracefully when a shutdown signal is received
///
/// # Architecture
///
/// The handler uses `tokio::select!` to multiplex three event sources:
///
/// - **Tick interval**: A tokio interval that fires at the configured tick rate
/// - **Terminal polling**: Non-blocking checks for crossterm events
/// - **Shutdown signal**: A oneshot channel that triggers graceful termination
///
/// # Thread Safety
///
/// The `EventHandler` is designed to run in its own tokio task. It uses
/// `tokio::task::spawn_blocking` for terminal polling to avoid blocking
/// the async runtime with synchronous crossterm calls.
///
/// # Example
///
/// ```ignore
/// use tokio::sync::{mpsc, oneshot};
/// use vibetea_monitor::tui::app::EventHandler;
///
/// async fn run_tui() {
///     let (event_tx, mut event_rx) = mpsc::channel(100);
///     let (shutdown_tx, shutdown_rx) = oneshot::channel();
///
///     // Spawn the event handler in a separate task
///     let handler = EventHandler::new(event_tx, shutdown_rx);
///     let event_task = tokio::spawn(handler.run());
///
///     // Main application loop
///     while let Some(event) = event_rx.recv().await {
///         match event {
///             TuiEvent::Tick => { /* update timers, animations */ }
///             TuiEvent::Key(key) => { /* handle input */ }
///             TuiEvent::Resize(w, h) => { /* recalculate layout */ }
///             _ => {}
///         }
///     }
///
///     // Trigger shutdown
///     let _ = shutdown_tx.send(());
///     event_task.await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct EventHandler {
    /// Channel sender for dispatching events to the main application.
    event_tx: mpsc::Sender<TuiEvent>,
    /// Receiver for the shutdown signal.
    shutdown_rx: oneshot::Receiver<()>,
    /// Tick rate in milliseconds.
    tick_rate: Duration,
}

impl EventHandler {
    /// Creates a new `EventHandler` with the default tick rate.
    ///
    /// # Arguments
    ///
    /// * `event_tx` - Channel sender for dispatching [`TuiEvent`]s to the application
    /// * `shutdown_rx` - Oneshot receiver that signals when the handler should terminate
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tokio::sync::{mpsc, oneshot};
    /// use vibetea_monitor::tui::app::EventHandler;
    ///
    /// let (event_tx, event_rx) = mpsc::channel(100);
    /// let (shutdown_tx, shutdown_rx) = oneshot::channel();
    ///
    /// let handler = EventHandler::new(event_tx, shutdown_rx);
    /// ```
    pub fn new(event_tx: mpsc::Sender<TuiEvent>, shutdown_rx: oneshot::Receiver<()>) -> Self {
        Self {
            event_tx,
            shutdown_rx,
            tick_rate: Duration::from_millis(DEFAULT_TICK_RATE_MS),
        }
    }

    /// Creates a new `EventHandler` with a custom tick rate.
    ///
    /// # Arguments
    ///
    /// * `event_tx` - Channel sender for dispatching [`TuiEvent`]s to the application
    /// * `shutdown_rx` - Oneshot receiver that signals when the handler should terminate
    /// * `tick_rate` - Custom tick rate for generating [`TuiEvent::Tick`] events
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::time::Duration;
    /// use tokio::sync::{mpsc, oneshot};
    /// use vibetea_monitor::tui::app::EventHandler;
    ///
    /// let (event_tx, event_rx) = mpsc::channel(100);
    /// let (shutdown_tx, shutdown_rx) = oneshot::channel();
    ///
    /// // Create a handler with 30 FPS (~33ms tick rate)
    /// let handler = EventHandler::with_tick_rate(event_tx, shutdown_rx, Duration::from_millis(33));
    /// ```
    pub fn with_tick_rate(
        event_tx: mpsc::Sender<TuiEvent>,
        shutdown_rx: oneshot::Receiver<()>,
        tick_rate: Duration,
    ) -> Self {
        Self {
            event_tx,
            shutdown_rx,
            tick_rate,
        }
    }

    /// Returns the configured tick rate.
    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }

    /// Runs the event loop until a shutdown signal is received.
    ///
    /// This method consumes the `EventHandler` and runs until either:
    /// - A shutdown signal is received via the `shutdown_rx` channel
    /// - The event sender is closed (all receivers dropped)
    ///
    /// # Event Generation
    ///
    /// The loop generates events in the following priority order:
    ///
    /// 1. **Shutdown check**: If a shutdown signal is received, exit immediately
    /// 2. **Tick events**: Generate [`TuiEvent::Tick`] at the configured interval
    /// 3. **Terminal events**: Poll for key presses and resize events
    ///
    /// # Errors
    ///
    /// Returns `Ok(())` on graceful shutdown. Returns an error if:
    /// - Terminal event polling fails (I/O error)
    /// - The event channel is closed unexpectedly
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tokio::sync::{mpsc, oneshot};
    /// use vibetea_monitor::tui::app::EventHandler;
    ///
    /// async fn example() {
    ///     let (event_tx, mut event_rx) = mpsc::channel(100);
    ///     let (shutdown_tx, shutdown_rx) = oneshot::channel();
    ///
    ///     let handler = EventHandler::new(event_tx, shutdown_rx);
    ///
    ///     // Run in a spawned task
    ///     tokio::spawn(async move {
    ///         if let Err(e) = handler.run().await {
    ///             eprintln!("Event handler error: {}", e);
    ///         }
    ///     });
    ///
    ///     // Process events...
    ///     while let Some(event) = event_rx.recv().await {
    ///         // handle event
    ///     }
    ///
    ///     // Signal shutdown
    ///     let _ = shutdown_tx.send(());
    /// }
    /// ```
    pub async fn run(mut self) -> std::io::Result<()> {
        let mut tick_interval = tokio::time::interval(self.tick_rate);
        // Use burst mode to avoid tick accumulation if processing falls behind
        tick_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Burst);

        // Consume the first tick immediately (interval ticks immediately on creation)
        tick_interval.tick().await;

        loop {
            // Use biased selection to ensure shutdown is always checked first
            tokio::select! {
                biased;

                // Highest priority: check for shutdown signal
                _ = &mut self.shutdown_rx => {
                    tracing::debug!("EventHandler received shutdown signal");
                    break;
                }

                // Generate tick events at the configured interval
                _ = tick_interval.tick() => {
                    if self.event_tx.send(TuiEvent::Tick).await.is_err() {
                        // Receiver dropped, exit gracefully
                        tracing::debug!("Event receiver dropped, exiting event loop");
                        break;
                    }
                }

                // Poll for terminal events with a short sleep to prevent busy-waiting
                // We use spawn_blocking to avoid blocking the async runtime
                result = async {
                    // Small delay before polling to allow tick events to be processed
                    tokio::time::sleep(Duration::from_millis(DEFAULT_POLL_TIMEOUT_MS)).await;
                    tokio::task::spawn_blocking(|| {
                        Self::poll_terminal_event(Duration::from_millis(DEFAULT_POLL_TIMEOUT_MS))
                    }).await
                } => {
                    match result {
                        Ok(Some(event)) => {
                            if self.event_tx.send(event).await.is_err() {
                                tracing::debug!("Event receiver dropped, exiting event loop");
                                break;
                            }
                        }
                        Ok(None) => {
                            // No event available within timeout, continue
                        }
                        Err(join_error) => {
                            tracing::error!("spawn_blocking task panicked: {}", join_error);
                            return Err(std::io::Error::other(
                                "Terminal polling task panicked",
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Polls for a terminal event with the specified timeout.
    ///
    /// This is a synchronous function designed to be called via `spawn_blocking`
    /// to avoid blocking the async runtime.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(event))` if a terminal event was received
    /// - `Ok(None)` if the timeout elapsed with no event, or if polling failed
    ///   due to non-terminal environment (e.g., in tests or CI)
    fn poll_terminal_event(timeout: Duration) -> Option<TuiEvent> {
        // In non-terminal environments (CI, tests), poll() may fail.
        // We treat this as "no event" rather than propagating the error.
        match event::poll(timeout) {
            Ok(true) => {
                // Event is available, try to read it
                match event::read() {
                    Ok(crossterm_event) => Self::convert_crossterm_event(crossterm_event),
                    Err(e) => {
                        tracing::trace!("Failed to read terminal event: {}", e);
                        None
                    }
                }
            }
            Ok(false) => {
                // No event within timeout
                None
            }
            Err(e) => {
                // Polling failed (likely no terminal available)
                tracing::trace!("Failed to poll terminal: {}", e);
                None
            }
        }
    }

    /// Converts a crossterm event to a TuiEvent.
    ///
    /// # Returns
    ///
    /// - `Some(TuiEvent)` for supported event types (Key, Resize)
    /// - `None` for unsupported event types (Mouse, Focus, Paste)
    fn convert_crossterm_event(event: CrosstermEvent) -> Option<TuiEvent> {
        match event {
            CrosstermEvent::Key(key_event) => Some(TuiEvent::Key(key_event)),
            CrosstermEvent::Resize(cols, rows) => Some(TuiEvent::Resize(cols, rows)),
            // Mouse events, focus events, and paste events are not currently handled
            CrosstermEvent::Mouse(_) => None,
            CrosstermEvent::FocusGained | CrosstermEvent::FocusLost => None,
            CrosstermEvent::Paste(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn event_stats_default() {
        let stats = EventStats::default();
        assert_eq!(stats.events_sent, 0);
        assert_eq!(stats.events_failed, 0);
    }

    #[test]
    fn connection_status_default_is_disconnected() {
        let status = ConnectionStatus::default();
        assert_eq!(status, ConnectionStatus::Disconnected);
    }

    #[test]
    fn tui_event_tick_is_debug() {
        let event = TuiEvent::Tick;
        let debug_str = format!("{:?}", event);
        assert!(debug_str.contains("Tick"));
    }

    #[test]
    fn tui_event_render_is_clone() {
        let event = TuiEvent::Render;
        let cloned = event.clone();
        assert!(matches!(cloned, TuiEvent::Render));
    }

    #[test]
    fn tui_event_key_wraps_key_event() {
        let key_event = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        let event = TuiEvent::Key(key_event);

        if let TuiEvent::Key(k) = event {
            assert_eq!(k.code, KeyCode::Char('q'));
            assert_eq!(k.modifiers, KeyModifiers::NONE);
        } else {
            panic!("Expected TuiEvent::Key variant");
        }
    }

    #[test]
    fn tui_event_resize_contains_dimensions() {
        let event = TuiEvent::Resize(120, 40);

        if let TuiEvent::Resize(cols, rows) = event {
            assert_eq!(cols, 120);
            assert_eq!(rows, 40);
        } else {
            panic!("Expected TuiEvent::Resize variant");
        }
    }

    #[test]
    fn tui_event_metrics_update_contains_stats() {
        let stats = EventStats {
            events_sent: 100,
            events_failed: 5,
        };
        let event = TuiEvent::MetricsUpdate(stats.clone());

        if let TuiEvent::MetricsUpdate(s) = event {
            assert_eq!(s.events_sent, 100);
            assert_eq!(s.events_failed, 5);
        } else {
            panic!("Expected TuiEvent::MetricsUpdate variant");
        }
    }

    #[test]
    fn tui_event_connection_change_contains_status() {
        let event = TuiEvent::ConnectionChange(ConnectionStatus::Connected);

        if let TuiEvent::ConnectionChange(status) = event {
            assert_eq!(status, ConnectionStatus::Connected);
        } else {
            panic!("Expected TuiEvent::ConnectionChange variant");
        }
    }

    // EventHandler tests

    #[test]
    fn event_handler_new_has_default_tick_rate() {
        let (event_tx, _event_rx) = mpsc::channel(10);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        let handler = EventHandler::new(event_tx, shutdown_rx);
        assert_eq!(
            handler.tick_rate(),
            Duration::from_millis(DEFAULT_TICK_RATE_MS)
        );
    }

    #[test]
    fn event_handler_with_custom_tick_rate() {
        let (event_tx, _event_rx) = mpsc::channel(10);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        let custom_rate = Duration::from_millis(33);
        let handler = EventHandler::with_tick_rate(event_tx, shutdown_rx, custom_rate);
        assert_eq!(handler.tick_rate(), custom_rate);
    }

    #[test]
    fn event_handler_is_debug() {
        let (event_tx, _event_rx) = mpsc::channel(10);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel();

        let handler = EventHandler::new(event_tx, shutdown_rx);
        let debug_str = format!("{:?}", handler);
        assert!(debug_str.contains("EventHandler"));
    }

    #[test]
    fn convert_crossterm_key_event() {
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        let crossterm_event = CrosstermEvent::Key(key_event);

        let result = EventHandler::convert_crossterm_event(crossterm_event);
        assert!(result.is_some());

        if let Some(TuiEvent::Key(k)) = result {
            assert_eq!(k.code, KeyCode::Char('a'));
            assert_eq!(k.modifiers, KeyModifiers::CONTROL);
        } else {
            panic!("Expected TuiEvent::Key variant");
        }
    }

    #[test]
    fn convert_crossterm_resize_event() {
        let crossterm_event = CrosstermEvent::Resize(80, 24);

        let result = EventHandler::convert_crossterm_event(crossterm_event);
        assert!(result.is_some());

        if let Some(TuiEvent::Resize(cols, rows)) = result {
            assert_eq!(cols, 80);
            assert_eq!(rows, 24);
        } else {
            panic!("Expected TuiEvent::Resize variant");
        }
    }

    #[test]
    fn convert_crossterm_mouse_event_returns_none() {
        use crossterm::event::{MouseEvent, MouseEventKind};

        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };
        let crossterm_event = CrosstermEvent::Mouse(mouse_event);

        let result = EventHandler::convert_crossterm_event(crossterm_event);
        assert!(result.is_none());
    }

    #[test]
    fn convert_crossterm_focus_events_return_none() {
        let focus_gained = CrosstermEvent::FocusGained;
        let focus_lost = CrosstermEvent::FocusLost;

        assert!(EventHandler::convert_crossterm_event(focus_gained).is_none());
        assert!(EventHandler::convert_crossterm_event(focus_lost).is_none());
    }

    #[test]
    fn convert_crossterm_paste_event_returns_none() {
        let crossterm_event = CrosstermEvent::Paste("clipboard content".to_string());

        let result = EventHandler::convert_crossterm_event(crossterm_event);
        assert!(result.is_none());
    }

    #[test]
    fn default_tick_rate_is_60ms() {
        assert_eq!(DEFAULT_TICK_RATE_MS, 60);
    }

    #[tokio::test]
    async fn event_handler_stops_on_shutdown_signal() {
        let (event_tx, _event_rx) = mpsc::channel(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let handler = EventHandler::with_tick_rate(
            event_tx,
            shutdown_rx,
            Duration::from_millis(500), // Long tick rate to ensure we test shutdown
        );

        // Spawn the handler
        let handle = tokio::spawn(handler.run());

        // Give it a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Send shutdown signal
        let _ = shutdown_tx.send(());

        // Handler should complete within a reasonable timeout
        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Handler should complete within timeout");

        let inner_result = result.unwrap();
        assert!(inner_result.is_ok(), "Spawn should complete without panic");
        assert!(
            inner_result.unwrap().is_ok(),
            "Handler should return Ok on shutdown"
        );
    }

    #[tokio::test]
    async fn event_handler_stops_when_receiver_dropped() {
        let (event_tx, event_rx) = mpsc::channel(1);
        let (_shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let handler = EventHandler::with_tick_rate(
            event_tx,
            shutdown_rx,
            Duration::from_millis(5), // Fast tick rate to quickly fill buffer
        );

        // Spawn the handler
        let handle = tokio::spawn(handler.run());

        // Give it a moment to start and send events
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Drop the receiver - this should cause the handler to exit
        // when it tries to send the next event
        drop(event_rx);

        // Handler should complete since receiver is dropped
        let result = tokio::time::timeout(Duration::from_secs(2), handle).await;
        assert!(result.is_ok(), "Handler should complete within timeout");
        assert!(
            result.unwrap().unwrap().is_ok(),
            "Handler should return Ok when receiver dropped"
        );
    }

    #[tokio::test]
    async fn event_handler_generates_tick_events() {
        let (event_tx, mut event_rx) = mpsc::channel(100);
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let handler = EventHandler::with_tick_rate(
            event_tx,
            shutdown_rx,
            Duration::from_millis(5), // Fast tick rate for testing
        );

        // Spawn the handler
        let handle = tokio::spawn(handler.run());

        // Collect tick events with a timeout
        let mut tick_count = 0;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(200);

        while tokio::time::Instant::now() < deadline {
            match tokio::time::timeout(Duration::from_millis(50), event_rx.recv()).await {
                Ok(Some(TuiEvent::Tick)) => {
                    tick_count += 1;
                    if tick_count >= 3 {
                        break;
                    }
                }
                Ok(Some(_)) => {
                    // Ignore other events
                }
                Ok(None) => {
                    // Channel closed
                    break;
                }
                Err(_) => {
                    // Timeout on individual recv, continue
                }
            }
        }

        // We should have received at least 3 tick events
        assert!(
            tick_count >= 3,
            "Expected at least 3 tick events, got {}",
            tick_count
        );

        // Clean shutdown
        let _ = shutdown_tx.send(());
        let _ = tokio::time::timeout(Duration::from_secs(1), handle).await;
    }

    // =============================================================================
    // Screen Tests
    // =============================================================================

    #[test]
    fn screen_default_is_setup() {
        let screen = Screen::default();
        assert_eq!(screen, Screen::Setup);
    }

    #[test]
    fn screen_variants_are_distinct() {
        assert_ne!(Screen::Setup, Screen::Dashboard);
    }

    #[test]
    fn screen_is_copy() {
        let screen = Screen::Dashboard;
        let copied = screen;
        assert_eq!(screen, copied); // Both still accessible because Screen is Copy
    }

    #[test]
    fn screen_is_debug() {
        let setup = Screen::Setup;
        let dashboard = Screen::Dashboard;

        let setup_debug = format!("{:?}", setup);
        let dashboard_debug = format!("{:?}", dashboard);

        assert!(setup_debug.contains("Setup"));
        assert!(dashboard_debug.contains("Dashboard"));
    }

    // =============================================================================
    // DashboardState Placeholder Tests
    // =============================================================================

    #[test]
    fn dashboard_state_default() {
        let state = DashboardState::default();
        // Just ensure it can be created with default
        let _ = format!("{:?}", state);
    }

    #[test]
    fn dashboard_state_is_clone() {
        let state = DashboardState::default();
        let cloned = state.clone();
        let _ = format!("{:?}", cloned);
    }

    // =============================================================================
    // Theme Tests
    // =============================================================================

    #[test]
    fn theme_default_creates_colorful_theme() {
        let theme = Theme::default();
        // Verify status colors are set
        assert_eq!(theme.status_connected.fg, Some(Color::Green));
        assert_eq!(theme.status_disconnected.fg, Some(Color::Red));
        assert_eq!(theme.status_connecting.fg, Some(Color::Yellow));
        assert_eq!(theme.status_reconnecting.fg, Some(Color::Yellow));
    }

    #[test]
    fn theme_default_event_stream_styles() {
        let theme = Theme::default();
        assert_eq!(theme.event_timestamp.fg, Some(Color::DarkGray));
        assert_eq!(theme.event_type_session.fg, Some(Color::Magenta));
        assert!(theme
            .event_type_session
            .add_modifier
            .contains(Modifier::BOLD));
        assert_eq!(theme.event_type_activity.fg, Some(Color::Blue));
        assert_eq!(theme.event_type_tool.fg, Some(Color::Cyan));
        assert_eq!(theme.event_type_agent.fg, Some(Color::Yellow));
        assert_eq!(theme.event_type_summary.fg, Some(Color::Green));
        assert_eq!(theme.event_type_error.fg, Some(Color::Red));
    }

    #[test]
    fn theme_default_statistics_styles() {
        let theme = Theme::default();
        assert_eq!(theme.stat_total.fg, Some(Color::White));
        assert_eq!(theme.stat_sent.fg, Some(Color::Green));
        assert_eq!(theme.stat_failed.fg, Some(Color::Red));
        assert!(theme.stat_failed.add_modifier.contains(Modifier::BOLD));
        assert_eq!(theme.stat_queued.fg, Some(Color::Yellow));
    }

    #[test]
    fn theme_default_form_styles() {
        let theme = Theme::default();
        assert_eq!(theme.input_focused.fg, Some(Color::Cyan));
        assert!(theme.input_focused.add_modifier.contains(Modifier::BOLD));
        assert_eq!(theme.input_unfocused.fg, Some(Color::Gray));
        assert_eq!(theme.input_error.fg, Some(Color::Red));
        assert_eq!(theme.label.fg, Some(Color::White));
    }

    #[test]
    fn theme_default_layout_styles() {
        let theme = Theme::default();
        assert_eq!(theme.border.fg, Some(Color::DarkGray));
        assert_eq!(theme.border_focused.fg, Some(Color::Cyan));
        assert_eq!(theme.title.fg, Some(Color::White));
        assert!(theme.title.add_modifier.contains(Modifier::BOLD));
        assert_eq!(theme.text_secondary.fg, Some(Color::Gray));
        assert_eq!(theme.text_muted.fg, Some(Color::DarkGray));
    }

    #[test]
    fn theme_monochrome_uses_no_colors() {
        let theme = Theme::monochrome();
        // Monochrome theme should not set foreground colors
        assert_eq!(theme.status_connected.fg, None);
        assert_eq!(theme.status_disconnected.fg, None);
        assert_eq!(theme.event_timestamp.fg, None);
        assert_eq!(theme.stat_total.fg, None);
        assert_eq!(theme.input_focused.fg, None);
        assert_eq!(theme.border.fg, None);
    }

    #[test]
    fn theme_monochrome_uses_modifiers() {
        let theme = Theme::monochrome();
        // Verify modifiers are used instead of colors
        assert!(theme.status_connected.add_modifier.contains(Modifier::BOLD));
        assert!(theme
            .status_disconnected
            .add_modifier
            .contains(Modifier::DIM));
        assert!(theme
            .status_connecting
            .add_modifier
            .contains(Modifier::ITALIC));
        assert!(theme.event_type_error.add_modifier.contains(Modifier::BOLD));
        assert!(theme
            .event_type_error
            .add_modifier
            .contains(Modifier::UNDERLINED));
    }

    #[test]
    fn theme_monochrome_stat_failed_is_distinguishable() {
        let theme = Theme::monochrome();
        // stat_failed should have both BOLD and UNDERLINED for visibility
        assert!(theme.stat_failed.add_modifier.contains(Modifier::BOLD));
        assert!(theme
            .stat_failed
            .add_modifier
            .contains(Modifier::UNDERLINED));
    }

    #[test]
    fn theme_is_debug() {
        let theme = Theme::default();
        let debug_str = format!("{:?}", theme);
        assert!(debug_str.contains("Theme"));
        assert!(debug_str.contains("status_connected"));
    }

    #[test]
    fn theme_is_clone() {
        let theme = Theme::default();
        let cloned = theme.clone();
        assert_eq!(cloned.status_connected.fg, theme.status_connected.fg);
        assert_eq!(cloned.event_type_error.fg, theme.event_type_error.fg);
    }

    #[test]
    fn theme_from_env_returns_colorful_when_no_color_unset() {
        // Temporarily ensure NO_COLOR is not set for this test
        let _guard = EnvGuard::new("NO_COLOR");
        std::env::remove_var("NO_COLOR");

        let theme = Theme::from_env();
        // Should return colorful theme
        assert_eq!(theme.status_connected.fg, Some(Color::Green));
    }

    #[test]
    fn theme_from_env_returns_monochrome_when_no_color_set() {
        let _guard = EnvGuard::new("NO_COLOR");
        std::env::set_var("NO_COLOR", "1");

        let theme = Theme::from_env();
        // Should return monochrome theme
        assert_eq!(theme.status_connected.fg, None);
        assert!(theme.status_connected.add_modifier.contains(Modifier::BOLD));
    }

    // =============================================================================
    // Symbols Tests
    // =============================================================================

    #[test]
    fn symbols_unicode_constants() {
        assert_eq!(UNICODE_SYMBOLS.connected, "●");
        assert_eq!(UNICODE_SYMBOLS.disconnected, "○");
        assert_eq!(UNICODE_SYMBOLS.connecting, "◔");
        assert_eq!(UNICODE_SYMBOLS.reconnecting, "◐");
        assert_eq!(UNICODE_SYMBOLS.success, "✓");
        assert_eq!(UNICODE_SYMBOLS.failure, "✗");
        assert_eq!(UNICODE_SYMBOLS.arrow, "→");
        assert_eq!(UNICODE_SYMBOLS.bullet, "•");
    }

    #[test]
    fn symbols_ascii_constants() {
        assert_eq!(ASCII_SYMBOLS.connected, "[*]");
        assert_eq!(ASCII_SYMBOLS.disconnected, "[ ]");
        assert_eq!(ASCII_SYMBOLS.connecting, "[.]");
        assert_eq!(ASCII_SYMBOLS.reconnecting, "[~]");
        assert_eq!(ASCII_SYMBOLS.success, "[+]");
        assert_eq!(ASCII_SYMBOLS.failure, "[x]");
        assert_eq!(ASCII_SYMBOLS.arrow, "->");
        assert_eq!(ASCII_SYMBOLS.bullet, "*");
    }

    #[test]
    fn symbols_is_debug() {
        let symbols = UNICODE_SYMBOLS;
        let debug_str = format!("{:?}", symbols);
        assert!(debug_str.contains("Symbols"));
        assert!(debug_str.contains("connected"));
    }

    #[test]
    fn symbols_is_copy() {
        let symbols = UNICODE_SYMBOLS;
        let copied = symbols;
        // Both still accessible because Symbols is Copy
        assert_eq!(symbols.connected, copied.connected);
        assert_eq!(symbols.failure, copied.failure);
    }

    #[test]
    fn ascii_symbols_is_copy() {
        let symbols = ASCII_SYMBOLS;
        let copied = symbols; // Copy, not clone
        assert_eq!(copied.arrow, "->");
    }

    #[test]
    fn symbols_detect_returns_unicode_for_typical_terminal() {
        let _guard = EnvGuard::new("TERM");
        std::env::set_var("TERM", "xterm-256color");

        let symbols = Symbols::detect();
        assert_eq!(symbols.connected, "●");
    }

    #[test]
    fn symbols_detect_returns_ascii_for_linux_console() {
        let _guard = EnvGuard::new("TERM");
        std::env::set_var("TERM", "linux");

        let symbols = Symbols::detect();
        assert_eq!(symbols.connected, "[*]");
    }

    #[test]
    fn symbols_detect_returns_ascii_for_vt100() {
        let _guard = EnvGuard::new("TERM");
        std::env::set_var("TERM", "vt100");

        let symbols = Symbols::detect();
        assert_eq!(symbols.connected, "[*]");
    }

    #[test]
    fn symbols_detect_returns_unicode_when_term_unset() {
        let _guard = EnvGuard::new("TERM");
        std::env::remove_var("TERM");

        let symbols = Symbols::detect();
        assert_eq!(symbols.connected, "●");
    }

    #[test]
    fn symbols_default_calls_detect() {
        // Default should behave the same as detect()
        let _guard = EnvGuard::new("TERM");
        std::env::set_var("TERM", "xterm");

        let default_symbols = Symbols::default();
        let detected_symbols = Symbols::detect();
        assert_eq!(default_symbols.connected, detected_symbols.connected);
    }

    /// RAII guard for environment variable testing.
    /// Saves the current value and restores it on drop.
    struct EnvGuard {
        key: &'static str,
        original: Option<String>,
    }

    impl EnvGuard {
        fn new(key: &'static str) -> Self {
            let original = std::env::var(key).ok();
            Self { key, original }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    // =============================================================================
    // AppState Tests
    // =============================================================================

    #[test]
    fn app_state_new_starts_on_setup_screen() {
        let state = AppState::new();
        assert_eq!(state.screen, Screen::Setup);
    }

    #[test]
    fn app_state_default_matches_new() {
        let from_new = AppState::new();
        let from_default = AppState::default();

        assert_eq!(from_new.screen, from_default.screen);
        assert_eq!(from_new.should_quit, from_default.should_quit);
    }

    #[test]
    fn app_state_new_does_not_start_quit() {
        let state = AppState::new();
        assert!(!state.should_quit);
    }

    #[test]
    fn app_state_is_setup_returns_true_on_setup_screen() {
        let state = AppState::new();
        assert!(state.is_setup());
    }

    #[test]
    fn app_state_is_setup_returns_false_on_dashboard_screen() {
        let mut state = AppState::new();
        state.screen = Screen::Dashboard;
        assert!(!state.is_setup());
    }

    #[test]
    fn app_state_is_dashboard_returns_false_on_setup_screen() {
        let state = AppState::new();
        assert!(!state.is_dashboard());
    }

    #[test]
    fn app_state_is_dashboard_returns_true_on_dashboard_screen() {
        let mut state = AppState::new();
        state.screen = Screen::Dashboard;
        assert!(state.is_dashboard());
    }

    #[test]
    fn app_state_should_quit_returns_false_initially() {
        let state = AppState::new();
        assert!(!state.should_quit());
    }

    #[test]
    fn app_state_quit_sets_should_quit_to_true() {
        let mut state = AppState::new();
        assert!(!state.should_quit());

        state.quit();
        assert!(state.should_quit());
    }

    #[test]
    fn app_state_quit_is_idempotent() {
        let mut state = AppState::new();

        state.quit();
        assert!(state.should_quit());

        // Calling quit again should still be true
        state.quit();
        assert!(state.should_quit());
    }

    #[test]
    fn app_state_is_debug() {
        let state = AppState::new();
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("AppState"));
        assert!(debug_str.contains("screen"));
        assert!(debug_str.contains("should_quit"));
    }

    #[test]
    fn app_state_is_clone() {
        let mut state = AppState::new();
        state.screen = Screen::Dashboard;
        state.quit();

        let cloned = state.clone();
        assert_eq!(cloned.screen, Screen::Dashboard);
        assert!(cloned.should_quit());
    }

    #[test]
    fn app_state_screen_can_be_changed() {
        let mut state = AppState::new();
        assert!(state.is_setup());

        state.screen = Screen::Dashboard;
        assert!(state.is_dashboard());

        state.screen = Screen::Setup;
        assert!(state.is_setup());
    }

    // =============================================================================
    // SetupField Tests (T053)
    // =============================================================================

    #[test]
    fn setup_field_default_is_session_name() {
        let field = SetupField::default();
        assert_eq!(field, SetupField::SessionName);
    }

    #[test]
    fn setup_field_variants_are_distinct() {
        assert_ne!(SetupField::SessionName, SetupField::KeyOption);
        assert_ne!(SetupField::KeyOption, SetupField::Submit);
        assert_ne!(SetupField::SessionName, SetupField::Submit);
    }

    #[test]
    fn setup_field_is_copy() {
        let field = SetupField::KeyOption;
        let copied = field;
        assert_eq!(field, copied); // Both accessible because SetupField is Copy
    }

    #[test]
    fn setup_field_is_clone() {
        let field = SetupField::Submit;
        let cloned = field.clone();
        assert_eq!(field, cloned);
    }

    #[test]
    fn setup_field_is_debug() {
        let session_name = SetupField::SessionName;
        let key_option = SetupField::KeyOption;
        let submit = SetupField::Submit;

        let debug1 = format!("{:?}", session_name);
        let debug2 = format!("{:?}", key_option);
        let debug3 = format!("{:?}", submit);

        assert!(debug1.contains("SessionName"));
        assert!(debug2.contains("KeyOption"));
        assert!(debug3.contains("Submit"));
    }

    #[test]
    fn setup_field_is_eq() {
        let field1 = SetupField::SessionName;
        let field2 = SetupField::SessionName;
        let field3 = SetupField::KeyOption;

        assert_eq!(field1, field2);
        assert_ne!(field1, field3);
    }

    // =============================================================================
    // KeyOption Tests (T054)
    // =============================================================================

    #[test]
    fn key_option_default_is_generate_new() {
        let option = KeyOption::default();
        assert_eq!(option, KeyOption::GenerateNew);
    }

    #[test]
    fn key_option_variants_are_distinct() {
        assert_ne!(KeyOption::UseExisting, KeyOption::GenerateNew);
    }

    #[test]
    fn key_option_toggle_from_generate_new_to_use_existing() {
        let option = KeyOption::GenerateNew;
        let toggled = option.toggle();
        assert_eq!(toggled, KeyOption::UseExisting);
    }

    #[test]
    fn key_option_toggle_from_use_existing_to_generate_new() {
        let option = KeyOption::UseExisting;
        let toggled = option.toggle();
        assert_eq!(toggled, KeyOption::GenerateNew);
    }

    #[test]
    fn key_option_toggle_is_reversible() {
        let original = KeyOption::GenerateNew;
        let toggled = original.toggle().toggle();
        assert_eq!(toggled, original);

        let original = KeyOption::UseExisting;
        let toggled = original.toggle().toggle();
        assert_eq!(toggled, original);
    }

    #[test]
    fn key_option_is_copy() {
        let option = KeyOption::UseExisting;
        let copied = option;
        assert_eq!(option, copied); // Both accessible because KeyOption is Copy
    }

    #[test]
    fn key_option_is_clone() {
        let option = KeyOption::GenerateNew;
        let cloned = option.clone();
        assert_eq!(option, cloned);
    }

    #[test]
    fn key_option_is_debug() {
        let use_existing = KeyOption::UseExisting;
        let generate_new = KeyOption::GenerateNew;

        let debug1 = format!("{:?}", use_existing);
        let debug2 = format!("{:?}", generate_new);

        assert!(debug1.contains("UseExisting"));
        assert!(debug2.contains("GenerateNew"));
    }

    #[test]
    fn key_option_is_eq() {
        let option1 = KeyOption::UseExisting;
        let option2 = KeyOption::UseExisting;
        let option3 = KeyOption::GenerateNew;

        assert_eq!(option1, option2);
        assert_ne!(option1, option3);
    }

    // =============================================================================
    // SetupFormState Tests (T052)
    // =============================================================================

    #[test]
    fn setup_form_state_default_has_empty_session_name() {
        let state = SetupFormState::default();
        assert!(state.session_name.is_empty());
    }

    #[test]
    fn setup_form_state_default_has_no_error() {
        let state = SetupFormState::default();
        assert!(state.session_name_error.is_none());
    }

    #[test]
    fn setup_form_state_default_key_option_is_generate_new() {
        let state = SetupFormState::default();
        assert_eq!(state.key_option, KeyOption::GenerateNew);
    }

    #[test]
    fn setup_form_state_default_focused_field_is_session_name() {
        let state = SetupFormState::default();
        assert_eq!(state.focused_field, SetupField::SessionName);
    }

    #[test]
    fn setup_form_state_default_existing_keys_found_is_false() {
        let state = SetupFormState::default();
        assert!(!state.existing_keys_found);
    }

    #[test]
    fn setup_form_state_can_be_constructed_with_values() {
        let state = SetupFormState {
            session_name: "test-session".to_string(),
            session_name_error: Some("Invalid character".to_string()),
            key_option: KeyOption::UseExisting,
            focused_field: SetupField::Submit,
            existing_keys_found: true,
        };

        assert_eq!(state.session_name, "test-session");
        assert_eq!(
            state.session_name_error,
            Some("Invalid character".to_string())
        );
        assert_eq!(state.key_option, KeyOption::UseExisting);
        assert_eq!(state.focused_field, SetupField::Submit);
        assert!(state.existing_keys_found);
    }

    #[test]
    fn setup_form_state_is_clone() {
        let state = SetupFormState {
            session_name: "my-session".to_string(),
            session_name_error: None,
            key_option: KeyOption::GenerateNew,
            focused_field: SetupField::KeyOption,
            existing_keys_found: false,
        };

        let cloned = state.clone();
        assert_eq!(cloned.session_name, state.session_name);
        assert_eq!(cloned.session_name_error, state.session_name_error);
        assert_eq!(cloned.key_option, state.key_option);
        assert_eq!(cloned.focused_field, state.focused_field);
        assert_eq!(cloned.existing_keys_found, state.existing_keys_found);
    }

    #[test]
    fn setup_form_state_is_debug() {
        let state = SetupFormState::default();
        let debug_str = format!("{:?}", state);

        assert!(debug_str.contains("SetupFormState"));
        assert!(debug_str.contains("session_name"));
        assert!(debug_str.contains("key_option"));
        assert!(debug_str.contains("focused_field"));
        assert!(debug_str.contains("existing_keys_found"));
    }

    #[test]
    fn setup_form_state_session_name_can_be_modified() {
        let mut state = SetupFormState::default();
        assert!(state.session_name.is_empty());

        state.session_name = "modified-session".to_string();
        assert_eq!(state.session_name, "modified-session");
    }

    #[test]
    fn setup_form_state_error_can_be_set_and_cleared() {
        let mut state = SetupFormState::default();
        assert!(state.session_name_error.is_none());

        state.session_name_error = Some("Error message".to_string());
        assert_eq!(state.session_name_error, Some("Error message".to_string()));

        state.session_name_error = None;
        assert!(state.session_name_error.is_none());
    }

    #[test]
    fn setup_form_state_key_option_can_be_toggled() {
        let mut state = SetupFormState::default();
        assert_eq!(state.key_option, KeyOption::GenerateNew);

        state.key_option = state.key_option.toggle();
        assert_eq!(state.key_option, KeyOption::UseExisting);

        state.key_option = state.key_option.toggle();
        assert_eq!(state.key_option, KeyOption::GenerateNew);
    }

    #[test]
    fn setup_form_state_focused_field_can_be_changed() {
        let mut state = SetupFormState::default();
        assert_eq!(state.focused_field, SetupField::SessionName);

        state.focused_field = SetupField::KeyOption;
        assert_eq!(state.focused_field, SetupField::KeyOption);

        state.focused_field = SetupField::Submit;
        assert_eq!(state.focused_field, SetupField::Submit);
    }
}
