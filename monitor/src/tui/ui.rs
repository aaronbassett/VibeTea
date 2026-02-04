//! UI rendering functions for the VibeTea Monitor TUI.
//!
//! This module provides the main rendering functions that compose widgets into
//! complete screens. It acts as the "View" layer in the MVC architecture,
//! dispatching to appropriate screen renderers based on the current application state.
//!
//! # Architecture
//!
//! The UI module follows a dispatch pattern:
//!
//! ```text
//! render() --> match state.screen {
//!     Setup     --> render_setup_screen()
//!     Dashboard --> render_dashboard_screen()
//! }
//! ```
//!
//! Each screen renderer is responsible for composing the appropriate widgets
//! and laying them out within the available terminal space.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::ui::render;
//! use vibetea_monitor::tui::AppState;
//!
//! let state = AppState::new();
//! terminal.draw(|frame| {
//!     render(frame, &state);
//! })?;
//! ```

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::tui::app::{AppState, Credentials, Screen};
use crate::tui::widgets::{
    header_height, CredentialsWidget, EventStreamWidget, HeaderWidget, SetupFormWidget,
    StatsFooterWidget, CREDENTIALS_HEIGHT, STATS_FOOTER_HEIGHT,
};

/// Renders the appropriate screen based on the current application state.
///
/// This is the main entry point for UI rendering. It dispatches to the correct
/// screen rendering function based on `state.screen`:
///
/// - [`Screen::Setup`] renders the setup form for initial configuration
/// - [`Screen::Dashboard`] renders the main monitoring dashboard (placeholder)
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into
/// * `state` - The current application state containing screen and form data
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::ui::render;
/// use vibetea_monitor::tui::AppState;
///
/// let state = AppState::new();
/// terminal.draw(|frame| {
///     render(frame, &state);
/// })?;
/// ```
pub fn render(frame: &mut Frame, state: &AppState) {
    match state.screen {
        Screen::Setup => render_setup_screen(frame, state),
        Screen::Dashboard => render_dashboard_screen(frame, state),
    }
}

/// Renders the setup screen with the configuration form.
///
/// This function draws the setup form widget centered on the screen,
/// using the form state, theme, and symbols from the application state.
/// The setup screen is displayed when the application first starts,
/// allowing users to configure:
///
/// - Session name identifier (FR-003)
/// - Key generation option (FR-004)
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into
/// * `state` - The application state containing setup form values and theme
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::ui::render_setup_screen;
/// use vibetea_monitor::tui::AppState;
///
/// let state = AppState::new();
/// terminal.draw(|frame| {
///     render_setup_screen(frame, &state);
/// })?;
/// ```
pub fn render_setup_screen(frame: &mut Frame, state: &AppState) {
    let setup_widget = SetupFormWidget::new(&state.setup, &state.theme, &state.symbols);
    frame.render_widget(setup_widget, frame.area());
}

/// Renders the main dashboard screen with header, event stream, credentials, and stats footer.
///
/// This function renders the complete dashboard view with a four-section layout:
///
/// 1. **Header** - Shows "VibeTea" branding and connection status indicator
/// 2. **Event stream** - Scrollable list of session events with timestamps and icons
/// 3. **Credentials** - Session name and public key for server configuration
/// 4. **Stats Footer** - Real-time event statistics (total events, events per second, uptime)
///
/// The layout adapts to terminal size using ratatui's Layout system:
/// - Header gets a fixed height based on terminal width
/// - Credentials panel gets a fixed height of 4 rows
/// - Stats footer gets a fixed height of 3 rows
/// - Event stream fills the remaining vertical space
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into
/// * `state` - The application state containing dashboard data
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::ui::render_dashboard_screen;
/// use vibetea_monitor::tui::{AppState, Screen};
///
/// let mut state = AppState::new();
/// state.screen = Screen::Dashboard;
/// state.dashboard.session_name = "my-macbook".to_string();
/// state.dashboard.public_key = "AAAAC3NzaC1lZDI1NTE5...".to_string();
/// terminal.draw(|frame| {
///     render_dashboard_screen(frame, &state);
/// })?;
/// ```
fn render_dashboard_screen(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // Calculate the header height based on terminal width
    let h_height = header_height(area.width);

    // Create the four-section vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(h_height),            // Header
            Constraint::Min(1),                      // Event stream (fills remaining space)
            Constraint::Length(CREDENTIALS_HEIGHT),  // Credentials
            Constraint::Length(STATS_FOOTER_HEIGHT), // Stats footer
        ])
        .split(area);

    // Render the header widget
    let header_widget = HeaderWidget::new(
        state.dashboard.connection_status,
        &state.theme,
        &state.symbols,
    )
    .with_session_name(&state.dashboard.session_name);
    frame.render_widget(header_widget, chunks[0]);

    // Render the event stream widget
    // Calculate visible height for the event stream (excluding borders if any)
    let event_visible_height = chunks[1].height;
    let event_stream_widget = EventStreamWidget::new(
        &state.dashboard.event_buffer,
        &state.theme,
        &state.symbols,
        state.dashboard.scroll.offset(),
        event_visible_height,
    );
    frame.render_widget(event_stream_widget, chunks[1]);

    // Create credentials from dashboard state and render the credentials widget
    let credentials = Credentials {
        session_name: state.dashboard.session_name.clone(),
        public_key: state.dashboard.public_key.clone(),
    };
    let credentials_widget = CredentialsWidget::new(&credentials, &state.theme);
    frame.render_widget(credentials_widget, chunks[2]);

    // Render the stats footer widget
    let stats_footer_widget = StatsFooterWidget::new(&state.dashboard.stats, &state.theme);
    frame.render_widget(stats_footer_widget, chunks[3]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    /// Creates a test terminal with a standard 80x24 size.
    fn create_test_terminal() -> Terminal<TestBackend> {
        let backend = TestBackend::new(80, 24);
        Terminal::new(backend).unwrap()
    }

    /// Creates a test terminal with custom dimensions.
    fn create_test_terminal_with_size(width: u16, height: u16) -> Terminal<TestBackend> {
        let backend = TestBackend::new(width, height);
        Terminal::new(backend).unwrap()
    }

    // =========================================================================
    // render_setup_screen Tests
    // =========================================================================

    #[test]
    fn render_setup_screen_does_not_panic() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();
        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen should not fail");
    }

    #[test]
    fn render_setup_screen_with_custom_state() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.setup.session_name = "test-session".to_string();
        state.setup.session_name_error = Some("Test error".to_string());

        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen with custom state should not fail");
    }

    #[test]
    fn render_setup_screen_with_small_terminal() {
        let mut terminal = create_test_terminal_with_size(40, 12);
        let state = AppState::new();
        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen in small terminal should not fail");
    }

    #[test]
    fn render_setup_screen_with_large_terminal() {
        let mut terminal = create_test_terminal_with_size(200, 60);
        let state = AppState::new();
        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen in large terminal should not fail");
    }

    #[test]
    fn render_setup_screen_with_monochrome_theme() {
        use crate::tui::app::Theme;

        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.theme = Theme::monochrome();

        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen with monochrome theme should not fail");
    }

    #[test]
    fn render_setup_screen_with_ascii_symbols() {
        use crate::tui::app::ASCII_SYMBOLS;

        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.symbols = ASCII_SYMBOLS;

        terminal
            .draw(|f| render_setup_screen(f, &state))
            .expect("Drawing setup screen with ASCII symbols should not fail");
    }

    // =========================================================================
    // render_dashboard_screen Tests
    // =========================================================================

    #[test]
    fn render_dashboard_screen_does_not_panic() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();
        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing dashboard screen should not fail");
    }

    #[test]
    fn render_dashboard_screen_with_small_terminal() {
        let mut terminal = create_test_terminal_with_size(40, 12);
        let state = AppState::new();
        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing dashboard screen in small terminal should not fail");
    }

    #[test]
    fn render_dashboard_screen_with_large_terminal() {
        let mut terminal = create_test_terminal_with_size(200, 60);
        let state = AppState::new();
        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing dashboard screen in large terminal should not fail");
    }

    #[test]
    fn render_dashboard_screen_contains_credentials_section() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();

        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing should not fail");

        // Get the buffer content
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Credentials"),
            "Dashboard should show Credentials section"
        );
    }

    #[test]
    fn render_dashboard_screen_contains_header_title() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();

        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing should not fail");

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("VibeTea"),
            "Dashboard should show VibeTea title in header"
        );
    }

    #[test]
    fn render_dashboard_screen_contains_stats_footer() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();

        terminal
            .draw(|f| render_dashboard_screen(f, &state))
            .expect("Drawing should not fail");

        // Get the buffer content
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Statistics"),
            "Dashboard should show Statistics section in footer"
        );
    }

    // =========================================================================
    // render (Main Dispatch) Tests
    // =========================================================================

    #[test]
    fn render_dispatches_to_setup_screen() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.screen = Screen::Setup;

        terminal
            .draw(|f| render(f, &state))
            .expect("Drawing via render() should not fail");

        // Verify setup screen is rendered by checking for its title
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("VibeTea Monitor Setup"),
            "Should dispatch to setup screen"
        );
    }

    #[test]
    fn render_dispatches_to_dashboard_screen() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.screen = Screen::Dashboard;

        terminal
            .draw(|f| render(f, &state))
            .expect("Drawing via render() should not fail");

        // Verify dashboard screen is rendered by checking for credentials section
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Credentials"),
            "Should dispatch to dashboard screen (Credentials visible)"
        );
    }

    #[test]
    fn render_default_state_shows_setup() {
        let mut terminal = create_test_terminal();
        let state = AppState::new(); // Default state should be Setup screen

        terminal
            .draw(|f| render(f, &state))
            .expect("Drawing via render() should not fail");

        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("VibeTea Monitor Setup"),
            "Default state should show setup screen"
        );
    }

    #[test]
    fn render_handles_screen_transition() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();

        // Start with Setup
        state.screen = Screen::Setup;
        terminal
            .draw(|f| render(f, &state))
            .expect("Drawing setup should not fail");

        // Transition to Dashboard
        state.screen = Screen::Dashboard;
        terminal
            .draw(|f| render(f, &state))
            .expect("Drawing dashboard should not fail");

        // Verify dashboard is now shown by checking for Credentials section
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Credentials"),
            "After transition should show dashboard (Credentials visible)"
        );
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn render_with_minimum_terminal_size() {
        let mut terminal = create_test_terminal_with_size(20, 5);
        let state = AppState::new();

        // Should not panic even with tiny terminal
        terminal
            .draw(|f| render(f, &state))
            .expect("Should handle minimum terminal size");
    }

    #[test]
    fn render_with_very_wide_terminal() {
        let mut terminal = create_test_terminal_with_size(300, 24);
        let state = AppState::new();

        terminal
            .draw(|f| render(f, &state))
            .expect("Should handle very wide terminal");
    }

    #[test]
    fn render_with_very_tall_terminal() {
        let mut terminal = create_test_terminal_with_size(80, 100);
        let state = AppState::new();

        terminal
            .draw(|f| render(f, &state))
            .expect("Should handle very tall terminal");
    }

    #[test]
    fn render_multiple_times_consecutively() {
        let mut terminal = create_test_terminal();
        let state = AppState::new();

        // Render multiple times - should be idempotent and not cause issues
        for _ in 0..10 {
            terminal
                .draw(|f| render(f, &state))
                .expect("Multiple renders should not fail");
        }
    }

    #[test]
    fn render_with_all_setup_fields_focused() {
        use crate::tui::app::SetupField;

        let mut terminal = create_test_terminal();
        let mut state = AppState::new();

        // Test each focus state
        for field in [
            SetupField::SessionName,
            SetupField::KeyOption,
            SetupField::Submit,
        ] {
            state.setup.focused_field = field;
            terminal
                .draw(|f| render(f, &state))
                .expect("Should render with any focused field");
        }
    }

    #[test]
    fn render_with_long_session_name() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.setup.session_name = "a".repeat(64); // Max length per FR-026

        terminal
            .draw(|f| render(f, &state))
            .expect("Should handle maximum length session name");
    }

    #[test]
    fn render_with_validation_error() {
        let mut terminal = create_test_terminal();
        let mut state = AppState::new();
        state.setup.session_name_error = Some("This is a validation error message".to_string());

        terminal
            .draw(|f| render(f, &state))
            .expect("Should render with validation error");
    }
}
