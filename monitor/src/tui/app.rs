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

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::{DateTime, Local};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use ratatui::style::{Color, Modifier, Style};
use tokio::sync::{mpsc, oneshot};

use crate::types::{Event, EventType};

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

// =============================================================================
// Key Detection and Hostname Detection
// =============================================================================

/// Checks if cryptographic keys already exist in the default location.
///
/// Checks for the existence of key files in `~/.vibetea/`.
///
/// # Returns
///
/// `true` if keys exist, `false` otherwise (including if the directory
/// doesn't exist or cannot be accessed).
///
/// # FR-004 Compliance
///
/// This function supports FR-004: "Key generation option MUST default to
/// 'Use existing' if keys exist, or 'Generate new' if no keys exist."
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::detect_existing_keys;
///
/// let keys_exist = detect_existing_keys();
/// if keys_exist {
///     println!("Found existing keys in ~/.vibetea");
/// } else {
///     println!("No existing keys found");
/// }
/// ```
pub fn detect_existing_keys() -> bool {
    let key_dir = directories::BaseDirs::new()
        .map(|dirs| dirs.home_dir().join(".vibetea"))
        .unwrap_or_else(|| PathBuf::from(".vibetea"));

    crate::crypto::Crypto::exists(&key_dir)
}

/// Checks if cryptographic keys exist in a specific directory.
///
/// This is useful for testing or when using a custom key directory.
///
/// # Arguments
///
/// * `key_dir` - Directory to check for key files
///
/// # Returns
///
/// `true` if keys exist in the specified directory, `false` otherwise.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::detect_existing_keys_in_dir;
/// use std::path::Path;
///
/// let keys_exist = detect_existing_keys_in_dir(Path::new("/custom/key/dir"));
/// ```
pub fn detect_existing_keys_in_dir(key_dir: &Path) -> bool {
    crate::crypto::Crypto::exists(key_dir)
}

/// Creates a new [`SetupFormState`] with auto-detected defaults.
///
/// This function:
/// - Sets the session name to the system hostname (FR-003)
/// - Detects if existing keys are present (FR-004)
/// - Sets the key option to [`KeyOption::UseExisting`] if keys exist,
///   [`KeyOption::GenerateNew`] otherwise
///
/// Use this function when initializing the TUI to provide the best defaults
/// based on the current system state.
///
/// # FR-003 Compliance
///
/// The session name defaults to the system hostname as specified in FR-003.
///
/// # FR-004 Compliance
///
/// The key option defaults based on whether existing keys are found:
/// - "Use existing" if keys exist in `~/.vibetea`
/// - "Generate new" if no keys exist
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{setup_form_with_detected_defaults, KeyOption};
///
/// let form_state = setup_form_with_detected_defaults();
///
/// // Session name is set to hostname
/// assert!(!form_state.session_name.is_empty());
///
/// // Key option is set based on whether keys exist
/// if form_state.existing_keys_found {
///     assert_eq!(form_state.key_option, KeyOption::UseExisting);
/// } else {
///     assert_eq!(form_state.key_option, KeyOption::GenerateNew);
/// }
/// ```
pub fn setup_form_with_detected_defaults() -> SetupFormState {
    let existing_keys = detect_existing_keys();

    SetupFormState {
        session_name: default_session_name(),
        session_name_error: None,
        key_option: if existing_keys {
            KeyOption::UseExisting
        } else {
            KeyOption::GenerateNew
        },
        focused_field: SetupField::default(),
        existing_keys_found: existing_keys,
    }
}

/// Creates a new [`SetupFormState`] with auto-detected defaults for a specific key directory.
///
/// This is useful for testing or when using a custom key directory.
///
/// # Arguments
///
/// * `key_dir` - Directory to check for existing keys
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{setup_form_with_detected_defaults_in_dir, KeyOption};
/// use std::path::Path;
///
/// let form_state = setup_form_with_detected_defaults_in_dir(Path::new("/custom/key/dir"));
/// ```
pub fn setup_form_with_detected_defaults_in_dir(key_dir: &Path) -> SetupFormState {
    let existing_keys = detect_existing_keys_in_dir(key_dir);

    SetupFormState {
        session_name: default_session_name(),
        session_name_error: None,
        key_option: if existing_keys {
            KeyOption::UseExisting
        } else {
            KeyOption::GenerateNew
        },
        focused_field: SetupField::default(),
        existing_keys_found: existing_keys,
    }
}

/// Returns the system hostname for use as the default session name.
///
/// This function attempts to detect the local machine's hostname using the
/// `gethostname` crate, which provides cross-platform support for Linux,
/// macOS, and Windows.
///
/// # Fallback Behavior
///
/// If the hostname cannot be determined (e.g., the hostname contains invalid
/// UTF-8 characters or the system call fails), this function falls back to
/// returning `"monitor"` as a safe default.
///
/// # FR-003 Compliance
///
/// This function implements FR-003: "Session name field MUST default to the
/// system hostname."
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::default_session_name;
///
/// let name = default_session_name();
/// // Returns the hostname or "monitor" as fallback
/// assert!(!name.is_empty());
/// ```
pub fn default_session_name() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "monitor".to_string())
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
/// // Create default state (session_name defaults to hostname)
/// let state = SetupFormState::default();
/// assert!(!state.session_name.is_empty()); // Hostname or "monitor" fallback
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
#[derive(Debug, Clone)]
pub struct SetupFormState {
    /// Current session name input value.
    ///
    /// Defaults to the system hostname per FR-003, with a fallback to "monitor"
    /// if the hostname cannot be determined. Limited to 64 characters maximum,
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

impl Default for SetupFormState {
    /// Creates a new [`SetupFormState`] with the session name defaulting to the
    /// system hostname per FR-003.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{SetupFormState, SetupField, KeyOption};
    ///
    /// let state = SetupFormState::default();
    /// assert!(!state.session_name.is_empty()); // Hostname or "monitor"
    /// assert!(state.session_name_error.is_none());
    /// assert_eq!(state.key_option, KeyOption::GenerateNew);
    /// assert_eq!(state.focused_field, SetupField::SessionName);
    /// assert!(!state.existing_keys_found);
    /// ```
    fn default() -> Self {
        Self {
            session_name: default_session_name(),
            session_name_error: None,
            key_option: KeyOption::default(),
            focused_field: SetupField::default(),
            existing_keys_found: false,
        }
    }
}

/// State for the dashboard screen.
///
/// Contains all state needed to render and update the dashboard view,
/// including event streams, metrics, and UI state.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{DashboardState, ConnectionStatus, EventStats};
///
/// let state = DashboardState::default();
/// assert!(state.session_name.is_empty());
/// assert!(state.event_buffer.is_empty());
/// ```
#[derive(Debug, Clone, Default)]
pub struct DashboardState {
    /// Session name used for identification.
    ///
    /// This is the trimmed session name from the setup form, used for
    /// display in the dashboard header and for identifying this monitor
    /// session on the server.
    pub session_name: String,

    /// Base64-encoded public key for server configuration.
    ///
    /// This key should be registered with the VibeTea server via the
    /// `VIBETEA_PUBLIC_KEYS` environment variable to authorize this
    /// monitor to send events.
    pub public_key: String,

    /// Connection status to the VibeTea server.
    ///
    /// Tracks the current WebSocket connection state for display in
    /// the status indicator.
    pub connection_status: ConnectionStatus,

    /// Event statistics (sent, failed counts).
    ///
    /// Updated as events are sent to the server, used for the stats
    /// footer display.
    pub stats: EventStats,

    /// Bounded event buffer for display events (FR-011).
    ///
    /// Stores the most recent events for display in the event stream widget.
    /// Older events are automatically evicted when the buffer reaches capacity.
    pub event_buffer: EventBuffer,

    /// Scroll state for event stream navigation (FR-011).
    ///
    /// Tracks the current scroll position and auto-scroll behavior
    /// for the event stream widget.
    pub scroll: ScrollState,
}

impl DashboardState {
    /// Pushes a display event to the event buffer and updates scroll state.
    ///
    /// This method adds a new event to the buffer and updates the scroll state's
    /// total event count. If auto-scroll is enabled, the view will automatically
    /// show the newest events.
    ///
    /// # Arguments
    ///
    /// * `event` - The display event to add to the buffer
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{DashboardState, DisplayEvent, DisplayEventType};
    ///
    /// let mut dashboard = DashboardState::default();
    ///
    /// let event = DisplayEvent::new(
    ///     "evt_123".to_string(),
    ///     DisplayEventType::Session,
    ///     "Session started".to_string(),
    /// );
    ///
    /// dashboard.push_event(event);
    ///
    /// assert_eq!(dashboard.event_buffer.len(), 1);
    /// assert_eq!(dashboard.scroll.total_events(), 1);
    /// ```
    pub fn push_event(&mut self, event: DisplayEvent) {
        self.event_buffer.push(event);
        self.scroll.update_total_events(self.event_buffer.len());
    }
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

    /// Completes the setup process and transitions to the dashboard.
    ///
    /// Loads or generates cryptographic keys based on the setup form's key option,
    /// validates the session name, and transitions to the dashboard screen.
    ///
    /// # Arguments
    ///
    /// * `key_dir` - Optional directory for key storage. Defaults to `~/.vibetea`
    ///
    /// # Key Handling
    ///
    /// The method handles keys based on the [`KeyOption`] selected in the setup form:
    ///
    /// - **`UseExisting`**: Loads existing keys from the key directory. Returns an
    ///   error if no keys are found.
    /// - **`GenerateNew`**: Generates a new Ed25519 keypair and saves it to the
    ///   key directory, overwriting any existing keys.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the transition was successful
    /// - `Err(String)` if validation fails or key operations fail
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vibetea_monitor::tui::app::AppState;
    /// use std::path::Path;
    ///
    /// let mut state = AppState::new();
    /// state.setup.session_name = "my-session".to_string();
    ///
    /// assert!(state.is_setup());
    /// // Use a custom key directory for testing
    /// state.complete_setup(Some(Path::new("/tmp/vibetea-test"))).unwrap();
    /// assert!(state.is_dashboard());
    /// assert!(!state.dashboard.public_key.is_empty());
    /// ```
    pub fn complete_setup(&mut self, key_dir: Option<&Path>) -> Result<(), String> {
        use crate::crypto::Crypto;

        // Validate session name using the validation function
        crate::tui::widgets::validate_session_name(&self.setup.session_name)?;

        // Determine key directory
        let key_path = key_dir.map(|p| p.to_path_buf()).unwrap_or_else(|| {
            directories::BaseDirs::new()
                .map(|dirs| dirs.home_dir().join(".vibetea"))
                .unwrap_or_else(|| PathBuf::from(".vibetea"))
        });

        // Load or generate keys based on the selected option
        let crypto = match self.setup.key_option {
            KeyOption::UseExisting if Crypto::exists(&key_path) => Crypto::load(&key_path)
                .map_err(|e| format!("Failed to load existing keys: {}", e))?,
            KeyOption::UseExisting => {
                // Keys don't exist but user selected "use existing"
                return Err("No existing keys found. Please select 'Generate new key'.".to_string());
            }
            KeyOption::GenerateNew => {
                let crypto = Crypto::generate();
                crypto
                    .save(&key_path)
                    .map_err(|e| format!("Failed to save new keys: {}", e))?;
                crypto
            }
        };

        // Clear any previous error
        self.setup.session_name_error = None;

        // Transition to dashboard
        self.screen = Screen::Dashboard;

        // Store credentials in dashboard state, preserving event_buffer and scroll
        self.dashboard = DashboardState {
            session_name: self.setup.session_name.trim().to_string(),
            public_key: crypto.public_key_base64(),
            event_buffer: std::mem::take(&mut self.dashboard.event_buffer),
            scroll: std::mem::take(&mut self.dashboard.scroll),
            ..Default::default()
        };

        Ok(())
    }

    /// Handles a watcher event by converting it to a display event and adding to the buffer.
    ///
    /// This method converts a [`crate::types::Event`] from the file watcher into a
    /// [`DisplayEvent`] and pushes it to the dashboard's event buffer. The scroll
    /// state is automatically updated to reflect the new event count.
    ///
    /// # Auto-Scroll Behavior
    ///
    /// If auto-scroll is enabled on the scroll state, the view will automatically
    /// stay at the bottom to show the newest events (FR-011).
    ///
    /// # Arguments
    ///
    /// * `event` - The watcher event to process
    ///
    /// # Example
    ///
    /// ```
    /// use uuid::Uuid;
    /// use vibetea_monitor::tui::app::AppState;
    /// use vibetea_monitor::types::{Event, EventType, EventPayload, SessionAction};
    ///
    /// let mut state = AppState::new();
    ///
    /// let event = Event::new(
    ///     "monitor-1".to_string(),
    ///     EventType::Session,
    ///     EventPayload::Session {
    ///         session_id: Uuid::new_v4(),
    ///         action: SessionAction::Started,
    ///         project: "my-project".to_string(),
    ///     },
    /// );
    ///
    /// state.handle_watch_event(event);
    ///
    /// assert_eq!(state.dashboard.event_buffer.len(), 1);
    /// assert_eq!(state.dashboard.scroll.total_events(), 1);
    /// ```
    pub fn handle_watch_event(&mut self, event: Event) {
        let display_event = DisplayEvent::from(&event);
        self.dashboard.push_event(display_event);
    }

    /// Handles a terminal resize event by updating the scroll state's visible height.
    ///
    /// This method updates the scroll state to reflect the new terminal dimensions,
    /// which affects scroll calculations and event visibility. The visible height
    /// for the event stream is typically the terminal height minus chrome (header,
    /// footer, borders, etc.).
    ///
    /// # Arguments
    ///
    /// * `width` - New terminal width in columns (reserved for future layout use)
    /// * `height` - New terminal height in rows
    ///
    /// # Note
    ///
    /// The actual visible height for the event stream should account for UI chrome.
    /// This method stores the raw terminal height; the caller is responsible for
    /// calculating the effective visible height if needed.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::AppState;
    ///
    /// let mut state = AppState::new();
    ///
    /// // Terminal resized to 80x24
    /// state.handle_resize(80, 24);
    ///
    /// // The visible height is updated (actual visible area depends on layout)
    /// assert_eq!(state.dashboard.scroll.visible_height(), 24);
    /// ```
    pub fn handle_resize(&mut self, _width: u16, height: u16) {
        self.dashboard.scroll.update_visible_height(height as usize);
    }

    /// Sets the connection status to the specified value.
    ///
    /// Updates the dashboard's connection status, which is displayed in the
    /// header widget to show the current server connection state (FR-012).
    ///
    /// # Arguments
    ///
    /// * `status` - The new connection status
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{AppState, ConnectionStatus};
    ///
    /// let mut state = AppState::new();
    /// assert_eq!(state.dashboard.connection_status, ConnectionStatus::Disconnected);
    ///
    /// state.set_connection_status(ConnectionStatus::Connected);
    /// assert_eq!(state.dashboard.connection_status, ConnectionStatus::Connected);
    /// ```
    pub fn set_connection_status(&mut self, status: ConnectionStatus) {
        self.dashboard.connection_status = status;
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

/// Session credentials for display in the credentials panel (FR-009, FR-010).
///
/// This struct holds the authentication credentials that users need to configure
/// the VibeTea server to accept events from this monitor instance. The public key
/// should be added to the server's `VIBETEA_PUBLIC_KEYS` environment variable.
///
/// # Fields
///
/// - `session_name`: The source_id used to identify this monitor session
/// - `public_key`: Base64-encoded Ed25519 public key for copy-paste to server config
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::Credentials;
///
/// let credentials = Credentials {
///     session_name: "my-workstation".to_string(),
///     public_key: "YWJjZGVmZ2hpamtsbW5vcHFyc3R1dnd4eXo=".to_string(),
/// };
///
/// assert_eq!(credentials.session_name, "my-workstation");
/// ```
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Credentials {
    /// Session name (source_id) used for identification.
    ///
    /// This is the name configured during setup that identifies this monitor
    /// instance when viewing events on the server.
    pub session_name: String,

    /// Base64-encoded Ed25519 public key.
    ///
    /// This key must be registered with the VibeTea server by adding it to
    /// the `VIBETEA_PUBLIC_KEYS` environment variable. The format is suitable
    /// for direct copy-paste.
    pub public_key: String,
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

/// Event type categories for display in the event stream (FR-010).
///
/// Maps from the underlying [`crate::types::EventType`] to a display-friendly
/// representation. Each variant corresponds to a unique visual identifier
/// (icon + color) in the TUI theme.
///
/// # Variants
///
/// - **Session**: Session lifecycle events (started, ended)
/// - **Activity**: Activity heartbeat events
/// - **Tool**: Tool usage events (started, completed)
/// - **Agent**: Agent state change events
/// - **Summary**: Session summary events
/// - **Error**: Error events
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::DisplayEventType;
/// use vibetea_monitor::types::EventType;
///
/// // Convert from EventType
/// let display_type = DisplayEventType::from(EventType::Tool);
/// assert_eq!(display_type, DisplayEventType::Tool);
///
/// // Get icon for display
/// let icon = display_type.icon();
/// assert!(!icon.is_empty());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEventType {
    /// Session lifecycle event (started, ended).
    Session,
    /// Activity heartbeat event.
    Activity,
    /// Tool usage event (started, completed).
    Tool,
    /// Agent state change event.
    Agent,
    /// Session summary event.
    Summary,
    /// Error event.
    Error,
}

impl DisplayEventType {
    /// Returns a short label for this event type.
    ///
    /// Used for text display alongside icons.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::DisplayEventType;
    ///
    /// assert_eq!(DisplayEventType::Session.label(), "SESSION");
    /// assert_eq!(DisplayEventType::Tool.label(), "TOOL");
    /// ```
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Session => "SESSION",
            Self::Activity => "ACTIVITY",
            Self::Tool => "TOOL",
            Self::Agent => "AGENT",
            Self::Summary => "SUMMARY",
            Self::Error => "ERROR",
        }
    }

    /// Returns a unicode icon for this event type.
    ///
    /// Icons provide visual distinction per FR-010: "Each event type
    /// MUST have a unique visual identifier (icon + color)."
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::DisplayEventType;
    ///
    /// assert_eq!(DisplayEventType::Session.icon(), "\u{1F4AC}"); // speech balloon
    /// assert_eq!(DisplayEventType::Error.icon(), "\u{26A0}");    // warning sign
    /// ```
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::Session => "\u{1F4AC}",  // speech balloon
            Self::Activity => "\u{1F49A}", // green heart (activity/heartbeat)
            Self::Tool => "\u{1F527}",     // wrench
            Self::Agent => "\u{1F916}",    // robot face
            Self::Summary => "\u{1F4CB}",  // clipboard
            Self::Error => "\u{26A0}",     // warning sign
        }
    }

    /// Returns an ASCII icon for terminals without unicode support.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::DisplayEventType;
    ///
    /// assert_eq!(DisplayEventType::Session.ascii_icon(), "[S]");
    /// assert_eq!(DisplayEventType::Tool.ascii_icon(), "[T]");
    /// ```
    #[must_use]
    pub const fn ascii_icon(&self) -> &'static str {
        match self {
            Self::Session => "[S]",
            Self::Activity => "[A]",
            Self::Tool => "[T]",
            Self::Agent => "[G]",
            Self::Summary => "[M]",
            Self::Error => "[!]",
        }
    }
}

impl From<EventType> for DisplayEventType {
    /// Converts from the underlying event type to the display type.
    ///
    /// This is a direct 1:1 mapping since the display types match
    /// the underlying event types.
    fn from(event_type: EventType) -> Self {
        match event_type {
            EventType::Session => Self::Session,
            EventType::Activity => Self::Activity,
            EventType::Tool => Self::Tool,
            EventType::Agent => Self::Agent,
            EventType::Summary => Self::Summary,
            EventType::Error => Self::Error,
        }
    }
}

impl From<&EventType> for DisplayEventType {
    /// Converts from a reference to the underlying event type.
    fn from(event_type: &EventType) -> Self {
        Self::from(*event_type)
    }
}

/// Event formatted for display in the event stream (FR-009, FR-010, FR-011).
///
/// Represents a processed event ready for rendering in the TUI event stream.
/// Contains all information needed to display the event including timestamp,
/// type classification, and human-readable message.
///
/// # FR-009 Compliance
///
/// Per FR-009: "Event stream MUST show timestamp, event type, and message
/// for each event." This struct provides all three components.
///
/// # FR-010 Compliance
///
/// Per FR-010: "Each event type MUST have a unique visual identifier
/// (icon + color)." The `event_type` field enables this through the
/// [`DisplayEventType`] methods.
///
/// # FR-011 Compliance
///
/// Per FR-011: "New events MUST auto-scroll unless user has manually
/// scrolled up." The `age_secs` field supports highlighting recent events.
///
/// # Example
///
/// ```
/// use chrono::Local;
/// use vibetea_monitor::tui::app::{DisplayEvent, DisplayEventType};
///
/// let event = DisplayEvent {
///     id: "evt_abc123".to_string(),
///     timestamp: Local::now(),
///     event_type: DisplayEventType::Tool,
///     message: "Read file: src/main.rs".to_string(),
/// };
///
/// assert_eq!(event.event_type.label(), "TOOL");
/// ```
#[derive(Debug, Clone)]
pub struct DisplayEvent {
    /// Unique event identifier.
    ///
    /// Used for keying and deduplication. Typically in the format
    /// `evt_` followed by alphanumeric characters.
    pub id: String,

    /// When the event occurred.
    ///
    /// Stored in local time for display formatting. The event stream
    /// typically shows this as `HH:MM:SS`.
    pub timestamp: DateTime<Local>,

    /// Event type for icon and color selection.
    ///
    /// Determines the visual presentation of the event in the stream.
    pub event_type: DisplayEventType,

    /// Human-readable event message.
    ///
    /// A brief description of what the event represents. May be
    /// truncated during rendering to fit the available width.
    pub message: String,
}

impl DisplayEvent {
    /// Creates a new `DisplayEvent` with the current timestamp.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique event identifier
    /// * `event_type` - Type classification for display
    /// * `message` - Human-readable description
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{DisplayEvent, DisplayEventType};
    ///
    /// let event = DisplayEvent::new(
    ///     "evt_xyz789".to_string(),
    ///     DisplayEventType::Session,
    ///     "Session started".to_string(),
    /// );
    ///
    /// assert_eq!(event.id, "evt_xyz789");
    /// assert_eq!(event.event_type, DisplayEventType::Session);
    /// ```
    #[must_use]
    pub fn new(id: String, event_type: DisplayEventType, message: String) -> Self {
        Self {
            id,
            timestamp: Local::now(),
            event_type,
            message,
        }
    }

    /// Returns the age of this event in seconds.
    ///
    /// Useful for highlighting recent events in the display.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{DisplayEvent, DisplayEventType};
    ///
    /// let event = DisplayEvent::new(
    ///     "evt_1".to_string(),
    ///     DisplayEventType::Activity,
    ///     "Heartbeat".to_string(),
    /// );
    ///
    /// // Just created, should be very recent
    /// assert!(event.age_secs() < 2);
    /// ```
    #[must_use]
    pub fn age_secs(&self) -> u64 {
        let now = Local::now();
        now.signed_duration_since(self.timestamp)
            .num_seconds()
            .max(0) as u64
    }

    /// Formats the timestamp as `HH:MM:SS` for display.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{DisplayEvent, DisplayEventType};
    ///
    /// let event = DisplayEvent::new(
    ///     "evt_1".to_string(),
    ///     DisplayEventType::Tool,
    ///     "Executed command".to_string(),
    /// );
    ///
    /// let formatted = event.formatted_timestamp();
    /// // Format: "HH:MM:SS" (8 characters)
    /// assert_eq!(formatted.len(), 8);
    /// ```
    #[must_use]
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%H:%M:%S").to_string()
    }
}

impl From<&Event> for DisplayEvent {
    /// Converts from the underlying `Event` type to a `DisplayEvent`.
    ///
    /// Extracts the relevant fields and generates a human-readable
    /// message based on the event payload.
    fn from(event: &Event) -> Self {
        use crate::types::EventPayload;

        let message = match &event.payload {
            EventPayload::Session {
                action, project, ..
            } => format!("{:?} session for {}", action, project),
            EventPayload::Activity { project, .. } => {
                if let Some(proj) = project {
                    format!("Activity heartbeat ({})", proj)
                } else {
                    "Activity heartbeat".to_string()
                }
            }
            EventPayload::Tool {
                tool,
                status,
                context,
                ..
            } => {
                let ctx = context
                    .as_ref()
                    .map(|c| format!(" - {}", c))
                    .unwrap_or_default();
                format!("{}: {:?}{}", tool, status, ctx)
            }
            EventPayload::Agent { state, .. } => format!("Agent state: {}", state),
            EventPayload::Summary { summary, .. } => {
                // Truncate long summaries for display
                if summary.len() > 50 {
                    format!("{}...", &summary[..47])
                } else {
                    summary.clone()
                }
            }
            EventPayload::Error { category, .. } => format!("Error: {}", category),
        };

        Self {
            id: event.id.clone(),
            // Convert from UTC to Local time
            timestamp: event.timestamp.with_timezone(&Local),
            event_type: DisplayEventType::from(event.event_type),
            message,
        }
    }
}

// =============================================================================
// Event Buffer
// =============================================================================

/// Default maximum capacity for the event buffer (FR-011).
pub const DEFAULT_EVENT_BUFFER_CAPACITY: usize = 1000;

/// A bounded FIFO buffer for storing display events.
///
/// The `EventBuffer` maintains a fixed-capacity queue of [`DisplayEvent`] items,
/// automatically evicting the oldest events when the capacity is reached. This
/// implements the FR-011 requirement for limiting the event stream to the 1000
/// most recent events.
///
/// # FIFO Eviction
///
/// When a new event is pushed and the buffer is at capacity, the oldest event
/// (at the front of the queue) is removed before the new event is added to the
/// back. This ensures constant memory usage regardless of how many events are
/// received over time.
///
/// # Iteration Order
///
/// Events are stored and iterated in chronological order (oldest to newest),
/// which matches the natural display order for an event stream where older
/// events appear at the top and newer events at the bottom.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
///
/// // Create a buffer with small capacity for demonstration
/// let mut buffer = EventBuffer::new(3);
///
/// // Push some events
/// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "First".into()));
/// buffer.push(DisplayEvent::new("evt_2".into(), DisplayEventType::Tool, "Second".into()));
/// buffer.push(DisplayEvent::new("evt_3".into(), DisplayEventType::Activity, "Third".into()));
///
/// assert_eq!(buffer.len(), 3);
///
/// // Adding a fourth event evicts the oldest
/// buffer.push(DisplayEvent::new("evt_4".into(), DisplayEventType::Session, "Fourth".into()));
/// assert_eq!(buffer.len(), 3);
///
/// // The first event was evicted
/// assert_eq!(buffer.get(0).unwrap().id, "evt_2");
/// ```
#[derive(Debug, Clone)]
pub struct EventBuffer {
    /// Internal storage using VecDeque for efficient front/back operations.
    events: VecDeque<DisplayEvent>,
    /// Maximum number of events to retain.
    max_capacity: usize,
}

impl EventBuffer {
    /// Creates a new `EventBuffer` with the specified capacity.
    ///
    /// # Arguments
    ///
    /// * `capacity` - Maximum number of events to store before eviction begins
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::EventBuffer;
    ///
    /// let buffer = EventBuffer::new(500);
    /// assert!(buffer.is_empty());
    /// ```
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "EventBuffer capacity must be greater than 0");
        Self {
            events: VecDeque::with_capacity(capacity),
            max_capacity: capacity,
        }
    }

    /// Pushes a new event into the buffer.
    ///
    /// If the buffer is at capacity, the oldest event (at the front) is
    /// removed before the new event is added to the back. This ensures
    /// FIFO eviction behavior.
    ///
    /// # Arguments
    ///
    /// * `event` - The display event to add
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
    ///
    /// let mut buffer = EventBuffer::new(2);
    /// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "One".into()));
    /// buffer.push(DisplayEvent::new("evt_2".into(), DisplayEventType::Tool, "Two".into()));
    ///
    /// // Buffer is at capacity, next push evicts oldest
    /// buffer.push(DisplayEvent::new("evt_3".into(), DisplayEventType::Activity, "Three".into()));
    ///
    /// assert_eq!(buffer.len(), 2);
    /// assert_eq!(buffer.get(0).unwrap().id, "evt_2"); // evt_1 was evicted
    /// ```
    pub fn push(&mut self, event: DisplayEvent) {
        if self.events.len() >= self.max_capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Returns an iterator over the events from oldest to newest.
    ///
    /// This iteration order matches the typical display layout where
    /// older events appear at the top of the list.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
    ///
    /// let mut buffer = EventBuffer::new(10);
    /// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "First".into()));
    /// buffer.push(DisplayEvent::new("evt_2".into(), DisplayEventType::Tool, "Second".into()));
    ///
    /// let ids: Vec<_> = buffer.iter().map(|e| e.id.as_str()).collect();
    /// assert_eq!(ids, vec!["evt_1", "evt_2"]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &DisplayEvent> {
        self.events.iter()
    }

    /// Returns the current number of events in the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::EventBuffer;
    ///
    /// let buffer = EventBuffer::new(100);
    /// assert_eq!(buffer.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` if the buffer contains no events.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
    ///
    /// let mut buffer = EventBuffer::new(10);
    /// assert!(buffer.is_empty());
    ///
    /// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "Test".into()));
    /// assert!(!buffer.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Removes all events from the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
    ///
    /// let mut buffer = EventBuffer::new(10);
    /// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "Test".into()));
    /// assert!(!buffer.is_empty());
    ///
    /// buffer.clear();
    /// assert!(buffer.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Returns a reference to the event at the given index.
    ///
    /// Index 0 is the oldest event, and `len() - 1` is the newest.
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based index where 0 is the oldest event
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DisplayEvent, DisplayEventType};
    ///
    /// let mut buffer = EventBuffer::new(10);
    /// buffer.push(DisplayEvent::new("evt_1".into(), DisplayEventType::Session, "First".into()));
    /// buffer.push(DisplayEvent::new("evt_2".into(), DisplayEventType::Tool, "Second".into()));
    ///
    /// assert_eq!(buffer.get(0).unwrap().id, "evt_1"); // oldest
    /// assert_eq!(buffer.get(1).unwrap().id, "evt_2"); // newest
    /// assert!(buffer.get(2).is_none()); // out of bounds
    /// ```
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&DisplayEvent> {
        self.events.get(index)
    }

    /// Returns the maximum capacity of the buffer.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::EventBuffer;
    ///
    /// let buffer = EventBuffer::new(500);
    /// assert_eq!(buffer.capacity(), 500);
    /// ```
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.max_capacity
    }
}

impl Default for EventBuffer {
    /// Creates an `EventBuffer` with the default capacity of 1000 events.
    ///
    /// This default satisfies FR-011 which specifies that the event buffer
    /// should be limited to the 1000 most recent events.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::{EventBuffer, DEFAULT_EVENT_BUFFER_CAPACITY};
    ///
    /// let buffer = EventBuffer::default();
    /// assert_eq!(buffer.capacity(), DEFAULT_EVENT_BUFFER_CAPACITY);
    /// ```
    fn default() -> Self {
        Self::new(DEFAULT_EVENT_BUFFER_CAPACITY)
    }
}

// =============================================================================
// Scroll State (FR-011)
// =============================================================================

/// Manages scroll state for the event stream display.
///
/// The `ScrollState` tracks the current scroll offset, auto-scroll mode, and
/// cached dimensions used for scroll calculations. It implements FR-011: "New
/// events MUST auto-scroll unless user has manually scrolled up."
///
/// # Scroll Model
///
/// The scroll offset represents how many events are hidden below the visible
/// area, counting from the bottom (newest events). An offset of 0 means the
/// newest events are visible at the bottom of the viewport.
///
/// ```text
/// offset = 0 (default, auto-scroll):
/// ┌─────────────────────┐
/// │ event 997           │  ← visible_height = 4
/// │ event 998           │
/// │ event 999           │
/// │ event 1000 (newest) │  ← bottom of viewport
/// └─────────────────────┘
///
/// offset = 2 (scrolled up):
/// ┌─────────────────────┐
/// │ event 995           │
/// │ event 996           │
/// │ event 997           │
/// │ event 998           │  ← 2 newer events hidden below
/// └─────────────────────┘
/// ```
///
/// # Auto-Scroll Behavior
///
/// When `auto_scroll` is `true`, new events cause the view to scroll down
/// automatically. When the user manually scrolls up (increasing offset),
/// `auto_scroll` is disabled. Scrolling to the bottom re-enables it.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::app::ScrollState;
///
/// let mut scroll = ScrollState::new(10); // 10 visible lines
/// scroll.update_total_events(100);
///
/// assert!(scroll.auto_scroll()); // Auto-scroll enabled by default
/// assert_eq!(scroll.offset(), 0);
///
/// scroll.scroll_up(); // User scrolls up
/// assert!(!scroll.auto_scroll()); // Auto-scroll disabled
/// assert_eq!(scroll.offset(), 1);
///
/// scroll.scroll_to_bottom(); // User scrolls to bottom
/// assert!(scroll.auto_scroll()); // Auto-scroll re-enabled
/// assert_eq!(scroll.offset(), 0);
/// ```
#[derive(Debug, Clone)]
pub struct ScrollState {
    /// Current scroll offset (0 = newest at bottom visible).
    ///
    /// The offset represents how many events are hidden below the visible
    /// viewport. Valid range: `0..=max(0, total_events - visible_height)`.
    offset: usize,

    /// Whether auto-scroll is enabled (scroll to newest on new events).
    ///
    /// Auto-scroll is enabled by default and disabled when the user manually
    /// scrolls up. It is re-enabled when the user scrolls to the bottom.
    auto_scroll: bool,

    /// Total events in buffer (cached for scroll calculations).
    ///
    /// This value is updated via `update_total_events` when the event buffer
    /// changes size. Used to calculate the maximum valid scroll offset.
    total_events: usize,

    /// Visible area height (set from terminal size).
    ///
    /// This value is updated via `update_visible_height` when the terminal
    /// is resized. Used to calculate the maximum valid scroll offset.
    visible_height: usize,
}

impl ScrollState {
    /// Creates a new `ScrollState` with the specified visible height.
    ///
    /// Auto-scroll is enabled by default, and the offset starts at 0 (newest
    /// events visible at the bottom).
    ///
    /// # Arguments
    ///
    /// * `visible_height` - Number of visible lines in the event stream viewport
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let scroll = ScrollState::new(20);
    /// assert!(scroll.auto_scroll());
    /// assert_eq!(scroll.offset(), 0);
    /// assert_eq!(scroll.visible_height(), 20);
    /// ```
    #[must_use]
    pub fn new(visible_height: usize) -> Self {
        Self {
            offset: 0,
            auto_scroll: true,
            total_events: 0,
            visible_height,
        }
    }

    /// Returns the current scroll offset.
    ///
    /// An offset of 0 means the newest events are visible at the bottom
    /// of the viewport.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// assert_eq!(scroll.offset(), 0);
    ///
    /// scroll.scroll_up();
    /// assert_eq!(scroll.offset(), 1);
    /// ```
    #[must_use]
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Returns whether auto-scroll is enabled.
    ///
    /// When auto-scroll is enabled, new events cause the view to scroll
    /// down automatically to keep the newest events visible.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// assert!(scroll.auto_scroll());
    ///
    /// scroll.scroll_up();
    /// assert!(!scroll.auto_scroll());
    /// ```
    #[must_use]
    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    /// Returns the total number of events.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// assert_eq!(scroll.total_events(), 0);
    ///
    /// scroll.update_total_events(100);
    /// assert_eq!(scroll.total_events(), 100);
    /// ```
    #[must_use]
    pub fn total_events(&self) -> usize {
        self.total_events
    }

    /// Returns the visible height of the viewport.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let scroll = ScrollState::new(25);
    /// assert_eq!(scroll.visible_height(), 25);
    /// ```
    #[must_use]
    pub fn visible_height(&self) -> usize {
        self.visible_height
    }

    /// Returns the maximum valid scroll offset.
    ///
    /// The maximum offset is `max(0, total_events - visible_height)`. When
    /// there are fewer events than the visible height, the max offset is 0.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// assert_eq!(scroll.max_offset(), 90);
    ///
    /// scroll.update_total_events(5); // Fewer events than visible
    /// assert_eq!(scroll.max_offset(), 0);
    /// ```
    #[must_use]
    pub fn max_offset(&self) -> usize {
        self.total_events.saturating_sub(self.visible_height)
    }

    /// Scrolls up by one line (increases offset by 1).
    ///
    /// Disables auto-scroll since this is a manual scroll action.
    /// The offset is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    ///
    /// scroll.scroll_up();
    /// assert_eq!(scroll.offset(), 1);
    /// assert!(!scroll.auto_scroll());
    /// ```
    pub fn scroll_up(&mut self) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_add(1);
        self.clamp();
    }

    /// Scrolls down by one line (decreases offset by 1).
    ///
    /// If scrolling to the bottom (offset becomes 0), re-enables auto-scroll.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// scroll.scroll_up();
    /// scroll.scroll_up();
    /// assert_eq!(scroll.offset(), 2);
    ///
    /// scroll.scroll_down();
    /// assert_eq!(scroll.offset(), 1);
    ///
    /// scroll.scroll_down();
    /// assert_eq!(scroll.offset(), 0);
    /// assert!(scroll.auto_scroll()); // Re-enabled at bottom
    /// ```
    pub fn scroll_down(&mut self) {
        self.offset = self.offset.saturating_sub(1);
        if self.offset == 0 {
            self.auto_scroll = true;
        }
    }

    /// Scrolls up by one page (increases offset by visible_height).
    ///
    /// Disables auto-scroll since this is a manual scroll action.
    /// The offset is clamped to the valid range.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    ///
    /// scroll.scroll_page_up();
    /// assert_eq!(scroll.offset(), 10);
    /// assert!(!scroll.auto_scroll());
    /// ```
    pub fn scroll_page_up(&mut self) {
        self.auto_scroll = false;
        self.offset = self.offset.saturating_add(self.visible_height);
        self.clamp();
    }

    /// Scrolls down by one page (decreases offset by visible_height).
    ///
    /// If scrolling to the bottom (offset becomes 0), re-enables auto-scroll.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// scroll.scroll_page_up();
    /// scroll.scroll_page_up();
    /// assert_eq!(scroll.offset(), 20);
    ///
    /// scroll.scroll_page_down();
    /// assert_eq!(scroll.offset(), 10);
    ///
    /// scroll.scroll_page_down();
    /// assert_eq!(scroll.offset(), 0);
    /// assert!(scroll.auto_scroll()); // Re-enabled at bottom
    /// ```
    pub fn scroll_page_down(&mut self) {
        self.offset = self.offset.saturating_sub(self.visible_height);
        if self.offset == 0 {
            self.auto_scroll = true;
        }
    }

    /// Scrolls to the top (oldest events visible).
    ///
    /// Sets offset to the maximum value and disables auto-scroll.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    ///
    /// scroll.scroll_to_top();
    /// assert_eq!(scroll.offset(), 90); // max_offset
    /// assert!(!scroll.auto_scroll());
    /// ```
    pub fn scroll_to_top(&mut self) {
        self.auto_scroll = false;
        self.offset = self.max_offset();
    }

    /// Scrolls to the bottom (newest events visible).
    ///
    /// Sets offset to 0 and re-enables auto-scroll.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// scroll.scroll_to_top();
    ///
    /// scroll.scroll_to_bottom();
    /// assert_eq!(scroll.offset(), 0);
    /// assert!(scroll.auto_scroll());
    /// ```
    pub fn scroll_to_bottom(&mut self) {
        self.offset = 0;
        self.auto_scroll = true;
    }

    /// Updates the total number of events.
    ///
    /// Call this when the event buffer size changes. The offset is clamped
    /// to ensure it remains within the valid range.
    ///
    /// # Arguments
    ///
    /// * `total` - New total number of events in the buffer
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// assert_eq!(scroll.total_events(), 100);
    /// assert_eq!(scroll.max_offset(), 90);
    /// ```
    pub fn update_total_events(&mut self, total: usize) {
        self.total_events = total;
        self.clamp();
    }

    /// Updates the visible height of the viewport.
    ///
    /// Call this when the terminal is resized. The offset is clamped
    /// to ensure it remains within the valid range.
    ///
    /// # Arguments
    ///
    /// * `height` - New visible height in lines
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// assert_eq!(scroll.max_offset(), 90);
    ///
    /// scroll.update_visible_height(20);
    /// assert_eq!(scroll.visible_height(), 20);
    /// assert_eq!(scroll.max_offset(), 80);
    /// ```
    pub fn update_visible_height(&mut self, height: usize) {
        self.visible_height = height;
        self.clamp();
    }

    /// Clamps the offset to the valid range.
    ///
    /// Ensures offset is between 0 and `max_offset()` (inclusive).
    /// Called automatically after scroll operations and dimension updates.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let mut scroll = ScrollState::new(10);
    /// scroll.update_total_events(100);
    /// scroll.scroll_to_top();
    /// assert_eq!(scroll.offset(), 90);
    ///
    /// // Reduce total events
    /// scroll.update_total_events(50);
    /// // Offset is automatically clamped
    /// assert_eq!(scroll.offset(), 40);
    /// ```
    pub fn clamp(&mut self) {
        let max = self.max_offset();
        if self.offset > max {
            self.offset = max;
        }
    }
}

impl Default for ScrollState {
    /// Creates a `ScrollState` with default values.
    ///
    /// The default state has:
    /// - `offset`: 0 (newest events visible)
    /// - `auto_scroll`: true
    /// - `total_events`: 0
    /// - `visible_height`: 0
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::app::ScrollState;
    ///
    /// let scroll = ScrollState::default();
    /// assert_eq!(scroll.offset(), 0);
    /// assert!(scroll.auto_scroll());
    /// assert_eq!(scroll.total_events(), 0);
    /// assert_eq!(scroll.visible_height(), 0);
    /// ```
    fn default() -> Self {
        Self {
            offset: 0,
            auto_scroll: true,
            total_events: 0,
            visible_height: 0,
        }
    }
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
    use tempfile::TempDir;

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
        // Verify default values
        assert!(state.session_name.is_empty());
        assert!(state.public_key.is_empty());
        assert_eq!(state.connection_status, ConnectionStatus::Disconnected);
        assert_eq!(state.stats.events_sent, 0);
        assert_eq!(state.stats.events_failed, 0);
        // Verify new fields
        assert!(state.event_buffer.is_empty());
        assert_eq!(state.scroll.total_events(), 0);
        assert!(state.scroll.auto_scroll());
    }

    #[test]
    fn dashboard_state_is_clone() {
        let state = DashboardState {
            session_name: "test-session".to_string(),
            public_key: "test-key".to_string(),
            connection_status: ConnectionStatus::Connected,
            stats: EventStats {
                events_sent: 10,
                events_failed: 2,
            },
            ..Default::default()
        };
        let cloned = state.clone();
        assert_eq!(cloned.session_name, "test-session");
        assert_eq!(cloned.public_key, "test-key");
        assert_eq!(cloned.connection_status, ConnectionStatus::Connected);
        assert_eq!(cloned.stats.events_sent, 10);
        assert_eq!(cloned.stats.events_failed, 2);
    }

    #[test]
    fn dashboard_state_is_debug() {
        let state = DashboardState {
            session_name: "my-session".to_string(),
            public_key: "my-key".to_string(),
            connection_status: ConnectionStatus::Connecting,
            stats: EventStats::default(),
            ..Default::default()
        };
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("DashboardState"));
        assert!(debug_str.contains("session_name"));
        assert!(debug_str.contains("public_key"));
        assert!(debug_str.contains("connection_status"));
        assert!(debug_str.contains("stats"));
        assert!(debug_str.contains("event_buffer"));
        assert!(debug_str.contains("scroll"));
    }

    #[test]
    fn dashboard_state_push_event_adds_to_buffer() {
        let mut state = DashboardState::default();
        assert!(state.event_buffer.is_empty());

        let event = DisplayEvent::new(
            "evt_test123".to_string(),
            DisplayEventType::Session,
            "Test session started".to_string(),
        );

        state.push_event(event);

        assert_eq!(state.event_buffer.len(), 1);
        assert_eq!(state.scroll.total_events(), 1);
    }

    #[test]
    fn dashboard_state_push_event_updates_scroll_total() {
        let mut state = DashboardState::default();

        for i in 0..5 {
            let event = DisplayEvent::new(
                format!("evt_{}", i),
                DisplayEventType::Tool,
                format!("Event {}", i),
            );
            state.push_event(event);
        }

        assert_eq!(state.event_buffer.len(), 5);
        assert_eq!(state.scroll.total_events(), 5);
    }

    #[test]
    fn dashboard_state_push_event_preserves_auto_scroll() {
        let mut state = DashboardState::default();
        assert!(state.scroll.auto_scroll());

        let event = DisplayEvent::new(
            "evt_1".to_string(),
            DisplayEventType::Activity,
            "Heartbeat".to_string(),
        );
        state.push_event(event);

        // Auto-scroll should remain enabled
        assert!(state.scroll.auto_scroll());
        // Offset should stay at 0 (showing newest)
        assert_eq!(state.scroll.offset(), 0);
    }

    // =============================================================================
    // AppState Watch Event and Resize Tests
    // =============================================================================

    #[test]
    fn app_state_handle_watch_event_converts_and_stores() {
        use crate::types::{Event, EventPayload, EventType, SessionAction};
        use uuid::Uuid;

        let mut state = AppState::new();
        assert!(state.dashboard.event_buffer.is_empty());

        let event = Event::new(
            "test-monitor".to_string(),
            EventType::Session,
            EventPayload::Session {
                session_id: Uuid::new_v4(),
                action: SessionAction::Started,
                project: "test-project".to_string(),
            },
        );

        state.handle_watch_event(event);

        assert_eq!(state.dashboard.event_buffer.len(), 1);
        assert_eq!(state.dashboard.scroll.total_events(), 1);
    }

    #[test]
    fn app_state_handle_watch_event_preserves_event_type() {
        use crate::types::{Event, EventPayload, EventType, ToolStatus};
        use uuid::Uuid;

        let mut state = AppState::new();

        let event = Event::new(
            "test-monitor".to_string(),
            EventType::Tool,
            EventPayload::Tool {
                session_id: Uuid::new_v4(),
                tool: "Read".to_string(),
                status: ToolStatus::Completed,
                context: Some("main.rs".to_string()),
                project: None,
            },
        );

        state.handle_watch_event(event);

        // Check that the event was converted with correct type
        let stored_event = state.dashboard.event_buffer.iter().next().unwrap();
        assert_eq!(stored_event.event_type, DisplayEventType::Tool);
    }

    #[test]
    fn app_state_handle_watch_event_multiple_events() {
        use crate::types::{Event, EventPayload, EventType};
        use uuid::Uuid;

        let mut state = AppState::new();

        for i in 0..10 {
            let event = Event::new(
                "test-monitor".to_string(),
                EventType::Activity,
                EventPayload::Activity {
                    session_id: Uuid::new_v4(),
                    project: Some(format!("project-{}", i)),
                },
            );
            state.handle_watch_event(event);
        }

        assert_eq!(state.dashboard.event_buffer.len(), 10);
        assert_eq!(state.dashboard.scroll.total_events(), 10);
    }

    #[test]
    fn app_state_handle_resize_updates_visible_height() {
        let mut state = AppState::new();
        assert_eq!(state.dashboard.scroll.visible_height(), 0);

        state.handle_resize(80, 24);

        assert_eq!(state.dashboard.scroll.visible_height(), 24);
    }

    #[test]
    fn app_state_handle_resize_updates_on_size_change() {
        let mut state = AppState::new();

        state.handle_resize(80, 24);
        assert_eq!(state.dashboard.scroll.visible_height(), 24);

        state.handle_resize(120, 40);
        assert_eq!(state.dashboard.scroll.visible_height(), 40);

        state.handle_resize(60, 10);
        assert_eq!(state.dashboard.scroll.visible_height(), 10);
    }

    #[test]
    fn app_state_handle_resize_clamps_scroll_offset() {
        use crate::types::{Event, EventPayload, EventType};
        use uuid::Uuid;

        let mut state = AppState::new();

        // Add some events
        for i in 0..20 {
            let event = Event::new(
                "test".to_string(),
                EventType::Activity,
                EventPayload::Activity {
                    session_id: Uuid::new_v4(),
                    project: Some(format!("proj-{}", i)),
                },
            );
            state.handle_watch_event(event);
        }

        // Set small visible height
        state.handle_resize(80, 5);
        assert_eq!(state.dashboard.scroll.visible_height(), 5);

        // Scroll to top
        state.dashboard.scroll.scroll_to_top();
        let offset_after_scroll = state.dashboard.scroll.offset();
        assert!(offset_after_scroll > 0);

        // Now resize to larger - offset should be clamped
        state.handle_resize(80, 30);
        // With 20 events and 30 visible, max offset is 0
        assert_eq!(state.dashboard.scroll.offset(), 0);
    }

    #[test]
    fn app_state_set_connection_status_updates_dashboard() {
        let mut state = AppState::new();
        assert_eq!(
            state.dashboard.connection_status,
            ConnectionStatus::Disconnected
        );

        state.set_connection_status(ConnectionStatus::Connecting);
        assert_eq!(
            state.dashboard.connection_status,
            ConnectionStatus::Connecting
        );

        state.set_connection_status(ConnectionStatus::Connected);
        assert_eq!(
            state.dashboard.connection_status,
            ConnectionStatus::Connected
        );

        state.set_connection_status(ConnectionStatus::Error);
        assert_eq!(state.dashboard.connection_status, ConnectionStatus::Error);

        state.set_connection_status(ConnectionStatus::Disconnected);
        assert_eq!(
            state.dashboard.connection_status,
            ConnectionStatus::Disconnected
        );
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
        // Clone is implemented (even though Copy is too), test it compiles
        let cloned: SetupField = Clone::clone(&field);
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
        // Clone is implemented (even though Copy is too), test it compiles
        let cloned: KeyOption = Clone::clone(&option);
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
    // Hostname Detection Tests (T070)
    // =============================================================================

    #[test]
    fn default_session_name_returns_non_empty_string() {
        let name = default_session_name();
        assert!(
            !name.is_empty(),
            "default_session_name should return a non-empty string"
        );
    }

    #[test]
    fn default_session_name_returns_hostname_or_fallback() {
        let name = default_session_name();
        // The result should be either the actual hostname or "monitor" fallback
        // We can verify it's a reasonable value (non-empty, no null bytes)
        assert!(
            !name.contains('\0'),
            "hostname should not contain null bytes"
        );
        assert!(name.len() <= 253, "hostname should be reasonable length"); // max DNS hostname
    }

    #[test]
    fn default_session_name_is_consistent() {
        // Calling it multiple times should return the same value
        let name1 = default_session_name();
        let name2 = default_session_name();
        assert_eq!(name1, name2, "default_session_name should be deterministic");
    }

    // =============================================================================
    // SetupFormState Tests (T052)
    // =============================================================================

    #[test]
    fn setup_form_state_default_uses_hostname() {
        let state = SetupFormState::default();
        // Session name should be set to hostname (non-empty)
        assert!(!state.session_name.is_empty());
        // Should match what default_session_name() returns
        assert_eq!(state.session_name, default_session_name());
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
        let original = state.session_name.clone();
        assert!(!original.is_empty()); // Now defaults to hostname

        state.session_name = "modified-session".to_string();
        assert_eq!(state.session_name, "modified-session");
        assert_ne!(state.session_name, original);
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

    // =============================================================================
    // complete_setup() Tests (T066, T068)
    // =============================================================================

    #[test]
    fn complete_setup_transitions_to_dashboard_with_key_generation() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "my-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        assert!(state.is_setup());
        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert!(state.is_dashboard());
    }

    #[test]
    fn complete_setup_validates_session_name() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "valid-session-name".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert!(state.is_dashboard());
    }

    #[test]
    fn complete_setup_rejects_empty_session_name() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "".to_string();

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Session name cannot be empty");
        // Should remain on setup screen
        assert!(state.is_setup());
    }

    #[test]
    fn complete_setup_rejects_invalid_characters() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "invalid@name".to_string();

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Session name can only contain letters, numbers, hyphens, and underscores"
        );
        // Should remain on setup screen
        assert!(state.is_setup());
    }

    #[test]
    fn complete_setup_clears_error_on_success() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "valid-session".to_string();
        state.setup.session_name_error = Some("Previous error".to_string());
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert!(state.setup.session_name_error.is_none());
    }

    #[test]
    fn complete_setup_preserves_setup_state_on_failure() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "invalid name with spaces".to_string();
        state.setup.key_option = KeyOption::UseExisting;
        state.setup.focused_field = SetupField::Submit;
        state.setup.existing_keys_found = true;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());

        // All setup state should be preserved on failure
        assert!(state.is_setup());
        assert_eq!(state.setup.session_name, "invalid name with spaces");
        assert_eq!(state.setup.key_option, KeyOption::UseExisting);
        assert_eq!(state.setup.focused_field, SetupField::Submit);
        assert!(state.setup.existing_keys_found);
    }

    #[test]
    fn complete_setup_initializes_dashboard_state_with_credentials() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        // Dashboard state should be initialized with credentials
        assert_eq!(state.dashboard.session_name, "test-session");
        assert!(!state.dashboard.public_key.is_empty());
        // Public key should be base64-encoded (44 chars for 32-byte key)
        assert!(state.dashboard.public_key.len() >= 43);
    }

    #[test]
    fn complete_setup_accepts_hyphens_and_underscores() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "my-session_name".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());
        assert!(state.is_dashboard());
    }

    #[test]
    fn complete_setup_rejects_whitespace_only() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "   ".to_string();

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Session name cannot be empty");
        assert!(state.is_setup());
    }

    #[test]
    fn complete_setup_rejects_too_long_session_name() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "a".repeat(65);

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Session name must be 64 characters or less"
        );
        assert!(state.is_setup());
    }

    // =============================================================================
    // Key Generation Tests (T068)
    // =============================================================================

    #[test]
    fn complete_setup_generates_new_keys_when_selected() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        // Verify no keys exist initially
        assert!(!crate::crypto::Crypto::exists(temp_dir.path()));

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        // Keys should now exist
        assert!(crate::crypto::Crypto::exists(temp_dir.path()));
    }

    #[test]
    fn complete_setup_loads_existing_keys_when_selected() {
        let temp_dir = TempDir::new().unwrap();

        // First, generate some keys
        let original_crypto = crate::crypto::Crypto::generate();
        let original_pubkey = original_crypto.public_key_base64();
        original_crypto.save(temp_dir.path()).unwrap();

        // Now try to load them
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::UseExisting;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        // The loaded public key should match the original
        assert_eq!(state.dashboard.public_key, original_pubkey);
    }

    #[test]
    fn complete_setup_fails_when_use_existing_but_no_keys() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::UseExisting;

        // No keys exist
        assert!(!crate::crypto::Crypto::exists(temp_dir.path()));

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "No existing keys found. Please select 'Generate new key'."
        );

        // Should remain on setup screen
        assert!(state.is_setup());
    }

    #[test]
    fn complete_setup_generates_different_keys_each_time() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();

        // Generate keys in first directory
        let mut state1 = AppState::new();
        state1.setup.session_name = "session1".to_string();
        state1.setup.key_option = KeyOption::GenerateNew;
        state1.complete_setup(Some(temp_dir1.path())).unwrap();

        // Generate keys in second directory
        let mut state2 = AppState::new();
        state2.setup.session_name = "session2".to_string();
        state2.setup.key_option = KeyOption::GenerateNew;
        state2.complete_setup(Some(temp_dir2.path())).unwrap();

        // Public keys should be different
        assert_ne!(state1.dashboard.public_key, state2.dashboard.public_key);
    }

    #[test]
    fn complete_setup_trims_session_name_in_dashboard() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "  test-session  ".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        // Session name in dashboard should be trimmed
        assert_eq!(state.dashboard.session_name, "test-session");
    }

    #[test]
    fn complete_setup_stores_session_name_in_dashboard() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "my-workstation".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        assert_eq!(state.dashboard.session_name, "my-workstation");
    }

    #[test]
    fn complete_setup_overwrites_existing_keys_when_generate_new() {
        let temp_dir = TempDir::new().unwrap();

        // First, generate some keys
        let original_crypto = crate::crypto::Crypto::generate();
        let original_pubkey = original_crypto.public_key_base64();
        original_crypto.save(temp_dir.path()).unwrap();

        // Now generate new keys (should overwrite)
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        // The new public key should be different from the original
        assert_ne!(state.dashboard.public_key, original_pubkey);
    }

    #[test]
    fn complete_setup_default_connection_status_is_disconnected() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        assert_eq!(
            state.dashboard.connection_status,
            ConnectionStatus::Disconnected
        );
    }

    #[test]
    fn complete_setup_default_stats_are_zero() {
        let temp_dir = TempDir::new().unwrap();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.key_option = KeyOption::GenerateNew;

        let result = state.complete_setup(Some(temp_dir.path()));
        assert!(result.is_ok());

        assert_eq!(state.dashboard.stats.events_sent, 0);
        assert_eq!(state.dashboard.stats.events_failed, 0);
    }

    // =============================================================================
    // Key Detection Tests (T072)
    // =============================================================================

    #[test]
    fn detect_existing_keys_returns_false_for_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        // Empty directory should have no keys
        assert!(!detect_existing_keys_in_dir(temp_dir.path()));
    }

    #[test]
    fn detect_existing_keys_returns_true_when_keys_exist() {
        let temp_dir = TempDir::new().unwrap();

        // Generate and save keys
        let crypto = crate::crypto::Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Now the function should return true
        assert!(detect_existing_keys_in_dir(temp_dir.path()));
    }

    #[test]
    fn detect_existing_keys_returns_false_for_nonexistent_dir() {
        let path = std::path::Path::new("/nonexistent/path/that/does/not/exist");
        assert!(!detect_existing_keys_in_dir(path));
    }

    #[test]
    fn detect_existing_keys_in_dir_is_consistent() {
        let temp_dir = TempDir::new().unwrap();

        // Before keys exist
        assert!(!detect_existing_keys_in_dir(temp_dir.path()));
        assert!(!detect_existing_keys_in_dir(temp_dir.path()));

        // Generate keys
        let crypto = crate::crypto::Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // After keys exist
        assert!(detect_existing_keys_in_dir(temp_dir.path()));
        assert!(detect_existing_keys_in_dir(temp_dir.path()));
    }

    // =============================================================================
    // setup_form_with_detected_defaults Tests (T072 - FR-004)
    // =============================================================================

    #[test]
    fn setup_form_with_detected_defaults_uses_existing_when_keys_present() {
        let temp_dir = TempDir::new().unwrap();

        // Generate and save keys
        let crypto = crate::crypto::Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // Create form state with detected defaults
        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());

        // Should default to UseExisting since keys exist (FR-004)
        assert_eq!(form_state.key_option, KeyOption::UseExisting);
        assert!(form_state.existing_keys_found);
    }

    #[test]
    fn setup_form_with_detected_defaults_generates_new_when_no_keys() {
        let temp_dir = TempDir::new().unwrap();

        // No keys in this directory
        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());

        // Should default to GenerateNew since no keys exist (FR-004)
        assert_eq!(form_state.key_option, KeyOption::GenerateNew);
        assert!(!form_state.existing_keys_found);
    }

    #[test]
    fn setup_form_with_detected_defaults_sets_session_name_to_hostname() {
        let temp_dir = TempDir::new().unwrap();

        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());

        // Session name should be set to hostname (FR-003)
        assert!(!form_state.session_name.is_empty());
        assert_eq!(form_state.session_name, default_session_name());
    }

    #[test]
    fn setup_form_with_detected_defaults_has_no_error() {
        let temp_dir = TempDir::new().unwrap();

        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());

        // Should have no initial error
        assert!(form_state.session_name_error.is_none());
    }

    #[test]
    fn setup_form_with_detected_defaults_focuses_session_name() {
        let temp_dir = TempDir::new().unwrap();

        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());

        // Should focus the session name field initially
        assert_eq!(form_state.focused_field, SetupField::SessionName);
    }

    #[test]
    fn setup_form_with_detected_defaults_existing_keys_found_matches_key_option() {
        let temp_dir = TempDir::new().unwrap();

        // Without keys
        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());
        assert!(!form_state.existing_keys_found);
        assert_eq!(form_state.key_option, KeyOption::GenerateNew);

        // Generate keys
        let crypto = crate::crypto::Crypto::generate();
        crypto.save(temp_dir.path()).unwrap();

        // With keys
        let form_state = setup_form_with_detected_defaults_in_dir(temp_dir.path());
        assert!(form_state.existing_keys_found);
        assert_eq!(form_state.key_option, KeyOption::UseExisting);
    }

    #[test]
    fn setup_form_with_detected_defaults_for_nonexistent_dir() {
        let path = std::path::Path::new("/nonexistent/path/that/does/not/exist");

        let form_state = setup_form_with_detected_defaults_in_dir(path);

        // Should default to GenerateNew for nonexistent directory
        assert_eq!(form_state.key_option, KeyOption::GenerateNew);
        assert!(!form_state.existing_keys_found);
        // Session name should still be set
        assert!(!form_state.session_name.is_empty());
    }

    // =============================================================================
    // ScrollState Tests (T097)
    // =============================================================================

    #[test]
    fn scroll_state_new_creates_with_visible_height() {
        let scroll = ScrollState::new(20);
        assert_eq!(scroll.visible_height(), 20);
        assert_eq!(scroll.offset(), 0);
        assert!(scroll.auto_scroll());
        assert_eq!(scroll.total_events(), 0);
    }

    #[test]
    fn scroll_state_default_has_zero_dimensions() {
        let scroll = ScrollState::default();
        assert_eq!(scroll.offset(), 0);
        assert!(scroll.auto_scroll());
        assert_eq!(scroll.total_events(), 0);
        assert_eq!(scroll.visible_height(), 0);
    }

    #[test]
    fn scroll_state_is_debug() {
        let scroll = ScrollState::new(10);
        let debug_str = format!("{:?}", scroll);
        assert!(debug_str.contains("ScrollState"));
        assert!(debug_str.contains("offset"));
        assert!(debug_str.contains("auto_scroll"));
    }

    #[test]
    fn scroll_state_is_clone() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_up();

        let cloned = scroll.clone();
        assert_eq!(cloned.offset(), scroll.offset());
        assert_eq!(cloned.auto_scroll(), scroll.auto_scroll());
        assert_eq!(cloned.total_events(), scroll.total_events());
        assert_eq!(cloned.visible_height(), scroll.visible_height());
    }

    #[test]
    fn scroll_state_max_offset_with_more_events_than_visible() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        assert_eq!(scroll.max_offset(), 90);
    }

    #[test]
    fn scroll_state_max_offset_with_fewer_events_than_visible() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(5);
        assert_eq!(scroll.max_offset(), 0);
    }

    #[test]
    fn scroll_state_max_offset_with_equal_events_and_visible() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(10);
        assert_eq!(scroll.max_offset(), 0);
    }

    #[test]
    fn scroll_state_scroll_up_increases_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_up();
        assert_eq!(scroll.offset(), 1);
    }

    #[test]
    fn scroll_state_scroll_up_disables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        assert!(scroll.auto_scroll());

        scroll.scroll_up();
        assert!(!scroll.auto_scroll());
    }

    #[test]
    fn scroll_state_scroll_up_clamps_to_max() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(15);
        // max_offset is 5

        for _ in 0..10 {
            scroll.scroll_up();
        }
        assert_eq!(scroll.offset(), 5); // Clamped to max
    }

    #[test]
    fn scroll_state_scroll_down_decreases_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_up();
        scroll.scroll_up();
        assert_eq!(scroll.offset(), 2);

        scroll.scroll_down();
        assert_eq!(scroll.offset(), 1);
    }

    #[test]
    fn scroll_state_scroll_down_clamps_to_zero() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_down(); // Already at 0
        assert_eq!(scroll.offset(), 0);
    }

    #[test]
    fn scroll_state_scroll_down_to_bottom_enables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_up();
        scroll.scroll_up();
        assert!(!scroll.auto_scroll());

        scroll.scroll_down();
        assert!(!scroll.auto_scroll()); // Still at offset 1

        scroll.scroll_down();
        assert!(scroll.auto_scroll()); // Now at offset 0
    }

    #[test]
    fn scroll_state_scroll_page_up_increases_by_visible_height() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_page_up();
        assert_eq!(scroll.offset(), 10);
    }

    #[test]
    fn scroll_state_scroll_page_up_disables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_page_up();
        assert!(!scroll.auto_scroll());
    }

    #[test]
    fn scroll_state_scroll_page_up_clamps_to_max() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(25);
        // max_offset is 15

        scroll.scroll_page_up();
        scroll.scroll_page_up();
        assert_eq!(scroll.offset(), 15); // Clamped to max
    }

    #[test]
    fn scroll_state_scroll_page_down_decreases_by_visible_height() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_page_up();
        scroll.scroll_page_up();
        assert_eq!(scroll.offset(), 20);

        scroll.scroll_page_down();
        assert_eq!(scroll.offset(), 10);
    }

    #[test]
    fn scroll_state_scroll_page_down_clamps_to_zero() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_up();
        scroll.scroll_up();
        assert_eq!(scroll.offset(), 2);

        scroll.scroll_page_down();
        assert_eq!(scroll.offset(), 0);
    }

    #[test]
    fn scroll_state_scroll_page_down_to_bottom_enables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_page_up();
        assert!(!scroll.auto_scroll());

        scroll.scroll_page_down();
        assert!(scroll.auto_scroll()); // Now at offset 0
    }

    #[test]
    fn scroll_state_scroll_to_top_sets_max_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_to_top();
        assert_eq!(scroll.offset(), 90);
    }

    #[test]
    fn scroll_state_scroll_to_top_disables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);

        scroll.scroll_to_top();
        assert!(!scroll.auto_scroll());
    }

    #[test]
    fn scroll_state_scroll_to_bottom_sets_zero_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_to_top();

        scroll.scroll_to_bottom();
        assert_eq!(scroll.offset(), 0);
    }

    #[test]
    fn scroll_state_scroll_to_bottom_enables_auto_scroll() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_to_top();
        assert!(!scroll.auto_scroll());

        scroll.scroll_to_bottom();
        assert!(scroll.auto_scroll());
    }

    #[test]
    fn scroll_state_update_total_events_clamps_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_to_top();
        assert_eq!(scroll.offset(), 90);

        scroll.update_total_events(50);
        // max_offset is now 40, offset should be clamped
        assert_eq!(scroll.offset(), 40);
    }

    #[test]
    fn scroll_state_update_visible_height_clamps_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_to_top();
        assert_eq!(scroll.offset(), 90);

        scroll.update_visible_height(50);
        // max_offset is now 50, offset should be clamped
        assert_eq!(scroll.offset(), 50);
    }

    #[test]
    fn scroll_state_clamp_reduces_excessive_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_to_top();
        assert_eq!(scroll.offset(), 90);

        // Simulate reducing events without going through update_total_events
        scroll.total_events = 20;
        scroll.clamp();
        assert_eq!(scroll.offset(), 10); // max_offset is now 10
    }

    #[test]
    fn scroll_state_clamp_does_not_change_valid_offset() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(100);
        scroll.scroll_up();
        scroll.scroll_up();
        assert_eq!(scroll.offset(), 2);

        scroll.clamp();
        assert_eq!(scroll.offset(), 2); // Still valid, unchanged
    }

    #[test]
    fn scroll_state_auto_scroll_maintained_on_new_events() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(50);
        assert!(scroll.auto_scroll());
        assert_eq!(scroll.offset(), 0);

        // Simulate new events arriving
        scroll.update_total_events(60);
        assert!(scroll.auto_scroll()); // Still enabled
        assert_eq!(scroll.offset(), 0); // Still at bottom
    }

    #[test]
    fn scroll_state_offset_preserved_when_scrolled_up_and_new_events() {
        let mut scroll = ScrollState::new(10);
        scroll.update_total_events(50);
        scroll.scroll_up();
        scroll.scroll_up();
        assert_eq!(scroll.offset(), 2);
        assert!(!scroll.auto_scroll());

        // Simulate new events arriving
        scroll.update_total_events(60);
        assert_eq!(scroll.offset(), 2); // Preserved
        assert!(!scroll.auto_scroll()); // Still disabled
    }

    #[test]
    fn scroll_state_zero_visible_height_max_offset_equals_total() {
        let mut scroll = ScrollState::new(0);
        scroll.update_total_events(100);
        assert_eq!(scroll.max_offset(), 100);
    }

    #[test]
    fn scroll_state_zero_total_events_max_offset_is_zero() {
        let scroll = ScrollState::new(10);
        assert_eq!(scroll.max_offset(), 0);
    }
}
