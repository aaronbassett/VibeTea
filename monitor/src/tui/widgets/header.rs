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
//! - **T125**: Header widget combines logo placeholder and status
//! - **T127**: Graceful degradation for narrow terminals
//!
//! # Layout Modes
//!
//! The header widget adapts based on terminal width:
//!
//! - **Wide (>= 80 columns)**: Shows "VibeTea" text on left, connection status on right
//! - **Narrow (< 80 columns)**: Shows "VibeTea" and status in compact form
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

use crate::tui::app::{ConnectionStatus, Symbols, Theme};

/// Width threshold for switching between wide and narrow layouts.
const WIDE_LAYOUT_THRESHOLD: u16 = 80;

/// Height of the header in both narrow and wide modes.
///
/// Currently returns 3 for all widths. This may be increased when
/// the full ASCII art logo is implemented in US8.
const HEADER_HEIGHT: u16 = 3;

/// Returns the height needed for the header widget.
///
/// This function calculates the required height based on terminal width.
/// Currently returns a fixed height, but will adapt when the full ASCII
/// art logo is implemented.
///
/// # Arguments
///
/// * `_area_width` - The available width (used for future logo sizing)
///
/// # Returns
///
/// The number of rows needed for the header.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::header_height;
///
/// let height = header_height(80);
/// assert_eq!(height, 3);
/// ```
#[must_use]
pub fn header_height(_area_width: u16) -> u16 {
    // For now, fixed height. When full logo is added in US8,
    // this will return different values based on width.
    HEADER_HEIGHT
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
        // Create a bordered block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .title("VibeTea")
            .title_style(self.theme.title);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Split inner area: logo placeholder on left, status on right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(10),    // Logo/branding area
                Constraint::Length(20), // Status area
            ])
            .split(inner);

        // Render branding text on the left (placeholder until US8 adds full logo)
        let branding = if let Some(name) = self.session_name {
            format!("Session: {}", name)
        } else {
            String::new()
        };

        if !branding.is_empty() {
            let branding_paragraph = Paragraph::new(branding).style(self.theme.text_secondary);
            branding_paragraph.render(chunks[0], buf);
        }

        // Render connection status on the right
        let status_widget = ConnectionStatusWidget::new(self.status, self.theme, self.symbols);

        // Position status in the right chunk, vertically centered
        let status_y = chunks[1].y + chunks[1].height / 2;
        let status_area = Rect::new(chunks[1].x, status_y, chunks[1].width, 1);
        status_widget.render(status_area, buf);
    }

    /// Renders the header in narrow layout mode.
    fn render_narrow(&self, area: Rect, buf: &mut Buffer) {
        // Create a bordered block with compact title
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .title("VibeTea")
            .title_style(self.theme.title);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // In narrow mode, just show the status right-aligned
        let status_widget = ConnectionStatusWidget::new(self.status, self.theme, self.symbols);

        // Render status on the first line of inner area
        let status_area = Rect::new(inner.x, inner.y, inner.width, 1);
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
    fn header_height_returns_correct_value() {
        // Currently fixed at 3, may change with US8 logo implementation
        assert_eq!(header_height(40), 3);
        assert_eq!(header_height(80), 3);
        assert_eq!(header_height(120), 3);
    }

    #[test]
    fn header_height_handles_zero_width() {
        assert_eq!(header_height(0), 3);
    }

    #[test]
    fn header_height_handles_very_large_width() {
        assert_eq!(header_height(1000), 3);
    }
}
