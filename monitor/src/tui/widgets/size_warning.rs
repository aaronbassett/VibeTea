//! Terminal size warning widget for the VibeTea Monitor TUI.
//!
//! This module provides a widget for displaying a warning when the terminal
//! is too small to properly render the TUI. The warning shows the current
//! terminal size and the minimum required dimensions.
//!
//! # Requirements Compliance
//!
//! - **T247**: Terminal size warning widget displays when terminal is below minimum
//! - **T249**: Minimum terminal size validation (80x24)
//!
//! # Minimum Size Requirements
//!
//! The TUI requires a minimum terminal size of 80 columns by 24 rows to display
//! properly. When the terminal is smaller than this, the size warning widget
//! should be rendered instead of the normal UI.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::{SizeWarningWidget, check_terminal_size};
//! use vibetea_monitor::tui::widgets::size_warning::{MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT};
//!
//! let (width, height) = tui.size()?;
//!
//! if !check_terminal_size(width, height) {
//!     let widget = SizeWarningWidget::new(width, height);
//!     frame.render_widget(widget, frame.area());
//! } else {
//!     // Render normal UI
//! }
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

// =============================================================================
// Minimum Size Constants
// =============================================================================

/// Minimum terminal width for the TUI to display properly.
///
/// The TUI layout assumes at least 80 columns to render all widgets
/// without truncation or overlap.
pub const MIN_TERMINAL_WIDTH: u16 = 80;

/// Minimum terminal height for the TUI to display properly.
///
/// The TUI layout assumes at least 24 rows to render the header,
/// event stream, and footer widgets without overlap.
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

// =============================================================================
// Size Check Functions
// =============================================================================

/// Checks if the terminal size meets minimum requirements.
///
/// Returns `true` if both the width and height meet or exceed the minimum
/// requirements ([`MIN_TERMINAL_WIDTH`] and [`MIN_TERMINAL_HEIGHT`]).
///
/// # Arguments
///
/// * `width` - The current terminal width in columns
/// * `height` - The current terminal height in rows
///
/// # Returns
///
/// `true` if the terminal meets minimum size requirements, `false` otherwise.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::widgets::size_warning::check_terminal_size;
///
/// assert!(check_terminal_size(80, 24));   // Exactly minimum
/// assert!(check_terminal_size(120, 40));  // Larger than minimum
/// assert!(!check_terminal_size(79, 24));  // Width too small
/// assert!(!check_terminal_size(80, 23));  // Height too small
/// assert!(!check_terminal_size(40, 12));  // Both too small
/// ```
#[must_use]
pub fn check_terminal_size(width: u16, height: u16) -> bool {
    width >= MIN_TERMINAL_WIDTH && height >= MIN_TERMINAL_HEIGHT
}

/// Returns information about the current terminal size status.
///
/// This function provides detailed information about whether the terminal
/// meets minimum requirements and what dimensions are lacking.
///
/// # Arguments
///
/// * `width` - The current terminal width in columns
/// * `height` - The current terminal height in rows
///
/// # Returns
///
/// A [`TerminalSizeStatus`] struct containing the current size, required size,
/// and whether each dimension meets requirements.
///
/// # Example
///
/// ```
/// use vibetea_monitor::tui::widgets::size_warning::get_terminal_size_status;
///
/// let status = get_terminal_size_status(60, 20);
/// assert!(!status.meets_requirements());
/// assert!(!status.width_ok);
/// assert!(!status.height_ok);
/// ```
#[must_use]
pub fn get_terminal_size_status(width: u16, height: u16) -> TerminalSizeStatus {
    TerminalSizeStatus {
        current_width: width,
        current_height: height,
        required_width: MIN_TERMINAL_WIDTH,
        required_height: MIN_TERMINAL_HEIGHT,
        width_ok: width >= MIN_TERMINAL_WIDTH,
        height_ok: height >= MIN_TERMINAL_HEIGHT,
    }
}

// =============================================================================
// Terminal Size Status
// =============================================================================

/// Information about terminal size requirements.
///
/// This struct provides detailed information about the current terminal size
/// compared to the minimum requirements, useful for displaying informative
/// error messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSizeStatus {
    /// Current terminal width in columns.
    pub current_width: u16,
    /// Current terminal height in rows.
    pub current_height: u16,
    /// Required minimum width in columns.
    pub required_width: u16,
    /// Required minimum height in rows.
    pub required_height: u16,
    /// Whether the current width meets requirements.
    pub width_ok: bool,
    /// Whether the current height meets requirements.
    pub height_ok: bool,
}

impl TerminalSizeStatus {
    /// Returns `true` if the terminal meets all size requirements.
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::widgets::size_warning::get_terminal_size_status;
    ///
    /// let status = get_terminal_size_status(80, 24);
    /// assert!(status.meets_requirements());
    ///
    /// let status = get_terminal_size_status(60, 20);
    /// assert!(!status.meets_requirements());
    /// ```
    #[must_use]
    pub fn meets_requirements(&self) -> bool {
        self.width_ok && self.height_ok
    }
}

// =============================================================================
// Size Warning Widget
// =============================================================================

/// Widget that displays a warning when the terminal is too small.
///
/// This widget renders a centered message explaining that the terminal
/// dimensions are below the minimum required for the TUI to display properly.
/// It shows both the current size and the required minimum size.
///
/// # Display
///
/// The warning is displayed with a yellow/orange theme to indicate a warning
/// state (not an error). The message is centered and wrapped to fit within
/// the available space.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::SizeWarningWidget;
///
/// let widget = SizeWarningWidget::new(60, 20);
/// frame.render_widget(widget, frame.area());
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SizeWarningWidget {
    /// Current terminal width in columns.
    current_width: u16,
    /// Current terminal height in rows.
    current_height: u16,
}

impl SizeWarningWidget {
    /// Creates a new `SizeWarningWidget`.
    ///
    /// # Arguments
    ///
    /// * `current_width` - The current terminal width in columns
    /// * `current_height` - The current terminal height in rows
    ///
    /// # Example
    ///
    /// ```
    /// use vibetea_monitor::tui::widgets::SizeWarningWidget;
    ///
    /// let widget = SizeWarningWidget::new(60, 20);
    /// ```
    #[must_use]
    pub fn new(current_width: u16, current_height: u16) -> Self {
        Self {
            current_width,
            current_height,
        }
    }

    /// Returns the current terminal width.
    #[must_use]
    pub fn current_width(&self) -> u16 {
        self.current_width
    }

    /// Returns the current terminal height.
    #[must_use]
    pub fn current_height(&self) -> u16 {
        self.current_height
    }

    /// Creates the warning message lines.
    fn create_message_lines(&self) -> Vec<Line<'static>> {
        let warning_style = Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD);
        let text_style = Style::default().fg(Color::White);
        let size_style = Style::default().fg(Color::Cyan);
        let error_size_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

        // Determine which dimensions are problematic
        let width_style = if self.current_width >= MIN_TERMINAL_WIDTH {
            size_style
        } else {
            error_size_style
        };
        let height_style = if self.current_height >= MIN_TERMINAL_HEIGHT {
            size_style
        } else {
            error_size_style
        };

        vec![
            Line::from(vec![Span::styled("Terminal Too Small", warning_style)]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "The terminal window is too small to display",
                text_style,
            )]),
            Line::from(vec![Span::styled(
                "the VibeTea Monitor interface.",
                text_style,
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Current size: ", text_style),
                Span::styled(format!("{}", self.current_width), width_style),
                Span::styled(" x ", text_style),
                Span::styled(format!("{}", self.current_height), height_style),
            ]),
            Line::from(vec![
                Span::styled("Required size: ", text_style),
                Span::styled(format!("{}", MIN_TERMINAL_WIDTH), size_style),
                Span::styled(" x ", text_style),
                Span::styled(format!("{}", MIN_TERMINAL_HEIGHT), size_style),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Please resize your terminal window.",
                text_style,
            )]),
        ]
    }
}

impl Widget for SizeWarningWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Handle zero-size area gracefully
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Create the warning block with a border
        let warning_style = Style::default().fg(Color::Yellow);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(warning_style)
            .title(" Warning ")
            .title_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        // If inner area is too small, just show minimal message
        if inner.width < 10 || inner.height < 3 {
            // Render a minimal "Resize" message
            let msg = "Resize";
            let x = inner.x + inner.width.saturating_sub(msg.len() as u16) / 2;
            let y = inner.y + inner.height / 2;
            if y < inner.y + inner.height && x < inner.x + inner.width {
                buf.set_string(x, y, msg, Style::default().fg(Color::Yellow));
            }
            return;
        }

        // Create the message lines
        let lines = self.create_message_lines();

        // Calculate vertical centering
        let content_height = lines.len() as u16;
        let vertical_offset = if inner.height > content_height {
            (inner.height - content_height) / 2
        } else {
            0
        };

        // Create and render the paragraph
        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        // Create an adjusted area for vertical centering
        let centered_area = Rect::new(
            inner.x,
            inner.y + vertical_offset,
            inner.width,
            inner.height.saturating_sub(vertical_offset),
        );

        paragraph.render(centered_area, buf);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Constants Tests
    // =========================================================================

    #[test]
    fn minimum_terminal_width_is_80() {
        assert_eq!(MIN_TERMINAL_WIDTH, 80);
    }

    #[test]
    fn minimum_terminal_height_is_24() {
        assert_eq!(MIN_TERMINAL_HEIGHT, 24);
    }

    // =========================================================================
    // check_terminal_size Tests
    // =========================================================================

    #[test]
    fn check_terminal_size_returns_true_at_exact_minimum() {
        assert!(check_terminal_size(80, 24));
    }

    #[test]
    fn check_terminal_size_returns_true_above_minimum() {
        assert!(check_terminal_size(100, 30));
        assert!(check_terminal_size(120, 40));
        assert!(check_terminal_size(200, 60));
    }

    #[test]
    fn check_terminal_size_returns_false_width_too_small() {
        assert!(!check_terminal_size(79, 24));
        assert!(!check_terminal_size(60, 24));
        assert!(!check_terminal_size(40, 24));
        assert!(!check_terminal_size(0, 24));
    }

    #[test]
    fn check_terminal_size_returns_false_height_too_small() {
        assert!(!check_terminal_size(80, 23));
        assert!(!check_terminal_size(80, 20));
        assert!(!check_terminal_size(80, 10));
        assert!(!check_terminal_size(80, 0));
    }

    #[test]
    fn check_terminal_size_returns_false_both_too_small() {
        assert!(!check_terminal_size(79, 23));
        assert!(!check_terminal_size(60, 20));
        assert!(!check_terminal_size(40, 12));
        assert!(!check_terminal_size(0, 0));
    }

    #[test]
    fn check_terminal_size_boundary_values() {
        // Just below minimum
        assert!(!check_terminal_size(79, 24));
        assert!(!check_terminal_size(80, 23));

        // Exactly at minimum
        assert!(check_terminal_size(80, 24));

        // Just above minimum
        assert!(check_terminal_size(81, 24));
        assert!(check_terminal_size(80, 25));
    }

    // =========================================================================
    // TerminalSizeStatus Tests
    // =========================================================================

    #[test]
    fn terminal_size_status_meets_requirements_at_minimum() {
        let status = get_terminal_size_status(80, 24);
        assert!(status.meets_requirements());
        assert!(status.width_ok);
        assert!(status.height_ok);
    }

    #[test]
    fn terminal_size_status_does_not_meet_requirements_below_minimum() {
        let status = get_terminal_size_status(60, 20);
        assert!(!status.meets_requirements());
        assert!(!status.width_ok);
        assert!(!status.height_ok);
    }

    #[test]
    fn terminal_size_status_partial_requirements() {
        // Width OK, height not
        let status = get_terminal_size_status(80, 20);
        assert!(!status.meets_requirements());
        assert!(status.width_ok);
        assert!(!status.height_ok);

        // Height OK, width not
        let status = get_terminal_size_status(60, 24);
        assert!(!status.meets_requirements());
        assert!(!status.width_ok);
        assert!(status.height_ok);
    }

    #[test]
    fn terminal_size_status_stores_correct_values() {
        let status = get_terminal_size_status(100, 50);
        assert_eq!(status.current_width, 100);
        assert_eq!(status.current_height, 50);
        assert_eq!(status.required_width, MIN_TERMINAL_WIDTH);
        assert_eq!(status.required_height, MIN_TERMINAL_HEIGHT);
    }

    #[test]
    fn terminal_size_status_is_debug() {
        let status = get_terminal_size_status(80, 24);
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("TerminalSizeStatus"));
    }

    #[test]
    fn terminal_size_status_is_clone_and_copy() {
        let status = get_terminal_size_status(80, 24);
        let cloned = status.clone();
        let copied = status;
        assert_eq!(status, cloned);
        assert_eq!(status, copied);
    }

    // =========================================================================
    // SizeWarningWidget Tests
    // =========================================================================

    #[test]
    fn size_warning_widget_can_be_created() {
        let widget = SizeWarningWidget::new(60, 20);
        assert_eq!(widget.current_width(), 60);
        assert_eq!(widget.current_height(), 20);
    }

    #[test]
    fn size_warning_widget_is_debug() {
        let widget = SizeWarningWidget::new(60, 20);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("SizeWarningWidget"));
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn size_warning_widget_is_clone_and_copy() {
        let widget = SizeWarningWidget::new(60, 20);
        let cloned = widget.clone();
        let copied = widget;
        assert_eq!(widget.current_width(), cloned.current_width());
        assert_eq!(widget.current_height(), copied.current_height());
    }

    #[test]
    fn size_warning_widget_renders_without_panic() {
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn size_warning_widget_handles_zero_area() {
        // Zero width
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 0, 20);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic

        // Zero height
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 60, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic

        // Both zero
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn size_warning_widget_handles_very_small_area() {
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 5, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should render minimal message without panic
    }

    #[test]
    fn size_warning_widget_renders_warning_message() {
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain key warning text
        assert!(
            content.contains("Terminal Too Small") || content.contains("Resize"),
            "Should show warning message"
        );
    }

    #[test]
    fn size_warning_widget_shows_current_size() {
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain the current dimensions
        assert!(content.contains("60"), "Should show current width");
        assert!(content.contains("20"), "Should show current height");
    }

    #[test]
    fn size_warning_widget_shows_required_size() {
        let widget = SizeWarningWidget::new(60, 20);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain the required dimensions
        assert!(content.contains("80"), "Should show required width");
        assert!(content.contains("24"), "Should show required height");
    }

    #[test]
    fn size_warning_widget_renders_at_various_sizes() {
        // Test rendering at different terminal sizes
        for (width, height) in [(40, 15), (50, 18), (70, 22), (60, 20)] {
            let widget = SizeWarningWidget::new(width, height);
            let area = Rect::new(0, 0, width, height);
            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
            // Should not panic at any size
        }
    }

    #[test]
    fn size_warning_widget_creates_correct_message_lines() {
        let widget = SizeWarningWidget::new(60, 20);
        let lines = widget.create_message_lines();

        // Should have multiple lines
        assert!(lines.len() > 5, "Should have multiple message lines");

        // First line should be the title
        let first_line_text: String = lines[0].spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(first_line_text.contains("Terminal Too Small"));
    }
}
