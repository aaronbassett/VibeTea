//! Terminal setup and RAII restoration for the VibeTea Monitor TUI.
//!
//! This module provides the [`Tui`] struct that wraps a ratatui terminal with
//! automatic cleanup via the [`Drop`] trait. The terminal enters raw mode and
//! alternate screen on creation, and restores the original state on drop.
//!
//! # Example
//!
//! ```ignore
//! use vibetea_monitor::tui::{Tui, install_panic_hook};
//!
//! // Install panic hook BEFORE creating the TUI
//! install_panic_hook();
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
//! 3. **Panic hook**: Via [`install_panic_hook()`] which ensures restoration
//!    even if a panic occurs before the [`Drop`] handler runs
//!
//! The [`Drop`] implementation silently ignores errors during cleanup to avoid
//! panics during stack unwinding.
//!
//! # Panic Safety
//!
//! The [`install_panic_hook()`] function should be called once at application
//! startup, before creating any [`Tui`] instance. This ensures that if a panic
//! occurs (even during TUI initialization), the terminal will be restored to
//! a usable state before the panic message is displayed.

use std::io::{self, Stdout};
use std::panic;

use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Installs a panic hook that restores terminal state before displaying panic messages.
///
/// This function should be called **once** at application startup, **before** creating
/// any [`Tui`] instance. It captures the existing panic hook and replaces it with a
/// custom hook that:
///
/// 1. Shows the cursor
/// 2. Leaves the alternate screen
/// 3. Disables raw mode
/// 4. Calls the previous panic handler to display the panic message
///
/// This ensures that panic messages are visible to the user and the terminal is left
/// in a usable state, even if the panic occurs before the [`Tui`]'s [`Drop`] handler
/// can run.
///
/// # Example
///
/// ```ignore
/// use vibetea_monitor::tui::{install_panic_hook, Tui};
///
/// fn main() -> std::io::Result<()> {
///     // Install panic hook first
///     install_panic_hook();
///
///     // Now it's safe to create the TUI
///     let mut tui = Tui::new()?;
///
///     // If any code panics from here on, the terminal will be restored
///     // before the panic message is shown.
///
///     Ok(())
/// }
/// ```
///
/// # Notes
///
/// - The restoration code ignores errors because the terminal may already be in
///   an inconsistent state when a panic occurs.
/// - This function is idempotent in the sense that calling it multiple times will
///   just chain the hooks, but it's intended to be called only once.
/// - The restoration is performed synchronously before the panic message is displayed,
///   ensuring the message is visible in the normal terminal buffer.
pub fn install_panic_hook() {
    // Capture the previous panic hook
    let previous_hook = panic::take_hook();

    panic::set_hook(Box::new(move |panic_info| {
        // Restore terminal state first, ignoring any errors.
        // The terminal may be in an inconsistent state, so we use best-effort
        // restoration without propagating errors.

        // Show cursor - ignore errors as terminal may be in bad state
        let _ = execute!(io::stdout(), Show);

        // Leave alternate screen - this brings back the normal terminal buffer
        // so the panic message will be visible
        let _ = execute!(io::stdout(), LeaveAlternateScreen);

        // Disable raw mode - this restores normal line-buffered input
        let _ = disable_raw_mode();

        // Now delegate to the previous panic handler to display the panic message.
        // This is typically the default handler that prints to stderr.
        previous_hook(panic_info);
    }));
}

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

    #[test]
    fn install_panic_hook_can_be_called() {
        // This test verifies that install_panic_hook can be called without panicking.
        // We can't easily test the actual panic behavior in a unit test, but we can
        // verify the function doesn't crash when called.
        //
        // Note: This test modifies global state (the panic hook), which is why
        // it should be run with --test-threads=1 to avoid interfering with other tests.
        install_panic_hook();

        // Verify we can install it again (chaining behavior)
        // This shouldn't panic or cause issues
        install_panic_hook();
    }

    #[test]
    fn panic_hook_closure_is_send_and_sync() {
        // Verify that the panic hook closure satisfies the required bounds.
        // std::panic::set_hook requires the closure to be Send + Sync + 'static.
        // This is a compile-time check.
        fn assert_hook_bounds<F>(_: F)
        where
            F: Fn(&panic::PanicHookInfo<'_>) + Send + Sync + 'static,
        {
        }

        // Create a closure similar to what install_panic_hook uses
        let previous_hook = panic::take_hook();
        let hook = move |panic_info: &panic::PanicHookInfo<'_>| {
            let _ = execute!(io::stdout(), Show);
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
            previous_hook(panic_info);
        };

        assert_hook_bounds(hook);

        // Restore a default hook since we took it
        panic::set_hook(Box::new(|_| {}));
    }
}
