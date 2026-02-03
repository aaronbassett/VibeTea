//! Setup form widget for initial configuration.
//!
//! This module provides the [`SetupFormWidget`] for rendering the setup screen
//! that appears when the VibeTea Monitor is first launched. The form collects:
//!
//! - **Session Name**: An identifier for this monitoring session (FR-003)
//! - **Key Option**: Whether to use existing keys or generate new ones (FR-004)
//!
//! # Layout
//!
//! The form is rendered as a centered panel with a title, two input fields,
//! and a submit button:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │        VibeTea Monitor Setup        │
//! │                                     │
//! │  Session Name:                      │
//! │  ┌───────────────────────────────┐  │
//! │  │ hostname█                     │  │
//! │  └───────────────────────────────┘  │
//! │  (validation error if any)          │
//! │                                     │
//! │  Key Option:                        │
//! │  [ ] Use existing    [*] Generate   │
//! │                                     │
//! │           [ Start ]                 │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Styling
//!
//! The widget uses the provided [`Theme`] for consistent styling:
//! - Focused fields have highlighted borders and text
//! - Validation errors are displayed in the error style
//! - The selected key option is indicated with a filled symbol
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::SetupFormWidget;
//! use vibetea_monitor::tui::{SetupFormState, Theme, Symbols};
//!
//! let state = SetupFormState::default();
//! let theme = Theme::default();
//! let symbols = Symbols::default();
//!
//! let widget = SetupFormWidget::new(&state, &theme, &symbols);
//! frame.render_widget(widget, area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::tui::app::{KeyOption, SetupField, SetupFormState, Symbols, Theme};

/// Minimum width required to render the form properly.
const MIN_FORM_WIDTH: u16 = 40;

/// Maximum width for the form panel.
const MAX_FORM_WIDTH: u16 = 60;

/// Height of the form content (excluding outer border).
const FORM_CONTENT_HEIGHT: u16 = 14;

/// Widget for rendering the setup form.
///
/// This widget is stateless and takes references to all required state and
/// configuration. It implements the [`Widget`] trait for rendering with ratatui.
///
/// # Fields
///
/// The widget renders:
/// - A titled border around the entire form
/// - A session name text input with cursor indicator when focused
/// - Validation error text below the session name (if present)
/// - Key option radio buttons
/// - A submit button
///
/// # Focus Indication
///
/// The currently focused field is indicated by:
/// - Highlighted border color (from theme)
/// - Bold text style
/// - Cursor indicator (for text input)
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::SetupFormWidget;
/// use vibetea_monitor::tui::{SetupFormState, Theme, Symbols};
///
/// let state = SetupFormState {
///     session_name: "my-session".to_string(),
///     session_name_error: None,
///     key_option: KeyOption::GenerateNew,
///     focused_field: SetupField::SessionName,
///     existing_keys_found: false,
/// };
/// let theme = Theme::default();
/// let symbols = Symbols::default();
///
/// let widget = SetupFormWidget::new(&state, &theme, &symbols);
/// // Render with frame.render_widget(widget, area);
/// ```
#[derive(Debug)]
pub struct SetupFormWidget<'a> {
    /// Reference to the form state.
    state: &'a SetupFormState,
    /// Reference to the theme for styling.
    theme: &'a Theme,
    /// Reference to the symbol set.
    symbols: &'a Symbols,
}

impl<'a> SetupFormWidget<'a> {
    /// Creates a new `SetupFormWidget` with the given state and styling.
    ///
    /// # Arguments
    ///
    /// * `state` - The current form state including field values and focus
    /// * `theme` - Theme configuration for colors and styles
    /// * `symbols` - Symbol set (unicode or ASCII) for visual indicators
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::SetupFormWidget;
    /// use vibetea_monitor::tui::{SetupFormState, Theme, Symbols};
    ///
    /// let state = SetupFormState::default();
    /// let theme = Theme::default();
    /// let symbols = Symbols::default();
    ///
    /// let widget = SetupFormWidget::new(&state, &theme, &symbols);
    /// ```
    #[must_use]
    pub fn new(state: &'a SetupFormState, theme: &'a Theme, symbols: &'a Symbols) -> Self {
        Self {
            state,
            theme,
            symbols,
        }
    }

    /// Calculates the centered area for the form panel.
    ///
    /// The form is horizontally and vertically centered within the given area,
    /// with a maximum width of [`MAX_FORM_WIDTH`] and height of [`FORM_CONTENT_HEIGHT`]
    /// plus border space.
    fn centered_rect(&self, area: Rect) -> Rect {
        // Calculate form width (constrained to MIN..MAX)
        let form_width = area.width.clamp(MIN_FORM_WIDTH, MAX_FORM_WIDTH);
        let form_height = FORM_CONTENT_HEIGHT + 2; // +2 for borders

        // Center horizontally
        let x = area.x + area.width.saturating_sub(form_width) / 2;
        // Center vertically
        let y = area.y + area.height.saturating_sub(form_height) / 2;

        Rect::new(
            x,
            y,
            form_width.min(area.width),
            form_height.min(area.height),
        )
    }

    /// Renders the session name input field.
    fn render_session_name_field(&self, buf: &mut Buffer, area: Rect) {
        let is_focused = self.state.focused_field == SetupField::SessionName;

        // Label
        let label_style = if is_focused {
            self.theme.label.add_modifier(Modifier::BOLD)
        } else {
            self.theme.label
        };
        let label = Paragraph::new("Session Name:").style(label_style);

        // Input field with border
        let input_border_style = if is_focused {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        // Build the input text with cursor if focused
        let input_text = if is_focused {
            format!("{}_", self.state.session_name)
        } else {
            self.state.session_name.clone()
        };

        let input_style = if is_focused {
            self.theme.input_focused
        } else {
            self.theme.input_unfocused
        };

        let input = Paragraph::new(input_text).style(input_style).block(
            Block::default().borders(Borders::ALL).style(
                Style::default().fg(input_border_style
                    .fg
                    .unwrap_or(ratatui::style::Color::Reset)),
            ),
        );

        // Split area: label (1 line) + input (3 lines with border)
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(3)]).split(area);

        label.render(chunks[0], buf);
        input.render(chunks[1], buf);
    }

    /// Renders the validation error message if present.
    fn render_error_message(&self, buf: &mut Buffer, area: Rect) {
        if let Some(ref error) = self.state.session_name_error {
            let error_text = Paragraph::new(error.as_str()).style(self.theme.input_error);
            error_text.render(area, buf);
        }
    }

    /// Renders the key option selector.
    fn render_key_option_field(&self, buf: &mut Buffer, area: Rect) {
        let is_focused = self.state.focused_field == SetupField::KeyOption;

        // Label
        let label_style = if is_focused {
            self.theme.label.add_modifier(Modifier::BOLD)
        } else {
            self.theme.label
        };
        let label = Paragraph::new("Key Option:").style(label_style);

        // Build the option line with radio buttons
        let (existing_indicator, generate_indicator) = match self.state.key_option {
            KeyOption::UseExisting => (self.symbols.connected, self.symbols.disconnected),
            KeyOption::GenerateNew => (self.symbols.disconnected, self.symbols.connected),
        };

        // Style for the options
        let option_style = if is_focused {
            self.theme.input_focused
        } else {
            self.theme.input_unfocused
        };

        // Build option text with proper styling
        let existing_text = if self.state.existing_keys_found {
            format!("[{}] Use existing", existing_indicator)
        } else {
            format!("[{}] Use existing (none found)", existing_indicator)
        };

        let generate_text = format!("[{}] Generate new", generate_indicator);

        // Highlight the selected option
        let (existing_style, generate_style) = match self.state.key_option {
            KeyOption::UseExisting => {
                let selected = if is_focused {
                    option_style.add_modifier(Modifier::BOLD)
                } else {
                    option_style
                };
                let unselected = self.theme.text_muted;
                (selected, unselected)
            }
            KeyOption::GenerateNew => {
                let selected = if is_focused {
                    option_style.add_modifier(Modifier::BOLD)
                } else {
                    option_style
                };
                let unselected = self.theme.text_muted;
                (unselected, selected)
            }
        };

        let options_line = Line::from(vec![
            Span::styled(existing_text, existing_style),
            Span::raw("  "),
            Span::styled(generate_text, generate_style),
        ]);

        let options = Paragraph::new(options_line);

        // Split area: label (1 line) + options (1 line)
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

        label.render(chunks[0], buf);
        options.render(chunks[1], buf);
    }

    /// Renders the submit button.
    fn render_submit_button(&self, buf: &mut Buffer, area: Rect) {
        let is_focused = self.state.focused_field == SetupField::Submit;

        let button_style = if is_focused {
            self.theme
                .input_focused
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            self.theme.input_unfocused
        };

        let button_text = if is_focused {
            format!(" {} Start {} ", self.symbols.arrow, self.symbols.arrow)
        } else {
            "  Start  ".to_string()
        };

        let button = Paragraph::new(button_text)
            .style(button_style)
            .alignment(Alignment::Center);

        button.render(area, buf);
    }
}

impl Widget for SetupFormWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get centered form area
        let form_area = self.centered_rect(area);

        // Create outer block with title
        let outer_block = Block::default()
            .title(" VibeTea Monitor Setup ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .style(self.theme.text_primary);

        // Calculate inner area (without borders)
        let inner_area = outer_block.inner(form_area);

        // Render the outer block
        outer_block.render(form_area, buf);

        // If there's not enough space, just show a message
        if inner_area.width < 20 || inner_area.height < 10 {
            let message = Paragraph::new("Window too small")
                .style(self.theme.text_muted)
                .alignment(Alignment::Center);
            message.render(inner_area, buf);
            return;
        }

        // Layout the form content with proper spacing
        // Layout:
        // - 1 line: top padding
        // - 4 lines: session name field (label + input with border)
        // - 1 line: error message
        // - 1 line: spacing
        // - 2 lines: key option (label + options)
        // - 1 line: spacing
        // - 1 line: submit button
        // - remaining: bottom padding
        let content_chunks = Layout::vertical([
            Constraint::Length(1), // Top padding
            Constraint::Length(4), // Session name field
            Constraint::Length(1), // Error message
            Constraint::Length(1), // Spacing
            Constraint::Length(2), // Key option field
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // Submit button
            Constraint::Min(0),    // Bottom padding
        ])
        .split(inner_area);

        // Add horizontal padding
        let h_padding = 2;
        let padded_area = |rect: Rect| -> Rect {
            Rect::new(
                rect.x + h_padding,
                rect.y,
                rect.width.saturating_sub(h_padding * 2),
                rect.height,
            )
        };

        // Render each field
        self.render_session_name_field(buf, padded_area(content_chunks[1]));
        self.render_error_message(buf, padded_area(content_chunks[2]));
        self.render_key_option_field(buf, padded_area(content_chunks[4]));
        self.render_submit_button(buf, padded_area(content_chunks[6]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{KeyOption, SetupField, SetupFormState, Symbols, Theme};

    /// Helper to create a default test state.
    fn default_state() -> SetupFormState {
        SetupFormState {
            session_name: "test-session".to_string(),
            session_name_error: None,
            key_option: KeyOption::GenerateNew,
            focused_field: SetupField::SessionName,
            existing_keys_found: false,
        }
    }

    #[test]
    fn setup_form_widget_can_be_created() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);
        // Verify the widget holds references correctly
        assert_eq!(widget.state.session_name, "test-session");
    }

    #[test]
    fn setup_form_widget_is_debug() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("SetupFormWidget"));
    }

    #[test]
    fn setup_form_widget_renders_without_panic() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        // Create a buffer to render into
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // This should not panic
        widget.render(area, &mut buf);
    }

    #[test]
    fn setup_form_widget_renders_title() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Check that the title is rendered somewhere in the buffer
        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("VibeTea Monitor Setup"),
            "Title should be in buffer"
        );
    }

    #[test]
    fn setup_form_widget_renders_session_name_label() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("Session Name"),
            "Session Name label should be in buffer"
        );
    }

    #[test]
    fn setup_form_widget_renders_key_option_label() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("Key Option"),
            "Key Option label should be in buffer"
        );
    }

    #[test]
    fn setup_form_widget_renders_with_error() {
        let mut state = default_state();
        state.session_name_error = Some("Invalid name".to_string());

        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("Invalid name"),
            "Error message should be in buffer"
        );
    }

    #[test]
    fn setup_form_widget_renders_in_small_area() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        // Very small area - should not panic
        let area = Rect::new(0, 0, 30, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Should show "Window too small" message
        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("Window too small"),
            "Should show size warning in small area"
        );
    }

    #[test]
    fn setup_form_widget_renders_with_monochrome_theme() {
        let state = default_state();
        let theme = Theme::monochrome();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // Should not panic with monochrome theme
        widget.render(area, &mut buf);
    }

    #[test]
    fn setup_form_widget_renders_with_ascii_symbols() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = crate::tui::app::ASCII_SYMBOLS;

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // Should not panic with ASCII symbols
        widget.render(area, &mut buf);
    }

    #[test]
    fn setup_form_widget_renders_all_focus_states() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Test each focus state
        for field in [
            SetupField::SessionName,
            SetupField::KeyOption,
            SetupField::Submit,
        ] {
            let state = SetupFormState {
                session_name: "test".to_string(),
                session_name_error: None,
                key_option: KeyOption::GenerateNew,
                focused_field: field,
                existing_keys_found: false,
            };

            let widget = SetupFormWidget::new(&state, &theme, &symbols);

            let area = Rect::new(0, 0, 80, 24);
            let mut buf = Buffer::empty(area);

            // Should not panic for any focus state
            widget.render(area, &mut buf);
        }
    }

    #[test]
    fn setup_form_widget_renders_both_key_options() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Test UseExisting
        let state = SetupFormState {
            session_name: "test".to_string(),
            session_name_error: None,
            key_option: KeyOption::UseExisting,
            focused_field: SetupField::KeyOption,
            existing_keys_found: true,
        };

        let widget = SetupFormWidget::new(&state, &theme, &symbols);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Test GenerateNew
        let state = SetupFormState {
            key_option: KeyOption::GenerateNew,
            ..state
        };

        let widget = SetupFormWidget::new(&state, &theme, &symbols);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
    }

    #[test]
    fn setup_form_widget_renders_existing_keys_not_found() {
        let state = SetupFormState {
            session_name: "test".to_string(),
            session_name_error: None,
            key_option: KeyOption::GenerateNew,
            focused_field: SetupField::KeyOption,
            existing_keys_found: false,
        };

        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("none found"),
            "Should indicate no existing keys"
        );
    }

    #[test]
    fn centered_rect_calculates_correct_position() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        // Test with a large area
        let area = Rect::new(0, 0, 100, 50);
        let centered = widget.centered_rect(area);

        // The form should be centered
        assert!(centered.x > 0, "Form should be horizontally offset");
        assert!(centered.y > 0, "Form should be vertically offset");
        assert!(
            centered.width <= MAX_FORM_WIDTH,
            "Width should be constrained"
        );
        assert!(
            centered.width >= MIN_FORM_WIDTH,
            "Width should meet minimum"
        );
    }

    #[test]
    fn centered_rect_handles_small_area() {
        let state = default_state();
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = SetupFormWidget::new(&state, &theme, &symbols);

        // Test with a small area
        let area = Rect::new(0, 0, 30, 10);
        let centered = widget.centered_rect(area);

        // Should not exceed the available area
        assert!(centered.x + centered.width <= area.width);
        assert!(centered.y + centered.height <= area.height);
    }
}
