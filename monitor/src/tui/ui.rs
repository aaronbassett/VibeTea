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
    layout::Alignment,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::app::{AppState, Screen};
use crate::tui::widgets::SetupFormWidget;

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

/// Renders the main dashboard screen (placeholder).
///
/// This function renders a placeholder dashboard view. The full implementation
/// will be added in Phase 4 (User Story 2) and will include:
///
/// - Header with connection status and server info
/// - Event stream showing real-time session events
/// - Statistics footer with event counts and keybindings
/// - Credentials panel with device public key
///
/// # TODO
///
/// This will be fully implemented in Phase 4 (User Story 2).
///
/// # Arguments
///
/// * `frame` - The ratatui frame to render into
/// * `_state` - The application state (unused in placeholder)
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::ui::render_dashboard_screen;
/// use vibetea_monitor::tui::{AppState, Screen};
///
/// let mut state = AppState::new();
/// state.screen = Screen::Dashboard;
/// terminal.draw(|frame| {
///     render_dashboard_screen(frame, &state);
/// })?;
/// ```
fn render_dashboard_screen(frame: &mut Frame, _state: &AppState) {
    let placeholder = Paragraph::new("Dashboard coming soon...")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" VibeTea Monitor "),
        );
    frame.render_widget(placeholder, frame.area());
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
    fn render_dashboard_screen_contains_placeholder_text() {
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
            content.contains("Dashboard coming soon"),
            "Dashboard should show placeholder text"
        );
    }

    #[test]
    fn render_dashboard_screen_contains_title() {
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
            content.contains("VibeTea Monitor"),
            "Dashboard should show title"
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

        // Verify dashboard screen is rendered by checking for placeholder text
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Dashboard coming soon"),
            "Should dispatch to dashboard screen"
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

        // Verify dashboard is now shown
        let buffer = terminal.backend().buffer();
        let content: String = buffer
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        assert!(
            content.contains("Dashboard coming soon"),
            "After transition should show dashboard"
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
