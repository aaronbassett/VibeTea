//! VibeTea Monitor - Claude Code session watcher.
//!
//! This binary watches Claude Code session files and forwards privacy-filtered
//! events to the VibeTea server.
//!
//! # Commands
//!
//! - `vibetea-monitor init`: Generate Ed25519 keypair for server authentication
//! - `vibetea-monitor run`: Start the monitor daemon
//!
//! # Environment Variables
//!
//! See the [`config`] module for available configuration options.

use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use directories::BaseDirs;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use vibetea_monitor::config::Config;
use vibetea_monitor::crypto::Crypto;
use vibetea_monitor::sender::{Sender, SenderConfig};

/// Default key directory name relative to home.
const DEFAULT_KEY_DIR: &str = ".vibetea";

/// Graceful shutdown timeout.
const SHUTDOWN_TIMEOUT_SECS: u64 = 5;

/// CLI command.
#[derive(Debug)]
enum Command {
    /// Initialize keypair.
    Init {
        /// Force overwrite existing keys.
        force: bool,
    },
    /// Run the monitor.
    Run,
    /// Show help.
    Help,
    /// Show version.
    Version,
}

fn main() -> Result<()> {
    // Parse command line arguments
    let command = parse_args()?;

    match command {
        Command::Init { force } => run_init(force),
        Command::Run => {
            // Initialize async runtime for the run command
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("Failed to create tokio runtime")?;

            runtime.block_on(run_monitor())
        }
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::Version => {
            print_version();
            Ok(())
        }
    }
}

/// Parses command line arguments into a Command.
fn parse_args() -> Result<Command> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Ok(Command::Help);
    }

    match args[1].as_str() {
        "init" => {
            let force = args.iter().any(|a| a == "--force" || a == "-f");
            Ok(Command::Init { force })
        }
        "run" => Ok(Command::Run),
        "help" | "--help" | "-h" => Ok(Command::Help),
        "version" | "--version" | "-V" => Ok(Command::Version),
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            print_help();
            std::process::exit(1);
        }
    }
}

/// Prints the help message.
fn print_help() {
    println!(
        r#"VibeTea Monitor - Claude Code session watcher

USAGE:
    vibetea-monitor <COMMAND>

COMMANDS:
    init    Generate Ed25519 keypair for server authentication
            Options:
              --force, -f    Overwrite existing keys

    run     Start the monitor daemon

    help    Show this help message

    version Show version information

ENVIRONMENT VARIABLES:
    VIBETEA_SERVER_URL       Server URL (required for 'run')
    VIBETEA_SOURCE_ID        Monitor identifier (default: hostname)
    VIBETEA_KEY_PATH         Key directory (default: ~/.vibetea)
    VIBETEA_CLAUDE_DIR       Claude directory (default: ~/.claude)
    VIBETEA_BUFFER_SIZE      Event buffer size (default: 1000)
    VIBETEA_BASENAME_ALLOWLIST   Comma-separated file extensions to include

EXAMPLES:
    # Generate a new keypair
    vibetea-monitor init

    # Start the monitor
    export VIBETEA_SERVER_URL=https://vibetea.fly.dev
    vibetea-monitor run
"#
    );
}

/// Prints version information.
fn print_version() {
    println!(
        "vibetea-monitor {}",
        env!("CARGO_PKG_VERSION")
    );
}

/// Runs the init command to generate a new keypair.
fn run_init(force: bool) -> Result<()> {
    let key_dir = get_key_directory()?;

    // Check if keys already exist
    if Crypto::exists(&key_dir) && !force {
        eprintln!("Keys already exist at: {}", key_dir.display());
        eprintln!();
        eprint!("Overwrite existing keys? [y/N] ");
        io::stderr().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            eprintln!("Aborted.");
            return Ok(());
        }
    }

    // Generate and save keypair
    println!("Generating Ed25519 keypair...");
    let crypto = Crypto::generate();
    crypto
        .save(&key_dir)
        .context("Failed to save keypair")?;

    println!();
    println!("Keypair saved to: {}", key_dir.display());
    println!();
    println!("Public key (register with server):");
    println!();
    println!("  {}", crypto.public_key_base64());
    println!();
    println!("To register this monitor with the server, add to VIBETEA_PUBLIC_KEYS:");
    println!();
    println!(
        "  export VIBETEA_PUBLIC_KEYS=\"{}:{}\"",
        get_default_source_id(),
        crypto.public_key_base64()
    );

    Ok(())
}

/// Runs the monitor daemon.
async fn run_monitor() -> Result<()> {
    // Initialize logging
    init_logging();

    info!("Starting VibeTea Monitor");

    // Load configuration
    let config = Config::from_env().context("Failed to load configuration")?;

    info!(
        server_url = %config.server_url,
        source_id = %config.source_id,
        claude_dir = %config.claude_dir.display(),
        "Configuration loaded"
    );

    // Load cryptographic keys
    let crypto = Crypto::load(&config.key_path).context(format!(
        "Failed to load keys from {}. Run 'vibetea-monitor init' first.",
        config.key_path.display()
    ))?;

    info!(
        key_path = %config.key_path.display(),
        "Cryptographic keys loaded"
    );

    // Create sender
    let sender_config = SenderConfig::new(
        config.server_url.clone(),
        config.source_id.clone(),
        config.buffer_size,
    );
    let mut sender = Sender::new(sender_config, crypto);

    // TODO: Initialize file watcher and parser pipeline
    // For now, just wait for shutdown signal
    info!("Monitor running. Press Ctrl+C to stop.");

    // Wait for shutdown signal
    wait_for_shutdown().await;

    // Graceful shutdown
    info!("Shutting down...");
    let unflushed = sender.shutdown(Duration::from_secs(SHUTDOWN_TIMEOUT_SECS)).await;

    if unflushed > 0 {
        error!(unflushed_events = unflushed, "Some events could not be sent");
    }

    info!("Monitor stopped");
    Ok(())
}

/// Initializes the logging subsystem.
fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();
}

/// Waits for a shutdown signal (SIGINT or SIGTERM).
async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

/// Gets the key directory path.
fn get_key_directory() -> Result<PathBuf> {
    // Check for VIBETEA_KEY_PATH environment variable
    if let Ok(path) = std::env::var("VIBETEA_KEY_PATH") {
        return Ok(PathBuf::from(path));
    }

    // Default to ~/.vibetea
    let base_dirs = BaseDirs::new().context("Failed to determine home directory")?;
    Ok(base_dirs.home_dir().join(DEFAULT_KEY_DIR))
}

/// Gets the default source ID (hostname).
fn get_default_source_id() -> String {
    gethostname::gethostname()
        .into_string()
        .unwrap_or_else(|_| "unknown".to_string())
}
