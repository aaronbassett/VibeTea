# Feature Specification: Monitor GitHub Actions Deployment

**Feature Branch**: `004-monitor-gh-actions`
**Created**: 2026-02-03
**Status**: Draft
**Input**: User description: "I want a way to easily deploy the monitor to a GitHub environment during a workflow run. We'll need to update it to accept a private key via env var rather than just a folder location so we can use a predetermined one as an actions secret. So when we're using Claude Code to perform PR reviews etc we can track those events too"

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## Clarifications

### Session 2026-02-03

- Q: When monitor exhausts retry attempts to send events, should workflow succeed or fail? → A: Workflow succeeds with warning logs; unsent events are lost (non-blocking monitoring)
- Q: When monitor receives 401 Unauthorized (key not registered), should it exit or continue? → A: Exit immediately with code 1 (configuration error); fail-fast on auth issues
- Q: How should monitor signal readiness to CI workflow? → A: No explicit signal; monitor ready immediately after process start (accept potential race)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Monitor with Environment Variable Private Key (Priority: P1)

As a DevOps engineer setting up VibeTea monitoring in CI/CD, I want to configure the monitor using a private key stored as a GitHub Actions secret so that I can track Claude Code events during workflow runs without needing to manage key files in the runner environment.

**Why this priority**: This is the foundational capability that enables all other GitHub Actions deployment scenarios. Without environment variable key support, the monitor cannot be used in ephemeral CI environments where file-based keys are impractical.

**Independent Test**: Can be fully tested by setting `VIBETEA_PRIVATE_KEY` environment variable and running the monitor, verifying it authenticates successfully with the server and sends events.

**Acceptance Scenarios**:

1. **Given** a base64-encoded Ed25519 private key stored in `VIBETEA_PRIVATE_KEY` environment variable, **When** the monitor starts with `run` command, **Then** it loads the key from the environment variable and successfully signs events.

2. **Given** both `VIBETEA_PRIVATE_KEY` environment variable and a key file at `VIBETEA_KEY_PATH`, **When** the monitor starts, **Then** it prefers the environment variable over the file (env var takes precedence).

3. **Given** an invalid base64 string in `VIBETEA_PRIVATE_KEY`, **When** the monitor starts, **Then** it displays a clear error message indicating the key format is invalid.

4. **Given** a valid base64 string that decodes to incorrect key length (not 32 bytes), **When** the monitor starts, **Then** it displays a clear error message indicating the key seed must be exactly 32 bytes.

---

### User Story 2 - Export Existing Key for GitHub Actions (Priority: P1)

As a developer who already has a local VibeTea keypair, I want to export my private key in the format expected by GitHub Actions secrets so that I can use the same identity in CI workflows.

**Why this priority**: Users with existing key pairs need a migration path. Without this, they would need to generate new keys and re-register them with the server, breaking their event history continuity.

**Independent Test**: Can be fully tested by running the export command on an existing key and verifying the output can be used with `VIBETEA_PRIVATE_KEY` to authenticate successfully.

**Acceptance Scenarios**:

1. **Given** an existing keypair at the default location (`~/.vibetea`), **When** user runs `vibetea-monitor export-key`, **Then** the base64-encoded private key seed is output to stdout.

2. **Given** an existing keypair at a custom path, **When** user runs `vibetea-monitor export-key --path /custom/path`, **Then** the base64-encoded private key seed from that location is output to stdout.

3. **Given** no existing keypair, **When** user runs `vibetea-monitor export-key`, **Then** a clear error message is displayed indicating no key exists at the expected location.

---

### User Story 3 - Run Monitor in GitHub Actions Workflow (Priority: P2)

As a repository maintainer, I want to run the VibeTea monitor during GitHub Actions workflows so that I can track Claude Code events that occur during PR reviews, code generation, and other AI-assisted tasks in CI.

**Why this priority**: This is the primary use case driving this feature, but it depends on the environment variable key support (P1) being complete first.

**Independent Test**: Can be fully tested by creating a workflow that starts the monitor, generates Claude Code events, and verifying events appear on the VibeTea server.

**Acceptance Scenarios**:

1. **Given** a GitHub Actions workflow with `VIBETEA_PRIVATE_KEY` secret configured, **When** the workflow runs the monitor binary, **Then** events are captured and sent to the configured server.

2. **Given** a running monitor in GitHub Actions, **When** Claude Code performs operations (PR review, code generation), **Then** those events are tracked with the workflow's source identifier.

3. **Given** a workflow using `VIBETEA_SOURCE_ID` to identify itself, **When** events are sent to the server, **Then** they are tagged with the custom source identifier (e.g., "github-actions-pr-123").

---

### User Story 4 - Reusable GitHub Action (Priority: P3)

As a repository maintainer, I want a pre-built GitHub Action that handles monitor setup so that I can add VibeTea monitoring to my workflows with minimal configuration.

**Why this priority**: This improves developer experience but is not essential - users can run the binary directly. It builds on top of the core functionality.

**Independent Test**: Can be fully tested by using the action in a workflow with just the required secrets and verifying it captures events.

**Acceptance Scenarios**:

1. **Given** a workflow using the VibeTea monitor action, **When** the action runs with `server-url` and `private-key` inputs, **Then** the monitor starts and captures events.

2. **Given** a workflow using the action with optional `source-id` input, **When** events are sent, **Then** they use the custom source identifier.

3. **Given** a workflow using the action, **When** the workflow completes, **Then** the monitor gracefully shuts down and flushes any buffered events.

---

### Edge Cases

- What happens when the `VIBETEA_PRIVATE_KEY` contains whitespace or newlines (common when copying from terminals)?
  - Monitor should trim whitespace before base64 decoding
- What happens when the GitHub Actions runner has no network access to the VibeTea server?
  - Monitor should log warnings but not fail the workflow; unsent events are lost (non-blocking monitoring)
- What happens when monitor exhausts retry attempts to send events?
  - Workflow succeeds with warning logs; unsent events are lost; exit code remains 0
- What happens when the private key in the environment doesn't match any registered public key on the server?
  - Server returns 401 Unauthorized; monitor exits immediately with code 1 (fail-fast on auth issues)
- What happens when `export-key` is run in a CI environment where no keys exist?
  - Clear error message directing user to run `init` first on a local machine

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Monitor MUST support loading Ed25519 private key seed from `VIBETEA_PRIVATE_KEY` environment variable as base64-encoded string
- **FR-002**: Monitor MUST prefer environment variable key over file-based key when both are present, logging an INFO message that file-based key is being ignored
- **FR-003**: Monitor MUST provide `export-key` subcommand to output ONLY the base64-encoded private key followed by a single newline (no additional text), enabling direct piping to clipboard or secret management tools
- **FR-004**: Monitor MUST validate environment variable key format and provide clear error messages for invalid keys
- **FR-005**: Monitor MUST trim whitespace from `VIBETEA_PRIVATE_KEY` value before decoding
- **FR-006**: Monitor MUST support running without filesystem key storage when environment variable is provided
- **FR-007**: Monitor MUST log which key source is being used (environment variable vs file) at startup, including public key fingerprint (first 8 characters of base64 public key) for verification
- **FR-008**: Reusable GitHub Action MUST accept `server-url` and `private-key` as required inputs
- **FR-009**: Reusable GitHub Action MUST accept optional `source-id` input
- **FR-010**: Reusable GitHub Action MUST handle graceful shutdown when workflow completes

### GitHub Workflow Requirements

- **FR-011**: Existing CI workflow MUST be updated to demonstrate monitor deployment pattern
- **FR-012**: Workflow MUST download pre-built monitor binary from release artifacts or build from source
- **FR-013**: Workflow MUST configure required environment variables from repository secrets
- **FR-014**: Workflow MUST start monitor in background before Claude Code operations begin

### Documentation Requirements

- **FR-015**: README MUST include a "GitHub Actions Setup" section with step-by-step instructions
- **FR-016**: Documentation MUST explain how to export existing keys for CI use
- **FR-017**: Documentation MUST include example workflow snippet that users can copy
- **FR-018**: Documentation MUST describe required secrets and environment variables

### Security Requirements

- **FR-019**: Monitor MUST NOT log the value of `VIBETEA_PRIVATE_KEY` at any log level; only presence/absence may be logged
- **FR-020**: Private key material in intermediate buffers MUST be zeroed after the signing key is constructed
- **FR-021**: Base64 encoding MUST use standard alphabet (A-Za-z0-9+/) with `=` padding; URL-safe base64 is NOT supported
- **FR-022**: Monitor MUST validate decoded key material is exactly 32 bytes before constructing signing key

### CLI Behavior Requirements

- **FR-023**: All diagnostic and error messages from `export-key` MUST go to stderr; only the key itself goes to stdout
- **FR-024**: Monitor MUST handle both SIGINT and SIGTERM for graceful shutdown
- **FR-025**: Monitor MUST exit within configurable shutdown timeout (default 5 seconds) after receiving termination signal
- **FR-026**: Exit codes: 0 for success, 1 for configuration error (invalid env var, missing key), 2 for runtime error

### Testing Requirements

- **FR-027**: Integration tests MUST verify that a key exported with `export-key` can be loaded via `VIBETEA_PRIVATE_KEY`
- **FR-028**: Integration tests MUST verify round-trip: generate key, export, load from env var, verify signing produces valid signatures

### Non-Blocking Behavior Requirements

- **FR-029**: Monitor MUST NOT fail the GitHub Actions workflow when event transmission fails due to network issues; unsent events are logged as warnings and lost
- **FR-030**: Monitor MUST exit with code 0 even when retry attempts are exhausted for transient network failures (non-blocking monitoring)
- **FR-031**: Monitor MUST exit immediately with code 1 when receiving 401 Unauthorized response (authentication failure indicates configuration error)

### Key Entities

- **Private Key Seed**: 32-byte Ed25519 private key seed, stored either as binary file or base64-encoded environment variable
- **Source Identifier**: String identifying the event source, defaults to hostname but can be customized (e.g., "github-actions-repo-pr-123")
- **GitHub Action**: Composite or Docker action that wraps monitor binary with standard inputs

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can configure the monitor for GitHub Actions in under 5 minutes using documentation
- **SC-002**: Monitor successfully authenticates and sends events when configured via environment variable in 100% of valid configurations
- **SC-003**: Export command produces output that is immediately usable as GitHub Actions secret without modification
- **SC-004**: Error messages for invalid key configurations clearly indicate the problem and suggest resolution
- **SC-005**: GitHub Action (when implemented) requires no more than 3 configuration parameters for basic usage

## Assumptions

- Users have access to a running VibeTea server instance
- Users understand how to configure GitHub Actions secrets
- Users have existing Ed25519 keypairs they want to reuse, or can generate new ones
- GitHub Actions runners have network access to the VibeTea server
- Base64 encoding is standard (RFC 4648) without URL-safe alphabet modifications
- Monitor is ready to capture events immediately after process start; workflows accept potential race condition for events generated in first ~100ms

## Out of Scope

- Key rotation mechanisms
- Multi-key support per environment
- Server-side key management UI
- Windows-specific GitHub Actions runner support (Linux runners assumed)
- Self-hosted runner configuration
