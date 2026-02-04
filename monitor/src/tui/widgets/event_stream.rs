//! Event stream widget for displaying real-time session events.
//!
//! This module provides the [`EventStreamWidget`] for rendering a scrollable list
//! of session events in the TUI. Each event is displayed with a timestamp, icon,
//! and message, with visual styling based on event type.
//!
//! # Requirements Compliance
//!
//! - **FR-009**: Event stream shows timestamp, event type, and message for each event
//! - **FR-010**: Each event type has unique visual identifier (icon + color)
//! - **FR-011**: New events auto-scroll unless user has manually scrolled up
//! - **NFR-002**: Color scheme is color-blind safe (uses symbols alongside colors)
//! - **NFR-003**: Supports both unicode and ASCII-only modes
//!
//! # Layout
//!
//! Each event row is formatted as:
//!
//! ```text
//! [HH:MM:SS] [ICON] Message text here...
//! ```
//!
//! Where:
//! - `HH:MM:SS` is the local timestamp
//! - `ICON` is the event type icon (unicode or ASCII based on [`Symbols`])
//! - Message is truncated if it exceeds available width
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::EventStreamWidget;
//! use vibetea_monitor::tui::app::{EventBuffer, Theme, Symbols, UNICODE_SYMBOLS};
//!
//! let buffer = EventBuffer::new(1000);
//! let theme = Theme::default();
//! let symbols = UNICODE_SYMBOLS;
//!
//! let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
//! frame.render_widget(widget, area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};

use crate::tui::app::{DisplayEvent, DisplayEventType, EventBuffer, Symbols, Theme, UNICODE_SYMBOLS};

/// Widget for rendering the event stream.
///
/// This widget is stateless and takes references to the event buffer and styling
/// configuration. It implements the [`Widget`] trait for rendering with ratatui.
///
/// # Scrolling
///
/// The widget supports scrolling through the event buffer. The `scroll_offset`
/// parameter determines how many events from the bottom are scrolled up. When
/// `scroll_offset` is 0, the view is at the bottom showing the most recent events.
///
/// # Visual Styling
///
/// Events are styled based on their type using the provided [`Theme`]:
/// - Session events: Magenta (bold)
/// - Activity events: Blue
/// - Tool events: Cyan
/// - Agent events: Yellow
/// - Summary events: Green
/// - Error events: Red
///
/// Recent events (less than 2 seconds old) are highlighted with bold text
/// per FR-011.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::EventStreamWidget;
/// use vibetea_monitor::tui::app::{EventBuffer, Theme, Symbols};
///
/// let buffer = EventBuffer::new(1000);
/// let theme = Theme::default();
/// let symbols = Symbols::detect();
///
/// // Show events with no scroll (at bottom)
/// let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
/// frame.render_widget(widget, area);
///
/// // Show events scrolled up by 10 lines
/// let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 10, 20);
/// frame.render_widget(widget, area);
/// ```
#[derive(Debug)]
pub struct EventStreamWidget<'a> {
    /// Reference to the event buffer containing display events.
    buffer: &'a EventBuffer,
    /// Reference to the theme for styling.
    theme: &'a Theme,
    /// Reference to the symbol set (unicode or ASCII).
    symbols: &'a Symbols,
    /// Number of lines scrolled up from the bottom.
    scroll_offset: usize,
    /// Number of visible lines in the widget area.
    visible_height: u16,
}

impl<'a> EventStreamWidget<'a> {
    /// Creates a new `EventStreamWidget` with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `buffer` - Event buffer containing the events to display
    /// * `theme` - Theme configuration for colors and styles
    /// * `symbols` - Symbol set (unicode or ASCII) for icons
    /// * `scroll_offset` - Lines scrolled up from the bottom (0 = at bottom)
    /// * `visible_height` - Number of visible lines in the render area
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::EventStreamWidget;
    /// use vibetea_monitor::tui::app::{EventBuffer, Theme, Symbols};
    ///
    /// let buffer = EventBuffer::new(100);
    /// let theme = Theme::default();
    /// let symbols = Symbols::detect();
    ///
    /// let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
    /// ```
    #[must_use]
    pub fn new(
        buffer: &'a EventBuffer,
        theme: &'a Theme,
        symbols: &'a Symbols,
        scroll_offset: usize,
        visible_height: u16,
    ) -> Self {
        Self {
            buffer,
            theme,
            symbols,
            scroll_offset,
            visible_height,
        }
    }

    /// Returns the style for a given event type from the theme.
    fn event_type_style(&self, event_type: DisplayEventType) -> Style {
        match event_type {
            DisplayEventType::Session => self.theme.event_type_session,
            DisplayEventType::Activity => self.theme.event_type_activity,
            DisplayEventType::Tool => self.theme.event_type_tool,
            DisplayEventType::Agent => self.theme.event_type_agent,
            DisplayEventType::Summary => self.theme.event_type_summary,
            DisplayEventType::Error => self.theme.event_type_error,
        }
    }

    /// Returns the appropriate icon for an event type based on symbol mode.
    ///
    /// Uses unicode icons if the symbol set is unicode, otherwise uses ASCII.
    fn event_icon(&self, event_type: DisplayEventType) -> &'static str {
        // Check if we're using unicode by comparing a known symbol
        if self.symbols.connected == UNICODE_SYMBOLS.connected {
            event_type.icon()
        } else {
            event_type.ascii_icon()
        }
    }

    /// Formats a single event as a styled line.
    ///
    /// Format: `[HH:MM:SS] [ICON] Message`
    ///
    /// Recent events (age < 2 seconds) are highlighted with bold styling.
    fn format_event(&self, event: &DisplayEvent, max_width: usize) -> Line<'a> {
        let timestamp = event.formatted_timestamp();
        let icon = self.event_icon(event.event_type);
        let is_recent = event.age_secs() < 2;

        // Calculate available space for the message
        // Format: "[HH:MM:SS] [ICON] " = 8 + 1 + 1 + icon_len + 1 + 1 = 12 + icon_len
        let icon_width = unicode_width(icon);
        let prefix_width = 12 + icon_width; // "[HH:MM:SS] " + icon + " "
        let available_width = max_width.saturating_sub(prefix_width);

        // Truncate message if needed
        let message = truncate_to_width(&event.message, available_width);

        // Build the styled spans
        let mut timestamp_style = self.theme.event_timestamp;
        let mut icon_style = self.event_type_style(event.event_type);
        let mut message_style = self.theme.text_primary;

        // Apply recent highlighting
        if is_recent {
            timestamp_style = timestamp_style.patch(self.theme.event_recent);
            icon_style = icon_style.patch(self.theme.event_recent);
            message_style = message_style.patch(self.theme.event_recent);
        }

        Line::from(vec![
            Span::styled("[", timestamp_style),
            Span::styled(timestamp, timestamp_style),
            Span::styled("] ", timestamp_style),
            Span::styled(icon, icon_style),
            Span::styled(" ", self.theme.text_primary),
            Span::styled(message, message_style),
        ])
    }

    /// Calculates which events should be visible based on scroll state.
    ///
    /// Returns the range of event indices to render, from oldest to newest
    /// within the visible window.
    fn visible_range(&self) -> std::ops::Range<usize> {
        let total_events = self.buffer.len();
        let visible = self.visible_height as usize;

        if total_events == 0 {
            return 0..0;
        }

        // Calculate the end index (accounting for scroll from bottom)
        let end = total_events.saturating_sub(self.scroll_offset);

        // Calculate the start index
        let start = end.saturating_sub(visible);

        start..end
    }
}

impl Widget for EventStreamWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let visible_range = self.visible_range();
        let events: Vec<_> = self.buffer.iter().collect();

        // Render empty state message if no events
        if events.is_empty() {
            let empty_message = "No events yet...";
            let x = area.x + area.width.saturating_sub(empty_message.len() as u16) / 2;
            let y = area.y + area.height / 2;
            if y < area.y + area.height && x < area.x + area.width {
                buf.set_string(x, y, empty_message, self.theme.text_muted);
            }
            return;
        }

        // Render visible events
        let max_width = area.width as usize;
        let mut y = area.y;

        for idx in visible_range {
            if y >= area.y + area.height {
                break;
            }

            if let Some(event) = events.get(idx) {
                let line = self.format_event(event, max_width);

                // Render the line spans
                let mut x = area.x;
                for span in line.spans {
                    let text = span.content.as_ref();
                    let remaining_width = (area.x + area.width).saturating_sub(x) as usize;
                    let text_to_render = if text.len() > remaining_width {
                        &text[..remaining_width]
                    } else {
                        text
                    };
                    buf.set_string(x, y, text_to_render, span.style);
                    x += text_to_render.len() as u16;
                }

                y += 1;
            }
        }
    }
}

/// Calculates the display width of a string (handling unicode).
///
/// For simplicity, this uses byte length as an approximation. A more accurate
/// implementation would use the `unicode-width` crate.
fn unicode_width(s: &str) -> usize {
    // Most unicode icons we use are 1-2 display columns
    // ASCII icons like "[S]" are exactly their byte length
    s.chars().count()
}

/// Truncates a string to fit within the specified display width.
///
/// Adds "..." suffix if truncation occurs.
fn truncate_to_width(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        return String::new();
    }

    let char_count = s.chars().count();
    if char_count <= max_width {
        return s.to_string();
    }

    // Truncate and add ellipsis
    let truncate_to = max_width.saturating_sub(3);
    let truncated: String = s.chars().take(truncate_to).collect();
    format!("{}...", truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{DisplayEvent, DisplayEventType, EventBuffer, Symbols, Theme, ASCII_SYMBOLS, UNICODE_SYMBOLS};

    /// Helper to create a test event.
    fn test_event(id: &str, event_type: DisplayEventType, message: &str) -> DisplayEvent {
        DisplayEvent::new(id.to_string(), event_type, message.to_string())
    }

    #[test]
    fn event_stream_widget_can_be_created() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
        assert_eq!(widget.scroll_offset, 0);
        assert_eq!(widget.visible_height, 20);
    }

    #[test]
    fn event_stream_widget_is_debug() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("EventStreamWidget"));
    }

    #[test]
    fn event_stream_widget_renders_without_panic() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // Should not panic
        widget.render(area, &mut buf);
    }

    #[test]
    fn event_stream_widget_renders_empty_state() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Check that empty message is rendered
        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("No events yet"),
            "Should show empty state message"
        );
    }

    #[test]
    fn event_stream_widget_renders_events() {
        let mut buffer = EventBuffer::new(100);
        buffer.push(test_event("evt_1", DisplayEventType::Session, "Session started"));
        buffer.push(test_event("evt_2", DisplayEventType::Tool, "Read file: main.rs"));

        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        // Check that event messages are rendered
        assert!(
            content.contains("Session started"),
            "Should render session event"
        );
        assert!(
            content.contains("Read file"),
            "Should render tool event"
        );
    }

    #[test]
    fn event_stream_widget_uses_unicode_icons() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        // Check icon selection
        assert_eq!(widget.event_icon(DisplayEventType::Session), "\u{1F4AC}");
        assert_eq!(widget.event_icon(DisplayEventType::Tool), "\u{1F527}");
        assert_eq!(widget.event_icon(DisplayEventType::Error), "\u{26A0}");
    }

    #[test]
    fn event_stream_widget_uses_ascii_icons() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        // Check icon selection
        assert_eq!(widget.event_icon(DisplayEventType::Session), "[S]");
        assert_eq!(widget.event_icon(DisplayEventType::Tool), "[T]");
        assert_eq!(widget.event_icon(DisplayEventType::Error), "[!]");
    }

    #[test]
    fn event_stream_widget_renders_with_monochrome_theme() {
        let mut buffer = EventBuffer::new(100);
        buffer.push(test_event("evt_1", DisplayEventType::Session, "Test event"));

        let theme = Theme::monochrome();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);

        // Should not panic with monochrome theme
        widget.render(area, &mut buf);
    }

    #[test]
    fn event_stream_widget_renders_with_ascii_symbols() {
        let mut buffer = EventBuffer::new(100);
        buffer.push(test_event("evt_1", DisplayEventType::Tool, "Using tool"));

        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Should render the ASCII icon
        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();
        assert!(
            content.contains("[T]"),
            "Should render ASCII tool icon"
        );
    }

    #[test]
    fn event_stream_widget_visible_range_empty_buffer() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let range = widget.visible_range();
        assert_eq!(range, 0..0);
    }

    #[test]
    fn event_stream_widget_visible_range_fewer_than_visible() {
        let mut buffer = EventBuffer::new(100);
        buffer.push(test_event("evt_1", DisplayEventType::Session, "Event 1"));
        buffer.push(test_event("evt_2", DisplayEventType::Session, "Event 2"));
        buffer.push(test_event("evt_3", DisplayEventType::Session, "Event 3"));

        let theme = Theme::default();
        let symbols = Symbols::detect();

        // 20 visible lines, but only 3 events
        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let range = widget.visible_range();
        assert_eq!(range, 0..3);
    }

    #[test]
    fn event_stream_widget_visible_range_with_scroll() {
        let mut buffer = EventBuffer::new(100);
        for i in 0..50 {
            buffer.push(test_event(&format!("evt_{}", i), DisplayEventType::Session, &format!("Event {}", i)));
        }

        let theme = Theme::default();
        let symbols = Symbols::detect();

        // 20 visible lines, scrolled up by 10
        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 10, 20);

        let range = widget.visible_range();
        // Total 50 events, scroll_offset 10, visible 20
        // End = 50 - 10 = 40, Start = 40 - 20 = 20
        assert_eq!(range, 20..40);
    }

    #[test]
    fn event_stream_widget_visible_range_at_bottom() {
        let mut buffer = EventBuffer::new(100);
        for i in 0..50 {
            buffer.push(test_event(&format!("evt_{}", i), DisplayEventType::Session, &format!("Event {}", i)));
        }

        let theme = Theme::default();
        let symbols = Symbols::detect();

        // 20 visible lines, at bottom (scroll_offset = 0)
        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let range = widget.visible_range();
        // Total 50 events, visible 20, at bottom
        // End = 50, Start = 30
        assert_eq!(range, 30..50);
    }

    #[test]
    fn event_type_style_returns_correct_theme_styles() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        assert_eq!(widget.event_type_style(DisplayEventType::Session), theme.event_type_session);
        assert_eq!(widget.event_type_style(DisplayEventType::Activity), theme.event_type_activity);
        assert_eq!(widget.event_type_style(DisplayEventType::Tool), theme.event_type_tool);
        assert_eq!(widget.event_type_style(DisplayEventType::Agent), theme.event_type_agent);
        assert_eq!(widget.event_type_style(DisplayEventType::Summary), theme.event_type_summary);
        assert_eq!(widget.event_type_style(DisplayEventType::Error), theme.event_type_error);
    }

    #[test]
    fn truncate_to_width_no_truncation_needed() {
        let result = truncate_to_width("short", 20);
        assert_eq!(result, "short");
    }

    #[test]
    fn truncate_to_width_exact_fit() {
        let result = truncate_to_width("exact", 5);
        assert_eq!(result, "exact");
    }

    #[test]
    fn truncate_to_width_truncates_with_ellipsis() {
        let result = truncate_to_width("this is a very long message", 15);
        assert_eq!(result, "this is a ve...");
        assert_eq!(result.chars().count(), 15);
    }

    #[test]
    fn truncate_to_width_very_small_width() {
        let result = truncate_to_width("anything", 3);
        assert_eq!(result, "");
    }

    #[test]
    fn unicode_width_ascii() {
        assert_eq!(unicode_width("hello"), 5);
        assert_eq!(unicode_width("[T]"), 3);
    }

    #[test]
    fn unicode_width_unicode() {
        // Single unicode character
        assert_eq!(unicode_width("\u{1F527}"), 1); // wrench emoji
        assert_eq!(unicode_width("●"), 1);
    }

    #[test]
    fn event_stream_widget_handles_zero_area() {
        let buffer = EventBuffer::new(100);
        let theme = Theme::default();
        let symbols = Symbols::detect();

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        // Zero width
        let area = Rect::new(0, 0, 0, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic

        // Zero height
        let buffer = EventBuffer::new(100);
        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);
        let area = Rect::new(0, 0, 80, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic
    }

    #[test]
    fn event_stream_widget_renders_all_event_types() {
        let mut buffer = EventBuffer::new(100);
        buffer.push(test_event("evt_1", DisplayEventType::Session, "Session event"));
        buffer.push(test_event("evt_2", DisplayEventType::Activity, "Activity event"));
        buffer.push(test_event("evt_3", DisplayEventType::Tool, "Tool event"));
        buffer.push(test_event("evt_4", DisplayEventType::Agent, "Agent event"));
        buffer.push(test_event("evt_5", DisplayEventType::Summary, "Summary event"));
        buffer.push(test_event("evt_6", DisplayEventType::Error, "Error event"));

        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 0, 20);

        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf
            .content
            .iter()
            .map(|cell| cell.symbol().chars().next().unwrap_or(' '))
            .collect();

        // All event messages should be present
        assert!(content.contains("Session event"));
        assert!(content.contains("Activity event"));
        assert!(content.contains("Tool event"));
        assert!(content.contains("Agent event"));
        assert!(content.contains("Summary event"));
        assert!(content.contains("Error event"));
    }

    #[test]
    fn event_stream_widget_scroll_offset_clamped() {
        let mut buffer = EventBuffer::new(100);
        for i in 0..10 {
            buffer.push(test_event(&format!("evt_{}", i), DisplayEventType::Session, &format!("Event {}", i)));
        }

        let theme = Theme::default();
        let symbols = Symbols::detect();

        // Scroll offset larger than total events
        let widget = EventStreamWidget::new(&buffer, &theme, &symbols, 100, 20);

        let range = widget.visible_range();
        // Should clamp to show nothing (end = 0, start = 0)
        assert_eq!(range.start, 0);
        assert!(range.end <= 10);
    }
}
