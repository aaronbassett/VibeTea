//! Reusable TUI widget components for VibeTea Monitor.
//!
//! This module contains custom widgets built on top of [`ratatui`] that provide
//! the visual components of the monitor interface. Each widget is self-contained
//! and follows ratatui's [`Widget`] trait pattern where applicable.
//!
//! # Widget Catalog
//!
//! ## Branding
//! - [`logo`]: ASCII art logo with gradient animation
//!
//! ## Setup Flow
//! - [`setup_form`]: Server URL and registration code input form
//!
//! ## Main Interface
//! - [`header`]: Connection status bar with server info and indicators
//! - [`event_stream`]: Scrollable list of session events with filtering
//! - [`credentials`]: Device credentials display (public key, device name)
//! - [`stats_footer`]: Session statistics and keybinding hints
//!
//! ## Utility
//! - [`size_warning`]: Terminal size warning when dimensions are too small
//!
//! # Design Principles
//!
//! - Widgets are stateless where possible; state lives in the App
//! - Each widget handles its own layout within its allocated area
//! - Consistent color theming via shared color constants
//! - Responsive design adapts to available terminal space

// TODO: Uncomment these module declarations as implementations are added
pub mod credentials;
pub mod event_stream;
pub mod header;
pub mod logo;
pub mod setup_form;
// pub mod size_warning;
pub mod stats_footer;

// Re-exports will be added as modules are implemented
pub use credentials::{CredentialsWidget, CREDENTIALS_HEIGHT};
pub use event_stream::EventStreamWidget;
pub use header::{header_height, ConnectionStatusWidget, HeaderWidget};
pub use logo::{logo_height, LogoVariant, LogoWidget, COMPACT_LOGO_HEIGHT, FULL_LOGO_HEIGHT, TEXT_LOGO_HEIGHT};
pub use setup_form::{validate_session_name, SetupFormWidget};
// pub use size_warning::SizeWarningWidget;
pub use stats_footer::{StatsFooterWidget, STATS_FOOTER_HEIGHT};
