//! ASCII art logo widget for the VibeTea Monitor TUI.
//!
//! This module provides a widget for displaying a stylized VibeTea ASCII logo
//! in the header. The logo gracefully degrades based on available terminal width,
//! supporting both unicode and ASCII fallback modes.
//!
//! # Requirements Compliance
//!
//! - **US8**: Display stylized VibeTea ASCII logo in header
//! - **US8 acceptance criteria**:
//!   - Multi-line ASCII art "VibeTea" with decorative elements (full mode)
//!   - Graceful degradation for narrow terminals (shorter version or text-only)
//!   - Support for both unicode and ASCII fallback modes
//!
//! # Layout Modes
//!
//! The logo widget adapts based on terminal width:
//!
//! - **Full (>= 60 columns)**: Multi-line ASCII art with decorative elements
//! - **Compact (>= 30 columns)**: Single-line stylized version
//! - **Text (< 30 columns)**: Plain "VibeTea" text with styling
//!
//! # Symbol Support
//!
//! The widget renders differently based on the symbol set:
//!
//! - **Unicode**: Uses decorative unicode characters for borders and accents
//! - **ASCII**: Uses plain ASCII characters for maximum compatibility
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::widgets::LogoWidget;
//! use vibetea_monitor::tui::app::{Theme, Symbols};
//!
//! let theme = Theme::default();
//! let symbols = Symbols::detect();
//!
//! let widget = LogoWidget::new(&theme, &symbols);
//! frame.render_widget(widget, logo_area);
//! ```

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::tui::app::{Symbols, Theme, UNICODE_SYMBOLS};

// =============================================================================
// Layout Constants
// =============================================================================

/// Minimum width required for the full ASCII art logo.
const FULL_LOGO_MIN_WIDTH: u16 = 60;

/// Minimum width required for the compact logo.
const COMPACT_LOGO_MIN_WIDTH: u16 = 30;

/// Height required for the full ASCII art logo (number of lines).
pub const FULL_LOGO_HEIGHT: u16 = 5;

/// Height required for the compact logo (single line).
pub const COMPACT_LOGO_HEIGHT: u16 = 1;

/// Height required for the text-only logo (single line).
pub const TEXT_LOGO_HEIGHT: u16 = 1;

// =============================================================================
// ASCII Art Logos
// =============================================================================

/// Full ASCII art logo for wide terminals (unicode version).
///
/// This is the primary branding display using unicode box-drawing characters
/// for a polished appearance.
const FULL_LOGO_UNICODE: &[&str] = &[
    "╭─────────────────────────────────────╮",
    "│  ╦  ╦╦╔╗ ╔═╗╔╦╗╔═╗╔═╗  ☕           │",
    "│  ╚╗╔╝║╠╩╗║╣  ║ ║╣ ╠═╣              │",
    "│   ╚╝ ╩╚═╝╚═╝ ╩ ╚═╝╩ ╩  Monitor     │",
    "╰─────────────────────────────────────╯",
];

/// Full ASCII art logo for wide terminals (ASCII version).
///
/// Fallback for terminals that don't support unicode box-drawing characters.
const FULL_LOGO_ASCII: &[&str] = &[
    "+-------------------------------------+",
    "|  V I B E T E A   [~]                |",
    "|  ~~~~~~~~~~~~~~~                    |",
    "|                    Monitor          |",
    "+-------------------------------------+",
];

/// Compact logo for medium-width terminals (unicode version).
const COMPACT_LOGO_UNICODE: &str = "☕ VibeTea Monitor";

/// Compact logo for medium-width terminals (ASCII version).
const COMPACT_LOGO_ASCII: &str = "[~] VibeTea Monitor";

/// Text-only logo for narrow terminals.
const TEXT_LOGO: &str = "VibeTea";

// =============================================================================
// Logo Variant
// =============================================================================

/// The variant of logo to render based on available space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogoVariant {
    /// Full multi-line ASCII art logo.
    Full,
    /// Single-line compact logo.
    Compact,
    /// Plain text logo.
    Text,
}

impl LogoVariant {
    /// Determines the appropriate logo variant based on available width.
    ///
    /// # Arguments
    ///
    /// * `width` - The available horizontal space in columns
    ///
    /// # Returns
    ///
    /// The most detailed logo variant that fits within the given width.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::logo::LogoVariant;
    ///
    /// assert_eq!(LogoVariant::from_width(80), LogoVariant::Full);
    /// assert_eq!(LogoVariant::from_width(40), LogoVariant::Compact);
    /// assert_eq!(LogoVariant::from_width(20), LogoVariant::Text);
    /// ```
    #[must_use]
    pub fn from_width(width: u16) -> Self {
        if width >= FULL_LOGO_MIN_WIDTH {
            LogoVariant::Full
        } else if width >= COMPACT_LOGO_MIN_WIDTH {
            LogoVariant::Compact
        } else {
            LogoVariant::Text
        }
    }

    /// Returns the height required for this logo variant.
    ///
    /// # Returns
    ///
    /// The number of rows needed to render this logo variant.
    #[must_use]
    pub fn height(self) -> u16 {
        match self {
            LogoVariant::Full => FULL_LOGO_HEIGHT,
            LogoVariant::Compact => COMPACT_LOGO_HEIGHT,
            LogoVariant::Text => TEXT_LOGO_HEIGHT,
        }
    }

    /// Returns the minimum width required for this logo variant.
    ///
    /// # Returns
    ///
    /// The minimum number of columns needed to render this logo variant.
    #[must_use]
    pub fn min_width(self) -> u16 {
        match self {
            LogoVariant::Full => FULL_LOGO_MIN_WIDTH,
            LogoVariant::Compact => COMPACT_LOGO_MIN_WIDTH,
            LogoVariant::Text => TEXT_LOGO.len() as u16,
        }
    }
}

// =============================================================================
// LogoWidget
// =============================================================================

/// Widget for displaying the VibeTea ASCII art logo.
///
/// Renders a stylized logo that adapts to the available terminal space.
/// The logo gracefully degrades from a full multi-line ASCII art display
/// to a compact single-line version to plain text based on width constraints.
///
/// # Symbol Support
///
/// The widget respects the provided [`Symbols`] configuration:
/// - Unicode symbols result in decorative unicode characters
/// - ASCII symbols result in plain ASCII fallback characters
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::LogoWidget;
/// use vibetea_monitor::tui::app::{Theme, Symbols};
///
/// let theme = Theme::default();
/// let symbols = Symbols::detect();
///
/// let widget = LogoWidget::new(&theme, &symbols);
/// frame.render_widget(widget, logo_area);
/// ```
#[derive(Debug)]
pub struct LogoWidget<'a> {
    /// Reference to the theme for styling.
    theme: &'a Theme,
    /// Reference to the symbol set (unicode or ASCII).
    symbols: &'a Symbols,
}

impl<'a> LogoWidget<'a> {
    /// Creates a new `LogoWidget`.
    ///
    /// # Arguments
    ///
    /// * `theme` - Theme configuration for colors and styles
    /// * `symbols` - Symbol set (unicode or ASCII) for rendering
    ///
    /// # Example
    ///
    /// ```ignore
    /// use vibetea_monitor::tui::widgets::LogoWidget;
    /// use vibetea_monitor::tui::app::{Theme, Symbols};
    ///
    /// let theme = Theme::default();
    /// let symbols = Symbols::detect();
    ///
    /// let widget = LogoWidget::new(&theme, &symbols);
    /// ```
    #[must_use]
    pub fn new(theme: &'a Theme, symbols: &'a Symbols) -> Self {
        Self { theme, symbols }
    }

    /// Returns whether unicode symbols are being used.
    fn is_unicode(&self) -> bool {
        // Check if we're using unicode by comparing with the known unicode symbol
        self.symbols.connected == UNICODE_SYMBOLS.connected
    }

    /// Returns the full logo lines based on symbol set.
    fn full_logo_lines(&self) -> &'static [&'static str] {
        if self.is_unicode() {
            FULL_LOGO_UNICODE
        } else {
            FULL_LOGO_ASCII
        }
    }

    /// Returns the compact logo text based on symbol set.
    fn compact_logo(&self) -> &'static str {
        if self.is_unicode() {
            COMPACT_LOGO_UNICODE
        } else {
            COMPACT_LOGO_ASCII
        }
    }

    /// Renders the full ASCII art logo.
    fn render_full(&self, area: Rect, buf: &mut Buffer) {
        let lines = self.full_logo_lines();
        let style = self.theme.title;

        // Center the logo vertically if there's extra space
        let logo_height = lines.len() as u16;
        let start_y = if area.height > logo_height {
            area.y + (area.height - logo_height) / 2
        } else {
            area.y
        };

        for (i, line) in lines.iter().enumerate() {
            let y = start_y + i as u16;
            if y >= area.y + area.height {
                break;
            }

            // Center horizontally
            let line_width = line.chars().count() as u16;
            let x = if area.width > line_width {
                area.x + (area.width - line_width) / 2
            } else {
                area.x
            };

            // Truncate if necessary
            let available_width = (area.x + area.width).saturating_sub(x) as usize;
            let display_line: String = line.chars().take(available_width).collect();

            buf.set_string(x, y, &display_line, style);
        }
    }

    /// Renders the compact single-line logo.
    fn render_compact(&self, area: Rect, buf: &mut Buffer) {
        let logo = self.compact_logo();
        let style = self.theme.title;

        // Center vertically
        let y = area.y + area.height / 2;

        // Center horizontally
        let logo_width = logo.chars().count() as u16;
        let x = if area.width > logo_width {
            area.x + (area.width - logo_width) / 2
        } else {
            area.x
        };

        // Truncate if necessary
        let available_width = (area.x + area.width).saturating_sub(x) as usize;
        let display_logo: String = logo.chars().take(available_width).collect();

        buf.set_string(x, y, &display_logo, style);
    }

    /// Renders the text-only logo.
    fn render_text(&self, area: Rect, buf: &mut Buffer) {
        let style = self.theme.title;

        // Center vertically
        let y = area.y + area.height / 2;

        // Center horizontally
        let logo_width = TEXT_LOGO.len() as u16;
        let x = if area.width > logo_width {
            area.x + (area.width - logo_width) / 2
        } else {
            area.x
        };

        // Truncate if necessary
        let available_width = (area.x + area.width).saturating_sub(x) as usize;
        let display_logo: String = TEXT_LOGO.chars().take(available_width).collect();

        buf.set_string(x, y, &display_logo, style);
    }
}

impl Widget for LogoWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let variant = LogoVariant::from_width(area.width);

        match variant {
            LogoVariant::Full => {
                // Only render full if we have enough height
                if area.height >= FULL_LOGO_HEIGHT {
                    self.render_full(area, buf);
                } else if area.height >= COMPACT_LOGO_HEIGHT {
                    self.render_compact(area, buf);
                } else {
                    self.render_text(area, buf);
                }
            }
            LogoVariant::Compact => {
                self.render_compact(area, buf);
            }
            LogoVariant::Text => {
                self.render_text(area, buf);
            }
        }
    }
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Returns the optimal height for the logo based on available width.
///
/// This function helps layout code determine how much vertical space
/// to allocate for the logo widget.
///
/// # Arguments
///
/// * `width` - The available horizontal space in columns
///
/// # Returns
///
/// The optimal height in rows for the logo at the given width.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::widgets::logo_height;
///
/// let height = logo_height(80); // Returns 5 for full logo
/// let height = logo_height(40); // Returns 1 for compact logo
/// ```
#[must_use]
pub fn logo_height(width: u16) -> u16 {
    LogoVariant::from_width(width).height()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::app::{ASCII_SYMBOLS, UNICODE_SYMBOLS};

    // =========================================================================
    // LogoVariant Tests
    // =========================================================================

    #[test]
    fn logo_variant_from_width_full() {
        assert_eq!(LogoVariant::from_width(60), LogoVariant::Full);
        assert_eq!(LogoVariant::from_width(80), LogoVariant::Full);
        assert_eq!(LogoVariant::from_width(100), LogoVariant::Full);
        assert_eq!(LogoVariant::from_width(200), LogoVariant::Full);
    }

    #[test]
    fn logo_variant_from_width_compact() {
        assert_eq!(LogoVariant::from_width(30), LogoVariant::Compact);
        assert_eq!(LogoVariant::from_width(40), LogoVariant::Compact);
        assert_eq!(LogoVariant::from_width(50), LogoVariant::Compact);
        assert_eq!(LogoVariant::from_width(59), LogoVariant::Compact);
    }

    #[test]
    fn logo_variant_from_width_text() {
        assert_eq!(LogoVariant::from_width(0), LogoVariant::Text);
        assert_eq!(LogoVariant::from_width(10), LogoVariant::Text);
        assert_eq!(LogoVariant::from_width(20), LogoVariant::Text);
        assert_eq!(LogoVariant::from_width(29), LogoVariant::Text);
    }

    #[test]
    fn logo_variant_height_correct() {
        assert_eq!(LogoVariant::Full.height(), FULL_LOGO_HEIGHT);
        assert_eq!(LogoVariant::Compact.height(), COMPACT_LOGO_HEIGHT);
        assert_eq!(LogoVariant::Text.height(), TEXT_LOGO_HEIGHT);
    }

    #[test]
    fn logo_variant_min_width_correct() {
        assert_eq!(LogoVariant::Full.min_width(), FULL_LOGO_MIN_WIDTH);
        assert_eq!(LogoVariant::Compact.min_width(), COMPACT_LOGO_MIN_WIDTH);
        assert_eq!(LogoVariant::Text.min_width(), TEXT_LOGO.len() as u16);
    }

    #[test]
    fn logo_variant_is_debug() {
        let variant = LogoVariant::Full;
        let debug_str = format!("{:?}", variant);
        assert!(debug_str.contains("Full"));
    }

    #[test]
    #[allow(clippy::clone_on_copy)]
    fn logo_variant_is_clone_and_copy() {
        let variant = LogoVariant::Full;
        let cloned = variant.clone();
        let copied = variant;
        assert_eq!(variant, cloned);
        assert_eq!(variant, copied);
    }

    #[test]
    fn logo_variant_equality() {
        assert_eq!(LogoVariant::Full, LogoVariant::Full);
        assert_eq!(LogoVariant::Compact, LogoVariant::Compact);
        assert_eq!(LogoVariant::Text, LogoVariant::Text);
        assert_ne!(LogoVariant::Full, LogoVariant::Compact);
        assert_ne!(LogoVariant::Compact, LogoVariant::Text);
        assert_ne!(LogoVariant::Full, LogoVariant::Text);
    }

    // =========================================================================
    // LogoWidget Tests
    // =========================================================================

    #[test]
    fn logo_widget_can_be_created() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = LogoWidget::new(&theme, &symbols);
        assert!(format!("{:?}", widget).contains("LogoWidget"));
    }

    #[test]
    fn logo_widget_is_debug() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = LogoWidget::new(&theme, &symbols);
        let debug_str = format!("{:?}", widget);
        assert!(debug_str.contains("LogoWidget"));
    }

    #[test]
    fn logo_widget_detects_unicode_symbols() {
        let theme = Theme::default();
        let unicode_symbols = UNICODE_SYMBOLS;
        let ascii_symbols = ASCII_SYMBOLS;

        let unicode_widget = LogoWidget::new(&theme, &unicode_symbols);
        let ascii_widget = LogoWidget::new(&theme, &ascii_symbols);

        assert!(unicode_widget.is_unicode());
        assert!(!ascii_widget.is_unicode());
    }

    #[test]
    fn logo_widget_returns_correct_full_logo_lines() {
        let theme = Theme::default();

        let unicode_widget = LogoWidget::new(&theme, &UNICODE_SYMBOLS);
        let ascii_widget = LogoWidget::new(&theme, &ASCII_SYMBOLS);

        assert_eq!(unicode_widget.full_logo_lines(), FULL_LOGO_UNICODE);
        assert_eq!(ascii_widget.full_logo_lines(), FULL_LOGO_ASCII);
    }

    #[test]
    fn logo_widget_returns_correct_compact_logo() {
        let theme = Theme::default();

        let unicode_widget = LogoWidget::new(&theme, &UNICODE_SYMBOLS);
        let ascii_widget = LogoWidget::new(&theme, &ASCII_SYMBOLS);

        assert_eq!(unicode_widget.compact_logo(), COMPACT_LOGO_UNICODE);
        assert_eq!(ascii_widget.compact_logo(), COMPACT_LOGO_ASCII);
    }

    #[test]
    fn logo_widget_renders_without_panic_all_widths() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        for width in [0, 10, 20, 30, 40, 50, 60, 80, 100, 120] {
            let widget = LogoWidget::new(&theme, &symbols);
            let area = Rect::new(0, 0, width, 10);
            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
            // Should not panic
        }
    }

    #[test]
    fn logo_widget_renders_without_panic_all_heights() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        for height in [0, 1, 2, 3, 4, 5, 10, 20] {
            let widget = LogoWidget::new(&theme, &symbols);
            let area = Rect::new(0, 0, 80, height);
            let mut buf = Buffer::empty(area);
            widget.render(area, &mut buf);
            // Should not panic
        }
    }

    #[test]
    fn logo_widget_handles_zero_area() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Zero width
        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 0, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic

        // Zero height
        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 80, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic

        // Both zero
        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 0, 0);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf); // Should not panic
    }

    #[test]
    fn logo_widget_renders_full_logo_unicode() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain parts of the unicode logo
        assert!(
            content.contains("VIBETEA") || content.contains("╦") || content.contains("Monitor"),
            "Full unicode logo should contain branding elements"
        );
    }

    #[test]
    fn logo_widget_renders_full_logo_ascii() {
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should contain parts of the ASCII logo
        assert!(
            content.contains("VIBETEA") || content.contains("Monitor"),
            "Full ASCII logo should contain branding elements"
        );
    }

    #[test]
    fn logo_widget_renders_compact_logo_unicode() {
        let theme = Theme::default();
        let symbols = UNICODE_SYMBOLS;

        let widget = LogoWidget::new(&theme, &symbols);
        // Width 40 should trigger compact mode
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("VibeTea"),
            "Compact logo should contain 'VibeTea'"
        );
    }

    #[test]
    fn logo_widget_renders_compact_logo_ascii() {
        let theme = Theme::default();
        let symbols = ASCII_SYMBOLS;

        let widget = LogoWidget::new(&theme, &symbols);
        // Width 40 should trigger compact mode
        let area = Rect::new(0, 0, 40, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("VibeTea"),
            "Compact ASCII logo should contain 'VibeTea'"
        );
    }

    #[test]
    fn logo_widget_renders_text_logo() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = LogoWidget::new(&theme, &symbols);
        // Width 20 should trigger text mode
        let area = Rect::new(0, 0, 20, 3);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        assert!(
            content.contains("VibeTea"),
            "Text logo should contain 'VibeTea'"
        );
    }

    #[test]
    fn logo_widget_degrades_gracefully_with_limited_height() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        // Wide enough for full logo, but not tall enough
        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 80, 2);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let content: String = buf.content.iter().map(|cell| cell.symbol()).collect();

        // Should fall back to compact/text mode
        assert!(
            content.contains("VibeTea"),
            "Should degrade to showing VibeTea text"
        );
    }

    #[test]
    fn logo_widget_with_monochrome_theme() {
        let theme = Theme::monochrome();
        let symbols = Symbols::default();

        let widget = LogoWidget::new(&theme, &symbols);
        let area = Rect::new(0, 0, 80, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        // Should not panic with monochrome theme
    }

    #[test]
    fn logo_widget_renders_at_offset_position() {
        let theme = Theme::default();
        let symbols = Symbols::default();

        let widget = LogoWidget::new(&theme, &symbols);
        // Area starting at non-zero position
        let area = Rect::new(10, 5, 60, 10);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 20));
        widget.render(area, &mut buf);

        // Should not panic and should render within bounds
    }

    // =========================================================================
    // logo_height Tests
    // =========================================================================

    #[test]
    fn logo_height_returns_correct_values() {
        assert_eq!(logo_height(80), FULL_LOGO_HEIGHT);
        assert_eq!(logo_height(60), FULL_LOGO_HEIGHT);
        assert_eq!(logo_height(59), COMPACT_LOGO_HEIGHT);
        assert_eq!(logo_height(40), COMPACT_LOGO_HEIGHT);
        assert_eq!(logo_height(30), COMPACT_LOGO_HEIGHT);
        assert_eq!(logo_height(29), TEXT_LOGO_HEIGHT);
        assert_eq!(logo_height(10), TEXT_LOGO_HEIGHT);
        assert_eq!(logo_height(0), TEXT_LOGO_HEIGHT);
    }

    #[test]
    fn logo_height_handles_boundary_values() {
        // At exact thresholds
        assert_eq!(logo_height(FULL_LOGO_MIN_WIDTH), FULL_LOGO_HEIGHT);
        assert_eq!(logo_height(COMPACT_LOGO_MIN_WIDTH), COMPACT_LOGO_HEIGHT);

        // Just below thresholds
        assert_eq!(logo_height(FULL_LOGO_MIN_WIDTH - 1), COMPACT_LOGO_HEIGHT);
        assert_eq!(logo_height(COMPACT_LOGO_MIN_WIDTH - 1), TEXT_LOGO_HEIGHT);
    }

    // =========================================================================
    // Logo Content Tests
    // =========================================================================

    #[test]
    fn full_logo_unicode_has_correct_structure() {
        assert_eq!(FULL_LOGO_UNICODE.len(), FULL_LOGO_HEIGHT as usize);
        for line in FULL_LOGO_UNICODE {
            assert!(!line.is_empty(), "Logo lines should not be empty");
        }
    }

    #[test]
    fn full_logo_ascii_has_correct_structure() {
        assert_eq!(FULL_LOGO_ASCII.len(), FULL_LOGO_HEIGHT as usize);
        for line in FULL_LOGO_ASCII {
            assert!(!line.is_empty(), "Logo lines should not be empty");
        }
    }

    #[test]
    fn compact_logos_contain_branding() {
        assert!(COMPACT_LOGO_UNICODE.contains("VibeTea"));
        assert!(COMPACT_LOGO_ASCII.contains("VibeTea"));
    }

    #[test]
    fn text_logo_contains_branding() {
        assert!(TEXT_LOGO.contains("VibeTea"));
    }

    #[test]
    fn logo_widths_are_reasonable() {
        // Full logo should fit in its minimum width
        for line in FULL_LOGO_UNICODE {
            assert!(
                line.chars().count() <= FULL_LOGO_MIN_WIDTH as usize,
                "Unicode logo line '{}' exceeds minimum width",
                line
            );
        }
        for line in FULL_LOGO_ASCII {
            assert!(
                line.chars().count() <= FULL_LOGO_MIN_WIDTH as usize,
                "ASCII logo line '{}' exceeds minimum width",
                line
            );
        }

        // Compact logos should fit in their minimum width
        assert!(
            COMPACT_LOGO_UNICODE.chars().count() <= COMPACT_LOGO_MIN_WIDTH as usize,
            "Compact unicode logo exceeds minimum width"
        );
        assert!(
            COMPACT_LOGO_ASCII.chars().count() <= COMPACT_LOGO_MIN_WIDTH as usize,
            "Compact ASCII logo exceeds minimum width"
        );
    }
}
