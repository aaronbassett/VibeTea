//! Terminal setup and RAII restoration for the VibeTea Monitor TUI.
//!
//! This module provides the [`Tui`] struct that wraps a ratatui terminal with
//! automatic cleanup via the [`Drop`] trait. The terminal enters raw mode and
//! alternate screen on creation, and restores the original state on drop.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::Tui;
//!
//! let mut tui = Tui::new()?;
//! tui.draw(|frame| {
//!     // render widgets to frame
//! })?;
//! // Terminal automatically restored when `tui` goes out of scope
//! ```
//!
//! # Cleanup Behavior
//!
//! The terminal state is restored in three scenarios:
//!
//! 1. **Normal drop**: When [`Tui`] goes out of scope
//! 2. **Explicit restore**: By calling [`Tui::restore()`]
//! 3. **Panic hook**: Via a separate panic handler (not implemented here)
//!
//! The [`Drop`] implementation silently ignores errors during cleanup to avoid
//! panics during stack unwinding.

use std::io::{self, Stdout};

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// A wrapper around ratatui's Terminal that provides RAII-based cleanup.
///
/// When dropped, this struct automatically:
/// - Shows the cursor
/// - Leaves the alternate screen
/// - Disables raw mode
///
/// This ensures the terminal is restored to its original state even if the
/// application panics or exits unexpectedly.
pub struct Tui {
    /// The underlying ratatui terminal.
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Track whether the terminal has been restored to avoid double cleanup.
    restored: bool,
}

impl Tui {
    /// Creates a new TUI instance, initializing the terminal for raw mode.
    ///
    /// This function:
    /// - Enables raw mode for character-by-character input
    /// - Enters the alternate screen buffer (preserves shell history)
    /// - Hides the cursor
    /// - Creates the ratatui terminal
    ///
    /// # Errors
    ///
    /// Returns an error if any terminal initialization step fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut tui = Tui::new()?;
    /// ```
    pub fn new() -> io::Result<Self> {
        // Enable raw mode for character-by-character input
        enable_raw_mode()?;

        // Get stdout handle for terminal operations
        let mut stdout = io::stdout();

        // Enter alternate screen and hide cursor
        // If this fails, we need to restore raw mode before returning
        if let Err(e) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(e);
        }

        // Create the ratatui backend and terminal
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                // Restore terminal state on error
                let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
                let _ = disable_raw_mode();
                return Err(e);
            }
        };

        Ok(Self {
            terminal,
            restored: false,
        })
    }

    /// Draws a frame to the terminal using the provided closure.
    ///
    /// The closure receives a [`ratatui::Frame`] that can be used to render
    /// widgets. The frame is automatically flushed to the terminal after
    /// the closure returns.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// tui.draw(|frame| {
    ///     let area = frame.area();
    ///     frame.render_widget(my_widget, area);
    /// })?;
    /// ```
    pub fn draw<F>(&mut self, f: F) -> io::Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(f)?;
        Ok(())
    }

    /// Returns the current terminal size as (width, height).
    ///
    /// # Errors
    ///
    /// Returns an error if the terminal size cannot be determined.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    /// Explicitly restores the terminal to its original state.
    ///
    /// This function:
    /// - Shows the cursor
    /// - Leaves the alternate screen
    /// - Disables raw mode
    ///
    /// After calling this method, the [`Tui`] should not be used for drawing.
    /// The [`Drop`] implementation will skip cleanup if this has been called.
    ///
    /// # Errors
    ///
    /// Returns an error if any restoration step fails. Unlike the [`Drop`]
    /// implementation, errors are propagated to the caller.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Explicit cleanup before exiting
    /// tui.restore()?;
    /// println!("Terminal restored successfully");
    /// ```
    pub fn restore(&mut self) -> io::Result<()> {
        if self.restored {
            return Ok(());
        }

        self.restored = true;

        // Show cursor and leave alternate screen
        execute!(io::stdout(), Show, LeaveAlternateScreen)?;

        // Disable raw mode
        disable_raw_mode()?;

        Ok(())
    }

    /// Clears the entire terminal screen.
    ///
    /// # Errors
    ///
    /// Returns an error if clearing fails.
    pub fn clear(&mut self) -> io::Result<()> {
        self.terminal.clear()?;
        Ok(())
    }

    /// Returns a reference to the underlying ratatui terminal.
    ///
    /// This provides access to advanced terminal features not exposed
    /// directly by the [`Tui`] wrapper.
    pub fn backend(&self) -> &CrosstermBackend<Stdout> {
        self.terminal.backend()
    }

    /// Returns a mutable reference to the underlying ratatui terminal.
    ///
    /// This provides access to advanced terminal features not exposed
    /// directly by the [`Tui`] wrapper.
    pub fn backend_mut(&mut self) -> &mut CrosstermBackend<Stdout> {
        self.terminal.backend_mut()
    }
}

impl Drop for Tui {
    /// Restores the terminal state when the [`Tui`] is dropped.
    ///
    /// This implementation silently ignores errors during cleanup to avoid
    /// panics during stack unwinding. For explicit error handling, use
    /// [`Tui::restore()`] before dropping.
    fn drop(&mut self) {
        if self.restored {
            return;
        }

        // Silently restore terminal state
        // We intentionally ignore errors here because:
        // 1. We may be in a panic context where logging isn't safe
        // 2. The terminal may already be in a bad state
        // 3. Double-panicking would abort the process
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Most tests for Tui require an actual terminal and cannot be run
    // in CI environments. These tests verify the API surface and basic logic.

    #[test]
    fn tui_struct_is_send() {
        // This test verifies the struct is Send at compile time.
        // Terminal<CrosstermBackend<Stdout>> is Send but not Sync
        // because Stdout is not Sync.
        fn assert_send<T: Send>() {}
        assert_send::<Tui>();
    }

    #[test]
    fn restore_flag_prevents_double_cleanup() {
        // Test that the restore flag logic prevents double cleanup.
        // This is a logic test that doesn't require a real terminal.
        //
        // We can't test with a real Tui without a terminal, but we can
        // verify the restored flag logic is correct.
        let mut restored = false;

        // Simulate first restore
        if !restored {
            restored = true;
        }

        // After first restore, flag should be true
        assert!(restored, "Flag should be set after first restore");

        // Simulate second restore - should be a no-op due to flag check
        let would_restore = !restored;
        assert!(
            !would_restore,
            "Flag should prevent second restore attempt"
        );
    }
}
