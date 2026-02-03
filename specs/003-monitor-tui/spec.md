# Feature Specification: Monitor TUI Interface

**Feature Branch**: `003-monitor-tui`
**Created**: 2026-02-03
**Status**: Draft
**Input**: User description: "update the monitor to have a cool TUI interface when first started it should let the user set the session name (or use default) and generate a new key or use existing. Once the user completes the form (make sure it has great defaults so the user can just hit enter and skip past it) it should show a header with logo and server status, the main body should be a streaming log of whats happening, below the stream show the session name and public key the user should add to the server to allow the monitor to auth, in the footer it should show counts of events, failures, etc"

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Quick Start with Defaults (Priority: P1)

As a user running the monitor for the first time, I want to start monitoring immediately by pressing Enter through the setup form so that I can see activity with minimal friction.

**Why this priority**: The core value proposition is making the monitor easy to start. Most users will accept defaults, so this flow must be seamless.

**Independent Test**: Can be fully tested by launching the monitor, pressing Enter through all prompts, and verifying the main dashboard displays with streaming logs.

**Acceptance Scenarios**:

1. **Given** the monitor is launched without existing keys, **When** the user presses Enter on each prompt, **Then** a new session name is generated (using hostname), a new keypair is created, and the main dashboard displays within 3 seconds.
2. **Given** the monitor is launched with existing keys in `~/.vibetea`, **When** the user presses Enter on the "use existing key" prompt, **Then** the existing keypair is loaded and the main dashboard displays.
3. **Given** the setup form is displayed, **When** the user views any field, **Then** a sensible default value is pre-filled and highlighted.

---

### User Story 2 - View Real-Time Event Stream (Priority: P1)

As a user monitoring Claude Code sessions, I want to see a live stream of events scrolling in the main body so that I can understand what's happening in real-time.

**Why this priority**: The streaming log is the primary interface users interact with after setup. Without this, the TUI provides no value.

**Independent Test**: Can be fully tested by starting the monitor, triggering Claude Code activity, and verifying events appear in the stream within 2 seconds.

**Acceptance Scenarios**:

1. **Given** the main dashboard is displayed, **When** a new event is received, **Then** the event appears at the bottom of the stream with timestamp, event type, and relevant details.
2. **Given** the stream contains 100+ events, **When** new events arrive, **Then** old events scroll up and the newest event is always visible.
3. **Given** the stream is active, **When** the user resizes the terminal, **Then** the stream adjusts to fit the new dimensions without losing events.

---

### User Story 3 - Monitor Server Connection Status (Priority: P1)

As a user, I want to see the server connection status in the header so that I know if my events are being sent successfully.

**Why this priority**: Without connection status visibility, users can't troubleshoot connectivity issues. This is critical for trust in the tool.

**Independent Test**: Can be fully tested by starting the monitor, observing the header shows "Connected" status, then disconnecting the network and observing the status changes.

**Acceptance Scenarios**:

1. **Given** the monitor successfully connects to the server, **When** the main dashboard displays, **Then** the header shows "Connected" status with a visual indicator (color/icon).
2. **Given** the monitor cannot reach the server, **When** the main dashboard displays, **Then** the header shows "Disconnected" status with a visual indicator.
3. **Given** the connection state changes, **When** the dashboard is visible, **Then** the status updates within 5 seconds.

---

### User Story 4 - View Authentication Credentials (Priority: P2)

As a user setting up a new monitor, I want to see my session name and public key displayed below the stream so that I can copy them to configure the server.

**Why this priority**: Users need credentials to complete server setup, but this is secondary to the core monitoring functionality.

**Independent Test**: Can be fully tested by starting the monitor and verifying the credentials panel displays the session name and base64-encoded public key.

**Acceptance Scenarios**:

1. **Given** the main dashboard is displayed, **When** the user looks below the stream, **Then** the session name and full public key are visible.
2. **Given** the credentials are displayed, **When** the user needs to copy them, **Then** the public key is shown in a format suitable for copy-paste (single line, base64).
3. **Given** the credentials panel is visible, **When** the terminal width is narrow, **Then** the public key wraps or truncates gracefully with an indication it's truncated.

---

### User Story 5 - Track Event Statistics (Priority: P2)

As a user, I want to see counts of total events, successful sends, and failures in the footer so that I can monitor the health of the monitoring system.

**Why this priority**: Statistics provide confidence that the system is working correctly, but users can function without them initially.

**Independent Test**: Can be fully tested by starting the monitor, triggering events, and verifying the footer counters increment appropriately.

**Acceptance Scenarios**:

1. **Given** the main dashboard is displayed, **When** events are processed, **Then** the footer shows counts for: Total Events, Sent, and Failed.
2. **Given** an event fails to send, **When** the footer updates, **Then** the Failed count increments and is visually distinguished (color/style).
3. **Given** the monitor has been running, **When** the user checks the footer, **Then** the counts reflect the actual number of events processed since startup.

---

### User Story 6 - Custom Session Name Setup (Priority: P3)

As a power user, I want to specify a custom session name during setup so that I can identify this monitor instance when viewing events on the server.

**Why this priority**: Customization is valuable for multi-machine setups but not required for basic functionality.

**Independent Test**: Can be fully tested by launching the monitor, entering a custom session name, completing setup, and verifying the credentials panel shows the custom name.

**Acceptance Scenarios**:

1. **Given** the setup form is displayed, **When** the user types a custom session name and presses Enter, **Then** the custom name is used for the session.
2. **Given** the user enters an invalid session name (empty after trim, too long, invalid characters), **When** they try to proceed, **Then** an inline error is shown and they can correct it.

---

### User Story 7 - Key Management Options (Priority: P3)

As a user with existing keys, I want to choose between using my existing keys or generating new ones during setup so that I can manage my authentication credentials.

**Why this priority**: Key management is important for security but most users will just use defaults.

**Independent Test**: Can be fully tested by running the monitor with existing keys, selecting "Generate new key", and verifying a new keypair is created.

**Acceptance Scenarios**:

1. **Given** existing keys are found in `~/.vibetea`, **When** the setup form displays, **Then** the user is presented with options: "Use existing key" (default) or "Generate new key".
2. **Given** the user selects "Generate new key", **When** they confirm, **Then** the old keys are backed up (renamed with timestamp) and new keys are generated.
3. **Given** no existing keys are found, **When** the setup form displays, **Then** the key option shows "Generate new key" as the only option (pre-selected).

---

### User Story 8 - Display VibeTea Logo (Priority: P3)

As a user, I want to see a stylized VibeTea logo in the header so that the tool feels polished and professional.

**Why this priority**: Branding enhances user experience but is not functional.

**Independent Test**: Can be fully tested by starting the monitor and verifying an ASCII art or styled logo appears in the header area.

**Acceptance Scenarios**:

1. **Given** the main dashboard is displayed, **When** the header renders, **Then** a VibeTea logo/banner is visible.
2. **Given** the terminal is narrower than the logo, **When** the header renders, **Then** the logo degrades gracefully (shorter version or text-only).

---

### Edge Cases

- What happens when the server URL is not configured? Display an error message in the setup form and prevent proceeding to the main dashboard until configured.
- What happens when key file permissions are incorrect? Show a warning but attempt to proceed; fail gracefully with a clear error message if keys cannot be read.
- What happens when the terminal is extremely small (e.g., 40x10)? Display a minimum size warning and refuse to start TUI mode until terminal is resized.
- What happens when events arrive faster than the display can update? Batch event display updates to prevent UI freezing; frames may be skipped if rendering takes too long.
- What happens when the user presses Ctrl+C during setup? Exit cleanly without creating partial state, restoring terminal to usable state.
- What happens when network connectivity is intermittent? Show "Reconnecting..." status and queue events locally.
- What happens when the application panics? Terminal state must be restored before the panic message is printed.
- What happens when the user scrolls up in the event stream? Auto-scroll to new events pauses until user scrolls back to bottom.
- What happens when running in tmux/screen? The TUI should function correctly in terminal multiplexers.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display a setup form when launched that allows configuring session name and key generation options.
- **FR-002**: Setup form MUST have sensible defaults for all fields so users can complete setup by pressing Enter repeatedly.
- **FR-003**: Session name field MUST default to the system hostname.
- **FR-004**: Key generation option MUST default to "Use existing" if keys exist, or "Generate new" if no keys exist.
- **FR-005**: System MUST transition from setup form to main dashboard within 3 seconds of completing setup.
- **FR-006**: Main dashboard MUST display a header section containing logo/banner and server connection status.
- **FR-007**: Main dashboard MUST display a scrolling event stream in the main body area.
- **FR-008**: Event stream MUST show each event's timestamp, type, and relevant details.
- **FR-009**: Main dashboard MUST display a credentials panel below the stream showing session name and public key.
- **FR-010**: Public key MUST be displayed in base64 format suitable for copy-paste.
- **FR-011**: Main dashboard MUST display a footer showing event statistics (total, sent, failed).
- **FR-012**: Failed event count MUST be visually distinguished from successful counts.
- **FR-013**: System MUST handle terminal resize events and re-render the layout appropriately.
- **FR-014**: System MUST validate session name input (non-empty, reasonable length, valid characters).
- **FR-015**: System MUST backup existing keys before generating new ones (timestamp-suffixed rename).
- **FR-016**: System MUST display server connection status (connected/disconnected/reconnecting).
- **FR-017**: System MUST allow user to exit via standard terminal shortcuts (Ctrl+C, q).
- **FR-018**: System MUST preserve existing CLI behavior (`init`, `run` subcommands) while adding TUI as the default mode.
- **FR-019**: System MUST restore terminal state (raw mode off, alternate screen exited) on all exit paths including panics, signals, and errors.
- **FR-020**: System MUST use the alternate screen buffer to preserve the user's shell history.
- **FR-021**: Event stream MUST auto-scroll to show new events unless the user has manually scrolled up; scrolling to the bottom resumes auto-scroll.
- **FR-022**: Event lines exceeding terminal width MUST be truncated with ellipsis, not wrapped.
- **FR-023**: Each event in the stream MUST display: timestamp (HH:MM:SS format), event type indicator, and relevant payload summary.
- **FR-024**: System MUST maintain a separate display buffer from the transmission buffer, limited to a configurable maximum (default: 1000 events).
- **FR-025**: The event sender MUST expose observable metrics (events queued, sent, failed) for UI consumption.
- **FR-026**: Session name input MUST be limited to 64 characters and allow only alphanumeric characters, hyphens, and underscores.

### Non-Functional Requirements

- **NFR-001**: The TUI MUST refresh at a consistent tick rate (target: 60ms intervals) to balance responsiveness with resource usage.
- **NFR-002**: Input event handling MUST NOT block the async runtime; use non-blocking polling integrated with the event loop.
- **NFR-003**: The TUI MUST integrate with the existing async event loop without blocking.
- **NFR-004**: The application MUST function correctly on terminals supporting VT100 escape sequences; Unicode box-drawing characters should degrade gracefully.
- **NFR-005**: When running in TUI mode, log output MUST be suppressed from stderr or redirected to a file to avoid corrupting the display.
- **NFR-006**: The application MUST use color-blind-safe color choices for status indicators (avoid relying solely on red/green).
- **NFR-007**: CPU usage SHOULD remain below 5% when idle (no new events arriving).

### Development Standards

- **DS-001**: Project MUST include Justfile with common development commands (build, test, run, lint, format).
- **DS-002**: Project MUST include lefthook configuration for pre-commit hooks (linting and formatting checks).
- **DS-003**: GitHub CI workflow MUST be updated to include TUI-related tests.
- **DS-004**: All new code MUST pass existing linting rules (clippy for Rust, ESLint for TypeScript).
- **DS-005**: All new code MUST be formatted according to project standards (rustfmt, Prettier).

### Key Entities

- **Setup Form State**: Session name input, key generation choice, validation errors, current field focus.
- **Dashboard State**: Connection status, event stream buffer, event statistics (total/sent/failed), credentials.
- **Event Stream Entry**: Timestamp, event type (Session/Activity/Tool/Summary), display-formatted details.
- **Statistics**: Total events processed, events successfully sent, events failed to send.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can complete setup and reach the main dashboard in under 10 seconds by pressing Enter through all prompts.
- **SC-002**: New events appear in the stream within 2 seconds of occurring.
- **SC-003**: Connection status updates reflect actual server connectivity within 5 seconds of state change.
- **SC-004**: Event statistics accurately reflect the number of events processed (no drift or missed counts).
- **SC-005**: The TUI remains responsive (accepts input, updates display) under sustained load of 100+ events per second.
- **SC-006**: Terminal resize events are handled smoothly without visual glitches or crashes.
- **SC-007**: The TUI renders correctly on terminals with minimum dimensions of 80x24 characters.
- **SC-008**: Users can successfully copy the displayed public key and use it for server configuration without modification.
- **SC-009**: Terminal is restored to a usable state after exit, including after panics or signals.
- **SC-010**: No visual artifacts appear when resizing the terminal rapidly.
- **SC-011**: Application exits cleanly on SIGTERM signal.
- **SC-012**: TUI functions correctly when running inside tmux or screen multiplexers.

## Assumptions

- Users have a terminal that supports ANSI escape codes (colors, cursor positioning).
- The existing `vibetea-monitor` crate structure will be extended rather than replaced.
- The TUI will become the default when running `vibetea-monitor` without subcommands, preserving `init` and `run` for scripting/automation use cases.
- Event buffering in the stream display will be limited (e.g., last 1000 events) to prevent memory growth.
