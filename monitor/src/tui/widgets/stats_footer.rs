//! Statistics footer widget for the VibeTea Monitor TUI.
//!
//! This module provides a widget for displaying session statistics including
//! total events, successfully sent events, and failed events. The failed count
//! is visually distinguished with a warning style when non-zero.
//!
//! # Requirements Compliance
//!
//! - **FR-012**: Footer MUST show counts for Total Events, Sent, and Failed
//! - **US5 acceptance criteria**:
//!   - Footer displays Total, Sent, and Failed counts
//!   - Failed count uses warning style (red/bold) when `events_failed > 0`
//!   - Counts reflect actual events processed since startup
//!   - Handles narrow terminals with graceful degradation
//!
//! # Layout
//!
//! The statistics footer displays a single line within a bordered block:
//!
//! ```text
//! ┌─ Statistics ─────────────────────────────────────┐
//! │ Total: 100  |  Sent: 95  |  Failed: 5            │
//! └───────────────────────────────────────────────────┘
//! ```
//!
//! On very narrow terminals, separators may be omitted or the display gracefully
//! degrades to show as much information as possible.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::StatsFooterWidget;
//! use vibetea_monitor::tui::app::{EventStats, Theme};
//!
//! let theme = Theme::default();
//! let stats = EventStats {
//!     total_events: 100,
//!     events_sent: 95,
//!     events_failed: 5,
//! };
//!
//! let widget = StatsFooterWidget::new(&stats, &theme);
//! frame.render_widget(widget, footer_area);
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::tui::app::{EventStats, Theme};

/// Label for the total events count.
const TOTAL_LABEL: &str = "Total: ";

/// Label for the sent events count.
const SENT_LABEL: &str = "Sent: ";

/// Label for the failed events count.
const FAILED_LABEL: &str = "Failed: ";

/// Separator between statistics.
const SEPARATOR: &str = "  |  ";

/// Height of the stats footer widget in rows.
///
/// The widget requires 3 rows: 1 for top border, 1 for content line,
/// and 1 for bottom border.
pub const STATS_FOOTER_HEIGHT: u16 = 3;

/// Widget for displaying session statistics.
///
/// Renders a bordered panel showing total events, sent count, and failed count.
/// The failed count is visually distinguished with a warning style (red/bold)
/// when there are any failed events.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::StatsFooterWidget;
/// use vibetea_monitor::tui::app::{EventStats, Theme};
///
/// let theme = Theme::default();
/// let stats = EventStats {
///     total_events: 100,
///     events_sent: 95,
///     events_failed: 5,
/// };
///
/// let widget = StatsFooterWidget::new(&stats, &theme);
/// frame.render_widget(widget, footer_area);
/// ```
#[derive(Debug)]
pub struct StatsFooterWidget<'a> {
    /// Reference to the statistics to display.
    stats: &'a EventStats,
    /// Reference to the theme for styling.
    theme: &'a Theme,
}

impl<'a> StatsFooterWidget<'a> {
    /// Creates a new `StatsFooterWidget`.
    ///
    /// # Arguments
    ///
    /// * `stats` - The event statistics to display (total, sent, failed)
    /// * `theme` - Theme configuration for colors and styles
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::StatsFooterWidget;
    /// use vibetea_monitor::tui::app::{EventStats, Theme};
    ///
    /// let theme = Theme::default();
    /// let stats = EventStats {
    ///     total_events: 100,
    ///     events_sent: 95,
    ///     events_failed: 5,
    /// };
    ///
    /// let widget = StatsFooterWidget::new(&stats, &theme);
    /// ```
    #[must_use]
    pub fn new(stats: &'a EventStats, theme: &'a Theme) -> Self {
        Self { stats, theme }
    }

    /// Creates the statistics line with optional separators based on available width.
    ///
    /// # Arguments
    ///
    /// * `available_width` - The total width available for the line content
    fn stats_line(&self, available_width: usize) -> Line<'a> {
        // Calculate minimum widths for each segment
        // Format: "Total: X" + separator + "Sent: Y" + separator + "Failed: Z"
        let total_value = self.stats.total_events.to_string();
        let sent_value = self.stats.events_sent.to_string();
        let failed_value = self.stats.events_failed.to_string();

        let total_segment_len = TOTAL_LABEL.len() + total_value.len();
        let sent_segment_len = SENT_LABEL.len() + sent_value.len();
        let failed_segment_len = FAILED_LABEL.len() + failed_value.len();

        let full_width_needed = total_segment_len
            + SEPARATOR.len()
            + sent_segment_len
            + SEPARATOR.len()
            + failed_segment_len;

        // Determine the style for the failed count
        let failed_style = if self.stats.events_failed > 0 {
            self.theme.stat_failed
        } else {
            self.theme.stat_sent // Use sent style (green/neutral) when no failures
        };

        // Build spans based on available width
        if available_width >= full_width_needed {
            // Full display with separators
            Line::from(vec![
                Span::styled(TOTAL_LABEL, self.theme.text_secondary),
                Span::styled(total_value, self.theme.stat_total),
                Span::styled(SEPARATOR, self.theme.text_secondary),
                Span::styled(SENT_LABEL, self.theme.text_secondary),
                Span::styled(sent_value, self.theme.stat_sent),
                Span::styled(SEPARATOR, self.theme.text_secondary),
                Span::styled(FAILED_LABEL, self.theme.text_secondary),
                Span::styled(failed_value, failed_style),
            ])
        } else {
            // Compact display without separators
            let compact_width = total_segment_len + 2 + sent_segment_len + 2 + failed_segment_len;
            if available_width >= compact_width {
                Line::from(vec![
                    Span::styled(TOTAL_LABEL, self.theme.text_secondary),
                    Span::styled(total_value, self.theme.stat_total),
                    Span::styled("  ", self.theme.text_secondary),
                    Span::styled(SENT_LABEL, self.theme.text_secondary),
                    Span::styled(sent_value, self.theme.stat_sent),
                    Span::styled("  ", self.theme.text_secondary),
                    Span::styled(FAILED_LABEL, self.theme.text_secondary),
                    Span::styled(failed_value, failed_style),
                ])
            } else {
                // Ultra-compact: show abbreviated labels
                let abbrev_width =
                    2 + total_value.len() + 1 + 2 + sent_value.len() + 1 + 2 + failed_value.len();
                if available_width >= abbrev_width {
                    Line::from(vec![
                        Span::styled("T:", self.theme.text_secondary),
                        Span::styled(total_value, self.theme.stat_total),
                        Span::styled(" ", self.theme.text_secondary),
                        Span::styled("S:", self.theme.text_secondary),
                        Span::styled(sent_value, self.theme.stat_sent),
                        Span::styled(" ", self.theme.text_secondary),
                        Span::styled("F:", self.theme.text_secondary),
                        Span::styled(failed_value, failed_style),
                    ])
                } else {
                    // Minimal: just show what fits
                    Line::from(vec![
                        Span::styled("T:", self.theme.text_secondary),
                        Span::styled(total_value, self.theme.stat_total),
                    ])
                }
            }
        }
    }
}

impl Widget for StatsFooterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        // Create a bordered block with title
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .title("Statistics")
            .title_style(self.theme.title);

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // Build the content line
        let available_width = inner.width as usize;
        let line = self.stats_line(available_width);

        // Render the paragraph within the inner area
        let paragraph = Paragraph::new(vec![line]);
        paragraph.render(inner, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates test statistics for use in tests.
    fn test_stats() -> EventStats {
        EventStats {
            total_events: 100,
            events_sent: 95,
            events_failed: 5,
        }
    }

    /// Creates test statistics with zero failures.
    fn test_stats_no_failures() -> EventStats {
        EventStats {
            total_events: 50,
            events_sent: 50,
            events_failed: 0,
        }
    }

    /// Creates test statistics with all zeros.
    fn test_stats_zeros() -> EventStats {
        EventStats {
            total_events: 0,
            events_sent: 0,
            events_failed: 0,
        }
    }

    // ============================================
    // StatsFooterWidget Construction Tests
    // ============================================

    #[test]
    fn stats_footer_widget_can_be_created() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        assert_eq!(widget.stats.total_events, 100);
        assert_eq!(widget.stats.events_sent, 95);
        assert_eq!(widget.stats.events_failed, 5);
    }

    #[test]
    fn stats_footer_widget_is_debug() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("StatsFooterWidget"));
    }

    // ============================================
    // Stats Line Creation Tests
    // ============================================

    #[test]
    fn stats_line_shows_all_values_when_wide() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let line = widget.stats_line(80);

        // Should have 8 spans: label, value, separator for each stat
        assert_eq!(line.spans.len(), 8);
        assert_eq!(line.spans[0].content, "Total: ");
        assert_eq!(line.spans[1].content, "100");
        assert_eq!(line.spans[2].content, "  |  ");
        assert_eq!(line.spans[3].content, "Sent: ");
        assert_eq!(line.spans[4].content, "95");
        assert_eq!(line.spans[5].content, "  |  ");
        assert_eq!(line.spans[6].content, "Failed: ");
        assert_eq!(line.spans[7].content, "5");
    }

    #[test]
    fn stats_line_uses_compact_format_when_medium_width() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        // Full format needs 37 chars (7+3+5+6+2+5+8+1), compact needs 31 (7+3+2+6+2+2+8+1)
        // Width 35 should trigger compact format
        let line = widget.stats_line(35);

        // Should still have 8 spans but with shorter separators
        assert_eq!(line.spans.len(), 8);
        // Check that separators are just spaces, not "  |  "
        assert_eq!(line.spans[2].content, "  ");
    }

    #[test]
    fn stats_line_uses_abbreviated_format_when_narrow() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        // Narrow width
        let line = widget.stats_line(25);

        // Should use abbreviated labels
        assert_eq!(line.spans[0].content, "T:");
        assert_eq!(line.spans[3].content, "S:");
        assert_eq!(line.spans[6].content, "F:");
    }

    #[test]
    fn stats_line_minimal_format_when_very_narrow() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        // Very narrow width - just show total
        let line = widget.stats_line(8);

        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content, "T:");
        assert_eq!(line.spans[1].content, "100");
    }

    #[test]
    fn stats_line_with_zero_failures_uses_sent_style() {
        let theme = Theme::default();
        let stats = test_stats_no_failures();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let line = widget.stats_line(80);

        // The failed value span should use stat_sent style when failures are 0
        // This is index 7 in the full format
        let failed_span = &line.spans[7];
        assert_eq!(failed_span.content, "0");
        // The style should be stat_sent (green), not stat_failed (red)
        assert_eq!(failed_span.style, theme.stat_sent);
    }

    #[test]
    fn stats_line_with_failures_uses_failed_style() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let line = widget.stats_line(80);

        // The failed value span should use stat_failed style when failures > 0
        let failed_span = &line.spans[7];
        assert_eq!(failed_span.content, "5");
        assert_eq!(failed_span.style, theme.stat_failed);
    }

    #[test]
    fn stats_line_handles_large_numbers() {
        let theme = Theme::default();
        let stats = EventStats {
            total_events: 1_000_000,
            events_sent: 999_999,
            events_failed: 1,
        };

        let widget = StatsFooterWidget::new(&stats, &theme);
        let line = widget.stats_line(100);

        assert_eq!(line.spans[1].content, "1000000");
        assert_eq!(line.spans[4].content, "999999");
        assert_eq!(line.spans[7].content, "1");
    }

    #[test]
    fn stats_line_handles_zeros() {
        let theme = Theme::default();
        let stats = test_stats_zeros();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let line = widget.stats_line(80);

        assert_eq!(line.spans[1].content, "0");
        assert_eq!(line.spans[4].content, "0");
        assert_eq!(line.spans[7].content, "0");
    }

    // ============================================
    // Rendering Tests
    // ============================================

    #[test]
    fn stats_footer_widget_renders_without_panic() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 60, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn stats_footer_widget_handles_zero_width() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 0, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn stats_footer_widget_handles_zero_height() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 60, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn stats_footer_widget_handles_minimal_area() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        // Minimum bordered area: 3x3 for borders, nothing inside
        let area = Rect::new(0, 0, 3, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic
    }

    #[test]
    fn stats_footer_widget_renders_content_in_wide_area() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 80, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain title
        assert!(
            content.contains("Statistics"),
            "Should show Statistics title"
        );
        // Should contain all labels and values
        assert!(content.contains("Total:"), "Should show Total label");
        assert!(content.contains("100"), "Should show total value");
        assert!(content.contains("Sent:"), "Should show Sent label");
        assert!(content.contains("95"), "Should show sent value");
        assert!(content.contains("Failed:"), "Should show Failed label");
        assert!(content.contains("5"), "Should show failed value");
    }

    #[test]
    fn stats_footer_widget_renders_in_narrow_area() {
        let theme = Theme::default();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        // Narrow width
        let area = Rect::new(0, 0, 30, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should still show some content, possibly abbreviated
        assert!(
            content.contains("T:") || content.contains("Total:"),
            "Should show total in some form"
        );
    }

    #[test]
    fn stats_footer_widget_renders_with_monochrome_theme() {
        let theme = Theme::monochrome();
        let stats = test_stats();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 60, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic with monochrome theme
    }

    #[test]
    fn stats_footer_widget_renders_zero_failures() {
        let theme = Theme::default();
        let stats = test_stats_no_failures();

        let widget = StatsFooterWidget::new(&stats, &theme);
        let area = Rect::new(0, 0, 60, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain the zero failure count
        assert!(content.contains("Failed:"), "Should show Failed label");
    }

    // ============================================
    // Constants Tests
    // ============================================

    #[test]
    fn stats_footer_height_is_correct() {
        // 1 top border + 1 content line + 1 bottom border = 3
        assert_eq!(STATS_FOOTER_HEIGHT, 3);
    }

    #[test]
    fn total_label_is_correct() {
        assert_eq!(TOTAL_LABEL, "Total: ");
    }

    #[test]
    fn sent_label_is_correct() {
        assert_eq!(SENT_LABEL, "Sent: ");
    }

    #[test]
    fn failed_label_is_correct() {
        assert_eq!(FAILED_LABEL, "Failed: ");
    }

    #[test]
    fn separator_is_correct() {
        assert_eq!(SEPARATOR, "  |  ");
    }
}
