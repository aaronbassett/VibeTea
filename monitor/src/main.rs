//! VibeTea Monitor - Claude Code session watcher.
//!
//! This binary watches Claude Code session files and forwards privacy-filtered
//! events to the VibeTea server.
//!
//! # Commands
//!
//! - `vibetea-monitor init`: Generate Ed25519 keypair for server authentication
//! - `vibetea-monitor export-key`: Export private key for GitHub Actions
//! - `vibetea-monitor run`: Start the monitor daemon
//!
//! # Environment Variables
//!
//! See the [`config`] module for available configuration options.

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use directories::BaseDirs;
use tokio::signal;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

use vibetea_monitor::config::Config;
use vibetea_monitor::crypto::{Crypto, KeySource};
use vibetea_monitor::parser::{ParsedEvent, ParsedEventKind, SessionParser};
use vibetea_monitor::privacy::{PrivacyConfig, PrivacyPipeline};
use vibetea_monitor::sender::{Sender, SenderConfig};
use vibetea_monitor::trackers::file_history_tracker::FileHistoryTracker;
use vibetea_monitor::trackers::skill_tracker::SkillTracker;
use vibetea_monitor::trackers::stats_tracker::{StatsEvent, StatsTracker};
use vibetea_monitor::trackers::todo_tracker::TodoTracker;
use vibetea_monitor::types::{
    AgentSpawnEvent, Event, EventPayload, EventType, FileChangeEvent, SessionAction,
    SkillInvocationEvent, TodoProgressEvent, ToolStatus,
};
use vibetea_monitor::watcher::{FileWatcher, WatchEvent};

/// Default key directory name relative to home.
const DEFAULT_KEY_DIR: &str = ".vibetea";

/// Graceful shutdown timeout.
const SHUTDOWN_TIMEOUT_SECS: u64 = 5;

/// Namespace UUID for generating deterministic session IDs from malformed paths.
/// This ensures the same path always maps to the same session ID across restarts.
const VIBETEA_NAMESPACE: Uuid = Uuid::from_bytes([
    0x76, 0x69, 0x62, 0x65, // "vibe"
    0x74, 0x65, 0x61, 0x2d, // "tea-"
    0x73, 0x65, 0x73, 0x73, // "sess"
    0x69, 0x6f, 0x6e, 0x73, // "ions"
]);

/// VibeTea Monitor - Claude Code session watcher.
///
/// Watches Claude Code session files and forwards privacy-filtered
/// events to the VibeTea server for real-time dashboard updates.
#[derive(Parser, Debug)]
#[command(name = "vibetea-monitor")]
#[command(author, version, about, long_about = None)]
#[command(after_help = "\
ENVIRONMENT VARIABLES:
    VIBETEA_SERVER_URL         Server URL (required for 'run')
    VIBETEA_SOURCE_ID          Monitor identifier (default: hostname)
    VIBETEA_KEY_PATH           Key directory (default: ~/.vibetea)
    VIBETEA_CLAUDE_DIR         Claude directory (default: ~/.claude)
    VIBETEA_BUFFER_SIZE        Event buffer size (default: 1000)
    VIBETEA_BASENAME_ALLOWLIST Comma-separated file extensions to include

EXAMPLES:
    # Generate a new keypair
    vibetea-monitor init

    # Force overwrite existing keys
    vibetea-monitor init --force

    # Export private key for GitHub Actions
    vibetea-monitor export-key

    # Start the monitor
    export VIBETEA_SERVER_URL=https://vibetea.fly.dev
    vibetea-monitor run
")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// CLI subcommands.
#[derive(Subcommand, Debug)]
enum Command {
    /// Generate Ed25519 keypair for server authentication.
    ///
    /// Creates a new keypair in ~/.vibetea (or VIBETEA_KEY_PATH).
    /// The public key must be registered with the server.
    Init {
        /// Force overwrite existing keys without confirmation.
        #[arg(short, long)]
        force: bool,
    },

    /// Export private key for GitHub Actions.
    ///
    /// Outputs the base64-encoded private key seed to stdout.
    /// Use this to set the VIBETEA_PRIVATE_KEY secret in GitHub Actions.
    ExportKey {
        /// Directory containing keypair.
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Start the monitor daemon.
    ///
    /// Watches Claude Code session files and forwards events to the server.
    /// Requires VIBETEA_SERVER_URL environment variable.
    Run,
}

fn main() -> Result<()> {
    // Parse command line arguments using clap
    let cli = Cli::parse();

    match cli.command {
        Command::Init { force } => run_init(force),
        Command::ExportKey { path } => run_export_key(path),
        Command::Run => {
            // Initialize async runtime for the run command
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .context("Failed to create tokio runtime")?;

            runtime.block_on(run_monitor())
        }
    }
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
    crypto.save(&key_dir).context("Failed to save keypair")?;

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

/// Runs the export-key command to output the private key seed.
fn run_export_key(path: Option<PathBuf>) -> Result<()> {
    // Determine key directory from argument or default
    let key_dir = match path {
        Some(p) => p,
        None => get_key_directory()?,
    };

    // Load the keypair from file (not load_with_fallback - we're exporting from file)
    match Crypto::load(&key_dir) {
        Ok(crypto) => {
            // Print ONLY the base64-encoded seed to stdout, followed by exactly one newline
            println!("{}", crypto.seed_base64());
            Ok(())
        }
        Err(_) => {
            // Print error to stderr and exit with code 1
            eprintln!("Error: No key found at {}/key.priv", key_dir.display());
            eprintln!("Run 'vibetea-monitor init' first.");
            std::process::exit(1);
        }
    }
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

    // Load cryptographic keys with environment variable precedence
    let (crypto, key_source) = Crypto::load_with_fallback(&config.key_path).context(format!(
        "Failed to load cryptographic key. Either set VIBETEA_PRIVATE_KEY environment variable \
         or run 'vibetea-monitor init' to generate keys at {}.",
        config.key_path.display()
    ))?;

    // Log which key source is being used (FR-007)
    match &key_source {
        KeySource::EnvironmentVariable => {
            // Check if file key also exists and log if it's being ignored (FR-002)
            if Crypto::exists(&config.key_path) {
                info!(
                    ignored_path = %config.key_path.display(),
                    "File key exists but VIBETEA_PRIVATE_KEY takes precedence"
                );
            }
            info!(
                source = "environment",
                fingerprint = %crypto.public_key_fingerprint(),
                "Cryptographic key loaded"
            );
        }
        KeySource::File(path) => {
            info!(
                source = "file",
                path = %path.display(),
                fingerprint = %crypto.public_key_fingerprint(),
                "Cryptographic key loaded"
            );
        }
    }

    // Create sender
    let sender_config = SenderConfig::new(
        config.server_url.clone(),
        config.source_id.clone(),
        config.buffer_size,
    );
    let mut sender = Sender::new(sender_config, crypto);

    // Create privacy pipeline
    let privacy_config = PrivacyConfig::from_env();
    let privacy_pipeline = PrivacyPipeline::new(privacy_config);

    info!("Privacy pipeline initialized");

    // Session parsers keyed by file path
    let mut session_parsers: HashMap<PathBuf, SessionParser> = HashMap::new();

    // Create channel for watch events
    let (watch_tx, mut watch_rx) = mpsc::channel::<WatchEvent>(config.buffer_size);

    // Initialize file watcher on the projects directory
    let watch_dir = config.claude_dir.join("projects");

    // Create watch directory if it doesn't exist
    if !watch_dir.exists() {
        info!(
            watch_dir = %watch_dir.display(),
            "Creating projects directory"
        );
        std::fs::create_dir_all(&watch_dir).context("Failed to create watch directory")?;
    }

    let _watcher = FileWatcher::new(watch_dir.clone(), watch_tx).context(format!(
        "Failed to initialize file watcher for {}",
        watch_dir.display()
    ))?;

    info!(
        watch_dir = %watch_dir.display(),
        "File watcher initialized"
    );

    // Create channel for stats events (session metrics + token usage) from stats tracker
    let (stats_tx, mut stats_rx) = mpsc::channel::<StatsEvent>(config.buffer_size);

    // Initialize stats tracker for session metrics and token usage monitoring
    let _stats_tracker = match StatsTracker::new(stats_tx) {
        Ok(tracker) => {
            info!(
                stats_path = %tracker.stats_path().display(),
                "Stats tracker initialized"
            );
            Some(tracker)
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to initialize stats tracker (token usage tracking disabled)"
            );
            None
        }
    };

    // Create channel for skill invocation events from skill tracker
    let (skill_tx, mut skill_rx) = mpsc::channel::<SkillInvocationEvent>(config.buffer_size);

    // Initialize skill tracker for skill/slash command monitoring
    let _skill_tracker = match SkillTracker::new(skill_tx) {
        Ok(tracker) => {
            info!(
                history_path = %tracker.history_path().display(),
                "Skill tracker initialized"
            );
            Some(tracker)
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to initialize skill tracker (skill invocation tracking disabled)"
            );
            None
        }
    };

    // Create channel for todo progress events from todo tracker
    let (todo_tx, mut todo_rx) = mpsc::channel::<TodoProgressEvent>(config.buffer_size);

    // Initialize todo tracker for task list progress monitoring
    let todo_tracker = match TodoTracker::new(todo_tx) {
        Ok(tracker) => {
            info!(
                todos_dir = %tracker.todos_dir().display(),
                "Todo tracker initialized"
            );
            Some(tracker)
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to initialize todo tracker (todo progress tracking disabled)"
            );
            None
        }
    };

    // Create channel for file change events from file history tracker
    let (file_change_tx, mut file_change_rx) = mpsc::channel::<FileChangeEvent>(config.buffer_size);

    // Initialize file history tracker for line change tracking
    let _file_history_tracker = match FileHistoryTracker::new(file_change_tx) {
        Ok(tracker) => {
            info!(
                file_history_dir = %tracker.root_dir().display(),
                "File history tracker initialized"
            );
            Some(tracker)
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to initialize file history tracker (line change tracking disabled)"
            );
            None
        }
    };

    info!("Monitor running. Press Ctrl+C to stop.");

    // Main event loop
    loop {
        tokio::select! {
            // Handle shutdown signal
            _ = wait_for_shutdown() => {
                info!("Shutdown signal received");
                break;
            }

            // Process watch events from session JSONL files
            Some(watch_event) = watch_rx.recv() => {
                process_watch_event(
                    watch_event,
                    &mut session_parsers,
                    &privacy_pipeline,
                    &mut sender,
                    &config.source_id,
                    todo_tracker.as_ref(),
                ).await;
            }

            // Process stats events (session metrics + token usage) from stats tracker
            Some(stats_event) = stats_rx.recv() => {
                process_stats_event(
                    stats_event,
                    &mut sender,
                    &config.source_id,
                ).await;
            }

            // Process skill invocation events from skill tracker
            Some(skill_event) = skill_rx.recv() => {
                process_skill_invocation_event(
                    skill_event,
                    &mut sender,
                    &config.source_id,
                ).await;
            }

            // Process todo progress events from todo tracker
            Some(todo_event) = todo_rx.recv() => {
                process_todo_progress_event(
                    todo_event,
                    &mut sender,
                    &config.source_id,
                ).await;
            }

            // Process file change events from file history tracker
            Some(file_change_event) = file_change_rx.recv() => {
                process_file_change_event(
                    file_change_event,
                    &mut sender,
                    &config.source_id,
                ).await;
            }
        }
    }

    // Graceful shutdown
    info!("Shutting down...");

    // Flush remaining events
    let unflushed = sender
        .shutdown(Duration::from_secs(SHUTDOWN_TIMEOUT_SECS))
        .await;

    if unflushed > 0 {
        error!(
            unflushed_events = unflushed,
            "Some events could not be sent"
        );
    }

    info!("Monitor stopped");
    Ok(())
}

/// Processes a single watch event, parsing JSONL lines and sending events.
async fn process_watch_event(
    watch_event: WatchEvent,
    session_parsers: &mut HashMap<PathBuf, SessionParser>,
    privacy_pipeline: &PrivacyPipeline,
    sender: &mut Sender,
    source_id: &str,
    todo_tracker: Option<&TodoTracker>,
) {
    match watch_event {
        WatchEvent::FileCreated(path) => {
            debug!(path = %path.display(), "New session file detected");
            // Parser will be created when we receive LinesAdded
        }

        WatchEvent::LinesAdded { path, lines } => {
            debug!(
                path = %path.display(),
                line_count = lines.len(),
                "Processing new lines"
            );

            // Get or create session parser for this file
            let parser = session_parsers.entry(path.clone()).or_insert_with(|| {
                match SessionParser::from_path(&path) {
                    Ok(parser) => {
                        info!(
                            path = %path.display(),
                            session_id = %parser.session_id(),
                            project = %parser.project(),
                            "Created session parser"
                        );
                        parser
                    }
                    Err(e) => {
                        warn!(
                            path = %path.display(),
                            error = %e,
                            "Failed to create session parser, using fallback"
                        );
                        // Fallback: use deterministic v5 UUID based on path.
                        // This ensures the same malformed path always maps to
                        // the same session ID, even across monitor restarts.
                        let path_str = path.to_string_lossy();
                        SessionParser::new(
                            Uuid::new_v5(&VIBETEA_NAMESPACE, path_str.as_bytes()),
                            path.parent()
                                .and_then(|p| p.file_name())
                                .and_then(|s| s.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                        )
                    }
                }
            });

            // Parse each line and convert to events
            for line in lines {
                let parsed_events = parser.parse_line(&line);

                for parsed_event in parsed_events {
                    // Check if this is a session end event (Summary)
                    // Mark session as ended in todo tracker for abandonment detection
                    if let ParsedEventKind::Summary = &parsed_event.kind {
                        if let Some(tracker) = todo_tracker {
                            tracker
                                .mark_session_ended(&parser.session_id().to_string())
                                .await;
                            debug!(
                                session_id = %parser.session_id(),
                                "Marked session as ended for todo abandonment detection"
                            );
                        }
                    }

                    if let Some(event) = convert_to_event(
                        parsed_event,
                        parser.session_id(),
                        parser.project(),
                        source_id,
                        privacy_pipeline,
                    ) {
                        // Queue event for sending
                        let evicted = sender.queue(event);
                        if evicted > 0 {
                            warn!(evicted, "Buffer overflow, events evicted");
                        }
                    }
                }
            }

            // Try to flush buffered events
            if let Err(e) = sender.flush().await {
                warn!(error = %e, "Failed to flush events, will retry later");
            }
        }

        WatchEvent::FileRemoved(path) => {
            info!(path = %path.display(), "Session file removed");
            // Clean up the session parser
            if let Some(parser) = session_parsers.remove(&path) {
                debug!(
                    session_id = %parser.session_id(),
                    "Removed session parser"
                );
            }
        }
    }
}

/// Processes a stats event from the stats tracker.
///
/// Handles [`SessionMetricsEvent`], [`TokenUsageEvent`], and [`ActivityPatternEvent`] variants.
async fn process_stats_event(stats_event: StatsEvent, sender: &mut Sender, source_id: &str) {
    let event = match stats_event {
        StatsEvent::SessionMetrics(metrics) => {
            debug!(
                total_sessions = metrics.total_sessions,
                total_messages = metrics.total_messages,
                total_tool_usage = metrics.total_tool_usage,
                longest_session = %metrics.longest_session,
                "Processing session metrics event"
            );
            Event::new(
                source_id.to_string(),
                EventType::SessionMetrics,
                EventPayload::SessionMetrics(metrics),
            )
        }
        StatsEvent::TokenUsage(token_event) => {
            debug!(
                model = %token_event.model,
                input_tokens = token_event.input_tokens,
                output_tokens = token_event.output_tokens,
                "Processing token usage event"
            );
            Event::new(
                source_id.to_string(),
                EventType::TokenUsage,
                EventPayload::TokenUsage(token_event),
            )
        }
        StatsEvent::ActivityPattern(activity_event) => {
            debug!(
                hour_count = activity_event.hour_counts.len(),
                "Processing activity pattern event"
            );
            Event::new(
                source_id.to_string(),
                EventType::ActivityPattern,
                EventPayload::ActivityPattern(activity_event),
            )
        }
        StatsEvent::ModelDistribution(dist_event) => {
            debug!(
                model_count = dist_event.model_usage.len(),
                "Processing model distribution event"
            );
            Event::new(
                source_id.to_string(),
                EventType::ModelDistribution,
                EventPayload::ModelDistribution(dist_event),
            )
        }
    };

    // Queue event for sending
    let evicted = sender.queue(event);
    if evicted > 0 {
        warn!(evicted, "Buffer overflow, events evicted");
    }

    // Try to flush buffered events
    if let Err(e) = sender.flush().await {
        warn!(error = %e, "Failed to flush stats events, will retry later");
    }
}

/// Processes a skill invocation event from the skill tracker.
async fn process_skill_invocation_event(
    skill_event: SkillInvocationEvent,
    sender: &mut Sender,
    source_id: &str,
) {
    debug!(
        skill_name = %skill_event.skill_name,
        session_id = %skill_event.session_id,
        project = %skill_event.project,
        "Processing skill invocation event"
    );

    // Convert to a full Event
    let event = Event::new(
        source_id.to_string(),
        EventType::SkillInvocation,
        EventPayload::SkillInvocation(skill_event),
    );

    // Queue event for sending
    let evicted = sender.queue(event);
    if evicted > 0 {
        warn!(evicted, "Buffer overflow, events evicted");
    }

    // Try to flush buffered events
    if let Err(e) = sender.flush().await {
        warn!(error = %e, "Failed to flush skill invocation events, will retry later");
    }
}

/// Processes a todo progress event from the todo tracker.
async fn process_todo_progress_event(
    todo_event: TodoProgressEvent,
    sender: &mut Sender,
    source_id: &str,
) {
    debug!(
        session_id = %todo_event.session_id,
        completed = todo_event.completed,
        pending = todo_event.pending,
        in_progress = todo_event.in_progress,
        abandoned = todo_event.abandoned,
        "Processing todo progress event"
    );

    // Convert to a full Event
    let event = Event::new(
        source_id.to_string(),
        EventType::TodoProgress,
        EventPayload::TodoProgress(todo_event),
    );

    // Queue event for sending
    let evicted = sender.queue(event);
    if evicted > 0 {
        warn!(evicted, "Buffer overflow, events evicted");
    }

    // Try to flush buffered events
    if let Err(e) = sender.flush().await {
        warn!(error = %e, "Failed to flush todo progress events, will retry later");
    }
}

/// Processes a file change event from the file history tracker.
async fn process_file_change_event(
    file_change_event: FileChangeEvent,
    sender: &mut Sender,
    source_id: &str,
) {
    debug!(
        session_id = %file_change_event.session_id,
        file_hash = %file_change_event.file_hash,
        version = file_change_event.version,
        lines_added = file_change_event.lines_added,
        lines_removed = file_change_event.lines_removed,
        "Processing file change event"
    );

    // Convert to a full Event
    let event = Event::new(
        source_id.to_string(),
        EventType::FileChange,
        EventPayload::FileChange(file_change_event),
    );

    // Queue event for sending
    let evicted = sender.queue(event);
    if evicted > 0 {
        warn!(evicted, "Buffer overflow, events evicted");
    }

    // Try to flush buffered events
    if let Err(e) = sender.flush().await {
        warn!(error = %e, "Failed to flush file change events, will retry later");
    }
}

/// Converts a parsed event to a VibeTea event with privacy filtering.
fn convert_to_event(
    parsed: ParsedEvent,
    session_id: Uuid,
    project: &str,
    source_id: &str,
    privacy_pipeline: &PrivacyPipeline,
) -> Option<Event> {
    let (event_type, payload) = match parsed.kind {
        ParsedEventKind::SessionStarted { project } => (
            EventType::Session,
            EventPayload::Session {
                session_id,
                action: SessionAction::Started,
                project,
            },
        ),

        ParsedEventKind::Activity => (
            EventType::Activity,
            EventPayload::Activity {
                session_id,
                project: Some(project.to_string()),
            },
        ),

        ParsedEventKind::ToolStarted { name, context } => (
            EventType::Tool,
            EventPayload::Tool {
                session_id,
                tool: name,
                status: ToolStatus::Started,
                context,
                project: Some(project.to_string()),
            },
        ),

        ParsedEventKind::ToolCompleted {
            name,
            success: _,
            context,
        } => (
            EventType::Tool,
            EventPayload::Tool {
                session_id,
                tool: name,
                status: ToolStatus::Completed,
                context,
                project: Some(project.to_string()),
            },
        ),

        ParsedEventKind::Summary => (
            EventType::Summary,
            EventPayload::Summary {
                session_id,
                summary: format!("Session ended for {}", project),
            },
        ),

        ParsedEventKind::AgentSpawned {
            agent_type,
            description,
        } => (
            EventType::AgentSpawn,
            EventPayload::AgentSpawn(AgentSpawnEvent {
                session_id: session_id.to_string(),
                agent_type,
                description,
                timestamp: parsed.timestamp,
            }),
        ),
    };

    // Apply privacy filtering
    let sanitized_payload = privacy_pipeline.process(payload);

    Some(Event {
        id: vibetea_monitor::types::Event::new(
            source_id.to_string(),
            event_type,
            sanitized_payload.clone(),
        )
        .id,
        source: source_id.to_string(),
        timestamp: parsed.timestamp,
        event_type,
        payload: sanitized_payload,
    })
}

/// Initializes the logging subsystem.
fn init_logging() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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
