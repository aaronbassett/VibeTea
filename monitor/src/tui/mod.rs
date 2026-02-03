//! Terminal User Interface for VibeTea Monitor.
//!
//! This module provides a TUI built with [`ratatui`] for monitoring Claude Code
//! sessions in real-time. The interface displays session events, connection status,
//! and provides interactive controls for managing the monitor.
//!
//! # Architecture
//!
//! The TUI follows a Model-View-Controller pattern:
//!
//! - **App** (`app`): Application state and business logic (Model/Controller)
//! - **UI** (`ui`): Layout and rendering logic (View)
//! - **Input** (`input`): Keyboard and event handling
//! - **Terminal** (`terminal`): Terminal setup, teardown, and raw mode management
//! - **Widgets** (`widgets`): Reusable UI components
//!
//! # Usage
//!
//! ```ignore
//! use vibetea_monitor::tui::App;
//!
//! let mut app = App::new(config)?;
//! app.run().await?;
//! ```
//!
//! # Submodules
//!
//! - [`app`]: Application state, event loop, and mode management
//! - [`ui`]: Frame rendering and layout composition
//! - [`input`]: Keyboard event processing and action dispatch
//! - [`terminal`]: Terminal initialization and cleanup with panic handling
//! - [`widgets`]: Reusable TUI components (logo, forms, event stream, etc.)

// TODO: Uncomment these module declarations as implementations are added
pub mod app;
// pub mod input;
pub mod terminal;
// pub mod ui;
pub mod widgets;

// Re-exports for convenient access to core TUI types
pub use app::{ConnectionStatus, EventStats, TuiEvent};
pub use terminal::{install_panic_hook, Tui};
