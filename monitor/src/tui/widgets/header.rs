//! Header and connection status widgets for the VibeTea Monitor TUI.
//!
//! This module provides widgets for displaying the header bar with connection status
//! and branding. The widgets adapt to terminal width for graceful degradation on
//! narrow terminals.
//!
//! # Requirements Compliance
//!
//! - **T121**: Connection status widget displays status with icon and text
//! - **T123**: Color-blind safe indicators (symbols + colors)
//! - **T125**: Header widget combines logo and status
//! - **T127**: Graceful degradation for narrow terminals
//! - **US8**: Display stylized VibeTea ASCII logo in header
//!
//! # Layout Modes
//!
//! The header widget adapts based on terminal width:
//!
//! - **Wide (>= 80 columns)**: Shows full ASCII art logo on left, connection status on right
//! - **Medium (>= 30 columns)**: Shows compact logo on left, connection status on right
//! - **Narrow (< 30 columns)**: Shows text logo and status in compact form
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::{ConnectionStatusWidget, HeaderWidget};
//! use vibetea_monitor::tui::app::{ConnectionStatus, Theme, Symbols};
//!
//! let theme = Theme::default();
//! let symbols = Symbols::detect();
//!
//! // Render just the connection status
//! let status_widget = ConnectionStatusWidget::new(
//!     ConnectionStatus::Connected,
//!     &theme,
//!     &symbols,
//! );
//! frame.render_widget(status_widget, status_area);
//!
//! // Render the full header
//! let header_widget = HeaderWidget::new(
//!     ConnectionStatus::Connected,
//!     &theme,
//!     &symbols,
//! );
//! frame.render_widget(header_widget, header_area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use super::logo::{logo_height, LogoWidget};
use crate::tui::app::{ConnectionStatus, Symbols, Theme};

/// Width threshold for switching between wide and narrow layouts.
const WIDE_LAYOUT_THRESHOLD: u16 = 80;

/// Minimum height for the header (accounts for border + content).
const MIN_HEADER_HEIGHT: u16 = 3;

/// Returns the height needed for the header widget.
///
/// This function calculates the required height based on terminal width,
/// accounting for the logo height plus borders.
///
/// # Arguments
///
/// * `area_width` - The available width for determining logo variant
///
/// # Returns
///
/// The number of rows needed for the header, including borders.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::header_height;
///
/// let height = header_height(80);  // Returns 7 (5 for full logo + 2 for borders)
/// let height = header_height(40);  // Returns 3 (1 for compact logo + 2 for borders)
/// ```
#[must_use]
pub fn header_height(area_width: u16) -> u16 {
    // Logo height + 2 for top and bottom borders
    let logo_h = logo_height(area_width);
    let total = logo_h + 2;
    total.max(MIN_HEADER_HEIGHT)
}

/// Widget for displaying the connection status indicator.
///
/// Renders the connection status with an appropriate symbol and text label.
/// The widget uses color-blind safe design by combining both symbols and colors
/// to indicate status.
///
/// # Status Display
///
/// | Status | Symbol | Text | Color |
/// |--------|--------|------|-------|
/// | Connected | ● or [*] | "Connected" | Green |
/// | Disconnected | ○ or [ ] | "Disconnected" | Red |
/// | Connecting | ◔ or [.] | "Connecting..." | Yellow |
/// | Error | ○ or [ ] | "Error" | Red |
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::ConnectionStatusWidget;
/// use vibetea_monitor::tui::app::{ConnectionStatus, Theme, Symbols};
///
/// let theme = Theme::default();
/// let symbols = Symbols::detect();
///
/// let widget = ConnectionStatusWidget::new(
///     ConnectionStatus::Connected,
///     &theme,
///     &symbols,
/// );
/// frame.render_widget(widget, area);
/// ```
#[derive(Debug)]
pub struct ConnectionStatusWidget<'a> {
    /// The current connection status to display.
    status: ConnectionStatus,
    /// Reference to the theme for styling.
    theme: &'a Theme,
    /// Reference to the symbol set (unicode or ASCII).
    symbols: &'a Symbols,
}

impl<'a> ConnectionStatusWidget<'a> {
    /// Creates a new `ConnectionStatusWidget`.
    ///
    /// # Arguments
    ///
    /// * `status` - The connection status to display
    /// * `theme` - Theme configuration for colors and styles
    /// * `symbols` - Symbol set (unicode or ASCII) for status icons
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::ConnectionStatusWidget;
    /// use vibetea_monitor::tui::app::{ConnectionStatus, Theme, Symbols};
    ///
    /// let theme = Theme::default();
    /// let symbols = Symbols::detect();
    ///
    /// let widget = ConnectionStatusWidget::new(
    ///     ConnectionStatus::Connected,
    ///     &theme,
    ///     &symbols,
    /// );
    /// ```
    #[must_use]
    pub fn new(status: ConnectionStatus, theme: &'a Theme, symbols: &'a Symbols) -> Self {
        Self {
            status,
            theme,
            symbols,
        }
    }

    /// Returns the symbol for the current status.
    fn status_symbol(&self) -> &'static str {
        match self.status {
            ConnectionStatus::Connected => self.symbols.connected,
            ConnectionStatus::Disconnected => self.symbols.disconnected,
            ConnectionStatus::Connecting => self.symbols.connecting,
            ConnectionStatus::Error => self.symbols.disconnected, // Use disconnected symbol for error
        }
    }

    /// Returns the text label for the current status.
    fn status_text(&self) -> &'static str {
        match self.status {
            ConnectionStatus::Connected => "Connected",
            ConnectionStatus::Disconnected => "Disconnected",
            ConnectionStatus::Connecting => "Connecting...",
            ConnectionStatus::Error => "Error",
        }
    }

    /// Returns the style for the current status.
    fn status_style(&self) -> Style {
        match self.status {
            ConnectionStatus::Connected => self.theme.status_connected,
            ConnectionStatus::Disconnected => self.theme.status_disconnected,
            ConnectionStatus::Connecting => self.theme.status_connecting,
            ConnectionStatus::Error => self.theme.status_disconnected, // Use disconnected style for error
        }
    }

    /// Creates a styled line for the connection status.
    fn status_line(&self) -> Line<'a> {
        let style = self.status_style();
        Line::from(vec![
            Span::styled(self.status_symbol(), style),
            Span::styled(" ", Style::default()),
            Span::styled(self.status_text(), style),
        ])
    }

    /// Returns the display width of the status (symbol + space + text).
    #[must_use]
    pub fn display_width(&self) -> usize {
        let symbol_width = self.status_symbol().chars().count();
        let text_width = self.status_text().len();
        symbol_width + 1 + text_width // symbol + space + text
    }
}

impl Widget for ConnectionStatusWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let line = self.status_line();

        // Render the line, right-aligned if there's extra space
        let text_width = self.display_width() as u16;
        let x = if area.width >= text_width {
            area.x + area.width - text_width
        } else {
            area.x
        };

        // Render each span
        let mut current_x = x;
        for span in line.spans {
            let text = span.content.as_ref();
            let remaining = (area.x + area.width).saturating_sub(current_x) as usize;
            if remaining == 0 {
                break;
            }
            // Use character count for display width, not byte length
            let char_count = text.chars().count();
            let text_to_render = if char_count > remaining {
                // Truncate by characters, not bytes
                text.chars().take(remaining).collect::<String>()
            } else {
                text.to_string()
            };
            let display_width = text_to_render.chars().count() as u16;
            buf.set_string(current_x, area.y, &text_to_render, span.style);
            current_x += display_width;
        }
    }
}

/// Widget for rendering the combined header with branding and connection status.
///
/// The header adapts its layout based on terminal width:
///
/// - **Wide mode (>= 80 columns)**: Shows "VibeTea" text on the left side of the
///   header with the connection status aligned to the right.
///
/// - **Narrow mode (< 80 columns)**: Shows a compact layout with "VibeTea" text
///   and connection status.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::HeaderWidget;
/// use vibetea_monitor::tui::app::{ConnectionStatus, Theme, Symbols};
///
/// let theme = Theme::default();
/// let symbols = Symbols::detect();
///
/// let widget = HeaderWidget::new(
///     ConnectionStatus::Connected,
///     &theme,
///     &symbols,
/// );
/// frame.render_widget(widget, header_area);
/// ```
#[derive(Debug)]
pub struct HeaderWidget<'a> {
    /// The current connection status.
    status: ConnectionStatus,
    /// Reference to the theme for styling.
    theme: &'a Theme,
    /// Reference to the symbol set.
    symbols: &'a Symbols,
    /// Optional session name to display.
    session_name: Option<&'a str>,
}

impl<'a> HeaderWidget<'a> {
    /// Creates a new `HeaderWidget`.
    ///
    /// # Arguments
    ///
    /// * `status` - The connection status to display
    /// * `theme` - Theme configuration for colors and styles
    /// * `symbols` - Symbol set (unicode or ASCII) for icons
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::HeaderWidget;
    /// use vibetea_monitor::tui::app::{ConnectionStatus, Theme, Symbols};
    ///
    /// let theme = Theme::default();
    /// let symbols = Symbols::detect();
    ///
    /// let widget = HeaderWidget::new(
    ///     ConnectionStatus::Connected,
    ///     &theme,
    ///     &symbols,
    /// );
    /// ```
    #[must_use]
    pub fn new(status: ConnectionStatus, theme: &'a Theme, symbols: &'a Symbols) -> Self {
        Self {
            status,
            theme,
            symbols,
            session_name: None,
        }
    }

    /// Sets the session name to display in the header.
    ///
    /// # Arguments
    ///
    /// * `name` - The session name to display
    ///
    /// # Example
    ///
    /// ```ignore
    /// let widget = HeaderWidget::new(status, &theme, &symbols)
    ///     .with_session_name("my-macbook");
    /// ```
    #[must_use]
    pub fn with_session_name(mut self, name: &'a str) -> Self {
        self.session_name = Some(name);
        self
    }

    /// Returns whether the layout should be in wide mode.
    fn is_wide_layout(&self, width: u16) -> bool {
        width >= WIDE_LAYOUT_THRESHOLD
    }

    /// Renders the header in wide layout mode.
    fn render_wide(&self, area: Rect, buf: &mut Buffer) {
        // Create a bordered block (no title since logo provides branding)
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Calculate status width: symbol + space + text + padding
        let status_widget = ConnectionStatusWidget::new(self.status, self.theme, self.symbols);
        let status_display_width = status_widget.display_width() as u16;
        // Add some padding for session name if present
        let session_width = self.session_name.map_or(0, |n| n.len() as u16 + 10);
        let right_column_width = status_display_width.max(session_width) + 2;

        // Split inner area: logo fills left, status/session on right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),                    // Logo area (fills available space)
                Constraint::Length(right_column_width), // Status area (fixed width)
            ])
            .split(inner);

        // Render the logo on the left
        let logo_widget = LogoWidget::new(self.theme, self.symbols);
        logo_widget.render(chunks[0], buf);

        // Render session name and connection status on the right
        // Stack them vertically: session name on top, status below (or centered if no session)
        if let Some(name) = self.session_name {
            // Session name at top of right column
            let session_text = format!("Session: {}", name);
            let session_paragraph = Paragraph::new(session_text).style(self.theme.text_secondary);
            let session_area = Rect::new(chunks[1].x, chunks[1].y, chunks[1].width, 1);
            session_paragraph.render(session_area, buf);

            // Status below session name (or at bottom if there's room)
            let status_y = if chunks[1].height > 2 {
                chunks[1].y + chunks[1].height - 1
            } else {
                chunks[1].y + 1
            };
            let status_area = Rect::new(chunks[1].x, status_y, chunks[1].width, 1);
            status_widget.render(status_area, buf);
        } else {
            // Just status, vertically centered
            let status_y = chunks[1].y + chunks[1].height / 2;
            let status_area = Rect::new(chunks[1].x, status_y, chunks[1].width, 1);
            status_widget.render(status_area, buf);
        }
    }

    /// Renders the header in narrow layout mode.
    fn render_narrow(&self, area: Rect, buf: &mut Buffer) {
        // Create a bordered block (no title since logo provides branding)
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Calculate status width for layout
        let status_widget = ConnectionStatusWidget::new(self.status, self.theme, self.symbols);
        let status_display_width = (status_widget.display_width() as u16).min(inner.width / 2);

        // Split: logo on left, status on right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),                          // Logo area
                Constraint::Length(status_display_width + 1), // Status area
            ])
            .split(inner);

        // Render compact/text logo on the left
        let logo_widget = LogoWidget::new(self.theme, self.symbols);
        logo_widget.render(chunks[0], buf);

        // Render status on the right, vertically centered
        let status_y = chunks[1].y + chunks[1].height / 2;
        let status_area = Rect::new(chunks[1].x, status_y, chunks[1].width, 1);
        status_widget.render(status_area, buf);
    }
}

impl Widget for HeaderWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        if self.is_wide_layout(area.width) {
            self.render_wide(area, buf);
        } else {
            self.render_narrow(area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{ASCII_SYMBOLS, UNICODE_SYMBOLS};

    // ============================================
    // ConnectionStatusWidget Tests
    // ============================================

    #[test]
    fn connection_status_widget_can_be_created() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert!(matches!(widget.status, ConnectionStatus::Connected));
    }

    #[test]
    fn connection_status_widget_is_debug() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("ConnectionStatusWidget"));
    }

    #[test]
    fn connection_status_widget_renders_without_panic_all_statuses() {
        let theme = Theme::default();
        let symbols = Symbols::default();
        let area = Rect::new(0, 0, 40, 1);

        for status in [
            ConnectionStatus::Connected,
            ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting,
            ConnectionStatus::Error,
        ] {
            let widget = ConnectionStatusWidget::new(status, &theme, &symbols);
            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
            // Should not panic
        }
    }

    #[test]
    fn connection_status_widget_uses_unicode_symbols() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "●");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Disconnected, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "○");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connecting, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "◔");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Error, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "○"); // Uses disconnected symbol
    }

    #[test]
    fn connection_status_widget_uses_ascii_symbols() {
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "[*]");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Disconnected, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "[ ]");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connecting, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "[.]");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Error, &theme, &symbols);
        assert_eq!(widget.status_symbol(), "[ ]"); // Uses disconnected symbol
    }

    #[test]
    fn connection_status_widget_returns_correct_text() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert_eq!(widget.status_text(), "Connected");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Disconnected, &theme, &symbols);
        assert_eq!(widget.status_text(), "Disconnected");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connecting, &theme, &symbols);
        assert_eq!(widget.status_text(), "Connecting...");

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Error, &theme, &symbols);
        assert_eq!(widget.status_text(), "Error");
    }

    #[test]
    fn connection_status_widget_uses_correct_theme_styles() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert_eq!(widget.status_style(), theme.status_connected);

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Disconnected, &theme, &symbols);
        assert_eq!(widget.status_style(), theme.status_disconnected);

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connecting, &theme, &symbols);
        assert_eq!(widget.status_style(), theme.status_connecting);

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Error, &theme, &symbols);
        assert_eq!(widget.status_style(), theme.status_disconnected);
    }

    #[test]
    fn connection_status_widget_handles_zero_area() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Zero width
        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 0, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic

        // Zero height
        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 40, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic
    }

    #[test]
    fn connection_status_widget_renders_connected_with_unicode() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        // Area needs to be wide enough to fit "● Connected" (11 chars)
        let area = Rect::new(0, 0, 25, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Collect buffer content (full symbols, not just first char)
        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("Connected"),
            "Should contain 'Connected' text, got: '{}'",
            content
        );
        assert!(
            content.contains("●"),
            "Should contain unicode connected symbol"
        );
    }

    #[test]
    fn connection_status_widget_renders_disconnected_with_ascii() {
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Disconnected, &theme, &symbols);
        // Area needs to be wide enough for "[ ] Disconnected" (16 chars)
        let area = Rect::new(0, 0, 25, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("Disconnected"),
            "Should contain 'Disconnected' text"
        );
    }

    #[test]
    fn connection_status_widget_display_width_correct() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        // "●" (1) + " " (1) + "Connected" (9) = 11
        assert_eq!(widget.display_width(), 11);

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connecting, &theme, &symbols);
        // "◔" (1) + " " (1) + "Connecting..." (13) = 15
        assert_eq!(widget.display_width(), 15);
    }

    #[test]
    fn connection_status_widget_with_monochrome_theme() {
        let theme = Theme::monochrome();
        let symbols = Symbols::default();

        let widget = ConnectionStatusWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic
    }

    // ============================================
    // HeaderWidget Tests
    // ============================================

    #[test]
    fn header_widget_can_be_created() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        assert!(matches!(widget.status, ConnectionStatus::Connected));
        assert!(widget.session_name.is_none());
    }

    #[test]
    fn header_widget_is_debug() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("HeaderWidget"));
    }

    #[test]
    fn header_widget_can_set_session_name() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols)
            .with_session_name("my-macbook");

        assert_eq!(widget.session_name, Some("my-macbook"));
    }

    #[test]
    fn header_widget_renders_without_panic_all_statuses() {
        let theme = Theme::default();
        let symbols = Symbols::default();
        let area = Rect::new(0, 0, 80, 3);

        for status in [
            ConnectionStatus::Connected,
            ConnectionStatus::Disconnected,
            ConnectionStatus::Connecting,
            ConnectionStatus::Error,
        ] {
            let widget = HeaderWidget::new(status, &theme, &symbols);
            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
            // Should not panic
        }
    }

    #[test]
    fn header_widget_handles_zero_area() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Zero width
        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 0, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic

        // Zero height
        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 80, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic
    }

    #[test]
    fn header_widget_detects_wide_layout() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);

        assert!(widget.is_wide_layout(80));
        assert!(widget.is_wide_layout(100));
        assert!(widget.is_wide_layout(120));
    }

    #[test]
    fn header_widget_detects_narrow_layout() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);

        assert!(!widget.is_wide_layout(79));
        assert!(!widget.is_wide_layout(60));
        assert!(!widget.is_wide_layout(40));
    }

    #[test]
    fn header_widget_renders_wide_layout() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 100, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain VibeTea title
        assert!(content.contains("VibeTea"), "Should show VibeTea title");
        // Should contain connection status
        assert!(
            content.contains("Connected"),
            "Should show connection status"
        );
    }

    #[test]
    fn header_widget_renders_narrow_layout() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 60, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should still show essential info
        assert!(
            content.contains("VibeTea"),
            "Should show VibeTea in narrow mode"
        );
    }

    #[test]
    fn header_widget_renders_with_session_name() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols)
            .with_session_name("my-macbook");
        let area = Rect::new(0, 0, 100, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain session name in wide layout
        assert!(content.contains("my-macbook"), "Should show session name");
    }

    #[test]
    fn header_widget_renders_with_ascii_symbols() {
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 80, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Should not panic with ASCII symbols
    }

    #[test]
    fn header_widget_renders_with_monochrome_theme() {
        let theme = Theme::monochrome();
        let symbols = Symbols::default();

        let widget = HeaderWidget::new(ConnectionStatus::Connected, &theme, &symbols);
        let area = Rect::new(0, 0, 80, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Should not panic with monochrome theme
    }

    // ============================================
    // header_height Tests
    // ============================================

    #[test]
    fn header_height_returns_correct_value_for_full_logo() {
        // Wide terminals get full logo (5 lines) + 2 for borders = 7
        assert_eq!(header_height(80), 7);
        assert_eq!(header_height(100), 7);
        assert_eq!(header_height(120), 7);
    }

    #[test]
    fn header_height_returns_correct_value_for_compact_logo() {
        // Medium terminals get compact logo (1 line) + 2 for borders = 3
        assert_eq!(header_height(40), 3);
        assert_eq!(header_height(50), 3);
        assert_eq!(header_height(59), 3);
    }

    #[test]
    fn header_height_returns_correct_value_for_text_logo() {
        // Narrow terminals get text logo (1 line) + 2 for borders = 3
        assert_eq!(header_height(20), 3);
        assert_eq!(header_height(29), 3);
    }

    #[test]
    fn header_height_handles_zero_width() {
        // Zero width gets text logo, minimum height of 3
        assert_eq!(header_height(0), 3);
    }

    #[test]
    fn header_height_handles_very_large_width() {
        // Very large width still uses full logo
        assert_eq!(header_height(1000), 7);
    }

    #[test]
    fn header_height_at_threshold_boundaries() {
        // At exact threshold for full logo (60)
        assert_eq!(header_height(60), 7);
        // Just below threshold for full logo
        assert_eq!(header_height(59), 3);
        // At exact threshold for compact logo (30)
        assert_eq!(header_height(30), 3);
        // Just below threshold for compact logo
        assert_eq!(header_height(29), 3);
    }
}
