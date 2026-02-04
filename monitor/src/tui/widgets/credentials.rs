//! Credentials panel widget for the VibeTea Monitor TUI.
//!
//! This module provides a widget for displaying session credentials including
//! the session name and public key. The public key is shown in base64 format
//! suitable for copy-paste.
//!
//! # Requirements Compliance
//!
//! - **FR-009**: Main dashboard MUST display a credentials panel below the stream
//!   showing session name and public key
//! - **FR-010**: Public key MUST be displayed in base64 format suitable for copy-paste
//! - **US4 acceptance criteria**:
//!   - Session name and full public key visible below the stream
//!   - Public key shown in single line, base64 format for copy-paste
//!   - Handle narrow terminals: public key truncates gracefully with "..." indication
//!
//! # Layout
//!
//! The credentials panel displays two lines within a bordered block:
//!
//! ```text
//! ┌─ Credentials ─────────────────────────────────┐
//! │ Session: my-session-name                      │
//! │ Public Key: AAAAC3NzaC1lZDI1NTE5AAAAI...      │
//! └───────────────────────────────────────────────┘
//! ```
//!
//! On narrow terminals, the public key is truncated with "..." to indicate
//! that the full value extends beyond the visible area.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::CredentialsWidget;
//! use vibetea_monitor::tui::app::{Credentials, Theme};
//!
//! let theme = Theme::default();
//! let credentials = Credentials {
//!     session_name: "my-macbook".to_string(),
//!     public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".to_string(),
//! };
//!
//! let widget = CredentialsWidget::new(&credentials, &theme);
//! frame.render_widget(widget, credentials_area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::tui::app::{Credentials, Theme};

/// Label for the session name field.
const SESSION_LABEL: &str = "Session: ";

/// Label for the public key field.
const PUBLIC_KEY_LABEL: &str = "Public Key: ";

/// Truncation indicator shown when the public key is too long to fit.
const TRUNCATION_INDICATOR: &str = "...";

/// Height of the credentials widget in rows.
///
/// The widget requires 4 rows: 1 for top border, 2 for content lines
/// (session name and public key), and 1 for bottom border.
pub const CREDENTIALS_HEIGHT: u16 = 4;

/// Widget for displaying session credentials.
///
/// Renders a bordered panel showing the session name and public key.
/// The public key is displayed in base64 format suitable for copy-paste.
/// On narrow terminals, the public key is truncated with "..." to indicate
/// that the full value extends beyond the visible area.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::CredentialsWidget;
/// use vibetea_monitor::tui::app::{Credentials, Theme};
///
/// let theme = Theme::default();
/// let credentials = Credentials {
///     session_name: "my-macbook".to_string(),
///     public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".to_string(),
/// };
///
/// let widget = CredentialsWidget::new(&credentials, &theme);
/// frame.render_widget(widget, credentials_area);
/// ```
#[derive(Debug)]
pub struct CredentialsWidget<'a> {
    /// Reference to the credentials to display.
    credentials: &'a Credentials,
    /// Reference to the theme for styling.
    theme: &'a Theme,
}

impl<'a> CredentialsWidget<'a> {
    /// Creates a new `CredentialsWidget`.
    ///
    /// # Arguments
    ///
    /// * `credentials` - The credentials to display (session name and public key)
    /// * `theme` - Theme configuration for colors and styles
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::CredentialsWidget;
    /// use vibetea_monitor::tui::app::{Credentials, Theme};
    ///
    /// let theme = Theme::default();
    /// let credentials = Credentials {
    ///     session_name: "my-macbook".to_string(),
    ///     public_key: "AAAAC3NzaC1lZDI1NTE5AAAAI...".to_string(),
    /// };
    ///
    /// let widget = CredentialsWidget::new(&credentials, &theme);
    /// ```
    #[must_use]
    pub fn new(credentials: &'a Credentials, theme: &'a Theme) -> Self {
        Self { credentials, theme }
    }

    /// Truncates the public key to fit within the available width.
    ///
    /// If the public key fits within the available width, it is returned as-is.
    /// Otherwise, it is truncated and "..." is appended to indicate truncation.
    ///
    /// # Arguments
    ///
    /// * `available_width` - The maximum number of characters available for the public key value
    ///
    /// # Returns
    ///
    /// The public key string, possibly truncated with "..." appended.
    fn truncate_public_key(&self, available_width: usize) -> String {
        let key = &self.credentials.public_key;
        let key_len = key.len();

        if key_len <= available_width {
            // Key fits entirely
            key.clone()
        } else if available_width <= TRUNCATION_INDICATOR.len() {
            // Not enough space for anything meaningful
            TRUNCATION_INDICATOR[..available_width].to_string()
        } else {
            // Truncate and add indicator
            let truncate_at = available_width - TRUNCATION_INDICATOR.len();
            format!("{}{}", &key[..truncate_at], TRUNCATION_INDICATOR)
        }
    }

    /// Creates the session name line.
    fn session_line(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled(SESSION_LABEL, self.theme.text_secondary),
            Span::styled(
                self.credentials.session_name.clone(),
                self.theme.text_primary,
            ),
        ])
    }

    /// Creates the public key line with optional truncation.
    ///
    /// # Arguments
    ///
    /// * `available_width` - The total width available for the line content
    fn public_key_line(&self, available_width: usize) -> Line<'a> {
        // Calculate available width for the public key value itself
        let label_len = PUBLIC_KEY_LABEL.len();
        let key_available = available_width.saturating_sub(label_len);

        let truncated_key = self.truncate_public_key(key_available);

        Line::from(vec![
            Span::styled(PUBLIC_KEY_LABEL, self.theme.text_secondary),
            Span::styled(truncated_key, self.theme.text_primary),
        ])
    }
}

impl Widget for CredentialsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Create a bordered block with title
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .title("Credentials")
            .title_style(self.theme.title);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Build the content lines
        let available_width = inner.width as usize;
        let lines = vec![self.session_line(), self.public_key_line(available_width)];

        // Render the paragraph within the inner area
        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates test credentials for use in tests.
    fn test_credentials() -> Credentials {
        Credentials {
            session_name: "test-session".to_string(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAIJOaQZmBw7a/P7GZm2P5sE1a".to_string(),
        }
    }

    // ============================================
    // CredentialsWidget Construction Tests
    // ============================================

    #[test]
    fn credentials_widget_can_be_created() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        assert_eq!(widget.credentials.session_name, "test-session");
    }

    #[test]
    fn credentials_widget_is_debug() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("CredentialsWidget"));
    }

    // ============================================
    // Public Key Truncation Tests
    // ============================================

    #[test]
    fn truncate_public_key_returns_full_key_when_fits() {
        let theme = Theme::default();
        let credentials = Credentials {
            session_name: "test".to_string(),
            public_key: "short-key".to_string(),
        };

        let widget = CredentialsWidget::new(&credentials, &theme);
        let result = widget.truncate_public_key(20);
        assert_eq!(result, "short-key");
    }

    #[test]
    fn truncate_public_key_truncates_with_ellipsis() {
        let theme = Theme::default();
        let credentials = Credentials {
            session_name: "test".to_string(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAIJOaQZmBw7a/P7GZm2P5sE1a".to_string(),
        };

        let widget = CredentialsWidget::new(&credentials, &theme);
        let result = widget.truncate_public_key(20);

        // Should be 17 chars of key + 3 chars of "..."
        assert_eq!(result.len(), 20);
        assert!(result.ends_with("..."));
        // First 17 chars of "AAAAC3NzaC1lZDI1NTE5AAAAIJOaQZmBw7a/P7GZm2P5sE1a"
        assert!(result.starts_with("AAAAC3NzaC1lZDI1N"));
    }

    #[test]
    fn truncate_public_key_handles_minimal_width() {
        let theme = Theme::default();
        let credentials = Credentials {
            session_name: "test".to_string(),
            public_key: "AAAAC3NzaC1lZDI1NTE5".to_string(),
        };

        let widget = CredentialsWidget::new(&credentials, &theme);

        // Width of 3 should just show "..."
        let result = widget.truncate_public_key(3);
        assert_eq!(result, "...");
    }

    #[test]
    fn truncate_public_key_handles_zero_width() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let result = widget.truncate_public_key(0);
        assert_eq!(result, "");
    }

    #[test]
    fn truncate_public_key_handles_width_less_than_ellipsis() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);

        // Width of 2 should truncate the ellipsis itself
        let result = widget.truncate_public_key(2);
        assert_eq!(result, "..");
    }

    // ============================================
    // Line Creation Tests
    // ============================================

    #[test]
    fn session_line_contains_label_and_name() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let line = widget.session_line();

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Session: ");
        assert_eq!(line.spans[1].content, "test-session");
    }

    #[test]
    fn public_key_line_contains_label_and_key() {
        let theme = Theme::default();
        let credentials = Credentials {
            session_name: "test".to_string(),
            public_key: "short-key".to_string(),
        };

        let widget = CredentialsWidget::new(&credentials, &theme);
        let line = widget.public_key_line(50);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Public Key: ");
        assert_eq!(line.spans[1].content, "short-key");
    }

    #[test]
    fn public_key_line_truncates_when_necessary() {
        let theme = Theme::default();
        let credentials = Credentials {
            session_name: "test".to_string(),
            public_key: "AAAAC3NzaC1lZDI1NTE5AAAAIJOaQZmBw7a/P7GZm2P5sE1a".to_string(),
        };

        let widget = CredentialsWidget::new(&credentials, &theme);
        // Label is 12 chars, so key gets 18 chars
        let line = widget.public_key_line(30);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "Public Key: ");
        // Key should be truncated to 18 chars total (15 + "...")
        assert!(line.spans[1].content.ends_with("..."));
        assert_eq!(line.spans[1].content.len(), 18);
    }

    // ============================================
    // Rendering Tests
    // ============================================

    #[test]
    fn credentials_widget_renders_without_panic() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let area = Rect::new(0, 0, 60, 4);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn credentials_widget_handles_zero_width() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let area = Rect::new(0, 0, 0, 4);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn credentials_widget_handles_zero_height() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let area = Rect::new(0, 0, 60, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn credentials_widget_handles_minimal_area() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        // Minimum bordered area: 3x3 for borders, nothing inside
        let area = Rect::new(0, 0, 3, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn credentials_widget_renders_content_in_wide_area() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let area = Rect::new(0, 0, 80, 5);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain title
        assert!(
            content.contains("Credentials"),
            "Should show Credentials title"
        );
        // Should contain session name
        assert!(content.contains("Session:"), "Should show Session label");
        assert!(content.contains("test-session"), "Should show session name");
        // Should contain public key
        assert!(
            content.contains("Public Key:"),
            "Should show Public Key label"
        );
    }

    #[test]
    fn credentials_widget_truncates_in_narrow_area() {
        let theme = Theme::default();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        // Narrow width that forces truncation
        // Border takes 2 chars, so inner width is 28
        let area = Rect::new(0, 0, 30, 4);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain truncation indicator
        assert!(
            content.contains("..."),
            "Should show truncation indicator for narrow terminal"
        );
    }

    #[test]
    fn credentials_widget_renders_with_monochrome_theme() {
        let theme = Theme::monochrome();
        let credentials = test_credentials();

        let widget = CredentialsWidget::new(&credentials, &theme);
        let area = Rect::new(0, 0, 60, 4);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic with monochrome theme
    }

    // ============================================
    // Constants Tests
    // ============================================

    #[test]
    fn credentials_height_is_correct() {
        // 1 top border + 2 content lines + 1 bottom border = 4
        assert_eq!(CREDENTIALS_HEIGHT, 4);
    }

    #[test]
    fn session_label_is_correct() {
        assert_eq!(SESSION_LABEL, "Session: ");
    }

    #[test]
    fn public_key_label_is_correct() {
        assert_eq!(PUBLIC_KEY_LABEL, "Public Key: ");
    }
}
