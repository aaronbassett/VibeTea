# Feature Specification: Supabase Persistence Layer

**Feature Branch**: `001-supabase-persistence`
**Created**: 2026-02-03
**Status**: Draft
**Input**: User description: "Add optional persistence with Supabase for historic data, activity heatmaps, and batch event submission via edge functions"

**Codebase Documentation**: See [.sdd/codebase/](.sdd/codebase/) for technical details

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Historic Activity Heatmap (Priority: P1)

As a user viewing the dashboard, I want to see an activity heatmap that shows my historic coding activity over the last 7 or 30 days, so I can understand my productivity patterns even when I wasn't connected in real-time.

**Why this priority**: This is the primary user-facing value of persistence - visualizing historic data. Without this, persistence has no visible benefit.

**Independent Test**: Can be tested by enabling persistence, generating events over multiple days, then viewing the heatmap to see historic data displayed correctly.

**Acceptance Scenarios**:

1. **Given** persistence is enabled and historic events exist in the database, **When** I load the client dashboard, **Then** the activity heatmap displays event counts from persisted historic data (not just the current session).

2. **Given** persistence is enabled and I select the 30-day view, **When** the heatmap loads, **Then** I see activity data spanning the last 30 calendar days from the database (or less if insufficient data exists).

3. **Given** persistence is NOT enabled (no `VITE_SUPABASE_URL` env var), **When** I load the client dashboard, **Then** the heatmap card is hidden entirely (not shown with "no data").

---

### User Story 2 - Monitor Batches and Persists Events (Priority: P2)

As a system operator, I want the monitor to automatically batch and persist events to Supabase, so that historic data is available for future analysis without impacting real-time performance.

**Why this priority**: Without event persistence, there's no historic data to display. This is the data ingestion foundation.

**Independent Test**: Can be tested by running the monitor with persistence enabled, generating events, and verifying batched events appear in Supabase.

**Acceptance Scenarios**:

1. **Given** the monitor has `VIBETEA_SUPABASE_URL` configured, **When** events are generated, **Then** the monitor batches events and sends them to the Supabase edge function periodically.

2. **Given** the monitor has accumulated events in its batch buffer, **When** the batch interval elapses (default 60 seconds) OR 1000 events are queued (whichever comes first), **Then** all buffered events are sent in a single request to the edge function.

3. **Given** the monitor does NOT have `VIBETEA_SUPABASE_URL` configured, **When** events are generated, **Then** events are still sent to the real-time server but NOT batched for persistence.

4. **Given** the monitor sends a batch to the edge function, **When** the request is made, **Then** the request is signed using the same Ed25519 private key used for real-time events (with `X-Source-ID` and `X-Signature` headers).

---

### User Story 3 - Client Queries Historic Data (Priority: P3)

As the client application, I need to fetch historic event data from Supabase to populate the activity heatmap with data from before the current session.

**Why this priority**: Enables the heatmap to show historic data by providing the query mechanism.

**Independent Test**: Can be tested by seeding historic data in Supabase, then loading the client and verifying the heatmap displays the seeded data.

**Acceptance Scenarios**:

1. **Given** persistence is enabled with `VITE_SUPABASE_URL`, **When** the client initializes, **Then** it fetches aggregated event data for the heatmap (last 7 or 30 days depending on view).

2. **Given** the client requests historic data from the edge function, **When** making the request, **Then** the client includes the `Authorization: Bearer <token>` header using the same token used for WebSocket auth.

3. **Given** historic data and real-time events both exist, **When** the heatmap renders, **Then** the data is merged at the hour level (real-time event counts for the current hour override historic hourly aggregates for that same hour; no event-level deduplication needed since aggregates are hour-granular counts).

---

### User Story 4 - Secure Edge Functions Handle All Database Access (Priority: P4)

As a system architect, I want all Supabase database access to go through edge functions using VibeTea's existing authentication, so the database remains completely private and not directly accessible.

**Why this priority**: Security foundation - ensures the database is locked down and only accessible through authenticated edge functions.

**Independent Test**: Can be tested by attempting direct database access (should fail) and verified by successful authenticated requests through edge functions.

**Acceptance Scenarios**:

1. **Given** a properly configured Supabase project, **When** anyone attempts to access the database directly (via Supabase client SDK or REST API), **Then** the request is denied (RLS policies deny all direct access).

2. **Given** a monitor sends a batch of events to the ingest edge function, **When** the edge function receives the request, **Then** it validates the Ed25519 signature before inserting events into the database.

3. **Given** a client requests historic data from the query edge function, **When** the edge function receives the request, **Then** it validates the bearer token before returning data.

4. **Given** invalid authentication credentials, **When** a request is made to any edge function, **Then** the request is rejected with a 401 status.

---

### Edge Cases

- What happens when the batch send fails due to network issues?
  - The monitor should retain failed events and retry on the next batch interval (up to a configurable retry limit).

- What happens when the Supabase edge function is temporarily unavailable?
  - The monitor continues operating normally for real-time events; persistence is best-effort (does not block real-time event flow; aims for 95% delivery under normal network conditions per SC-002).

- What happens when the database contains events from a time period the client already has in memory from real-time streaming?
  - Deduplication occurs at the hour level: real-time event counts for the current hour override historic hourly aggregates for that same hour. Event-level deduplication is not needed since query returns aggregated hourly counts (HourlyAggregate), not individual events.

- What happens when persistence is enabled on the monitor but not on the client (or vice versa)?
  - Each component operates independently based on its own configuration. The monitor persists regardless of client configuration, and the client only attempts to fetch historic data if its own persistence is enabled.

- What happens when the client is loading historic data?
  - The heatmap displays "Fetching historic data..." loading state while fetching (5-second timeout). If the fetch fails or times out, the heatmap shows real-time data only with an error message: "Unable to load historic data. Showing real-time events only." and a Retry button (non-blocking).

- What happens if the batch submission to Supabase repeatedly fails?
  - After 3 consecutive failures with exponential backoff, the monitor drops the batch and logs a warning. This prevents unbounded memory growth while preserving real-time functionality.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support optional persistence that can be enabled/disabled via environment variables (`VIBETEA_SUPABASE_URL` for monitor, `VITE_SUPABASE_URL` for client).

- **FR-002**: Monitor MUST batch events and send them to a Supabase edge function when either: (a) the configurable interval elapses (default: 60 seconds via `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS`), or (b) maximum batch size of 1000 events is reached, whichever comes first.

- **FR-003**: Monitor MUST sign ingest edge function requests using the same Ed25519 key pair used for real-time event submission, with `X-Source-ID` and `X-Signature` headers.

- **FR-004**: Client MUST authenticate to the query edge function using the same bearer token used for WebSocket authentication, via `Authorization: Bearer <token>` header.

- **FR-005a**: Ingest edge function MUST validate Ed25519 signature (X-Signature header) against the public key for the source (X-Source-ID header) before processing.

- **FR-005b**: Query edge function MUST validate bearer token (Authorization header) using constant-time comparison before returning data.

- **FR-006**: Database MUST be configured with Row Level Security (RLS) policies that deny all direct access (all access through service role in edge functions only).

- **FR-007**: Client MUST hide the activity heatmap card entirely when persistence is not enabled.

- **FR-008**: Client MUST fetch and display historic activity data in the heatmap when persistence is enabled.

- **FR-009**: Monitor MUST continue sending real-time events regardless of persistence status or persistence failures.

- **FR-010**: *(Consolidated into FR-002)* Batch size limit of 1000 events defined in FR-002.

- **FR-011**: Query edge function MUST return hourly aggregated counts (HourlyAggregate[]) for heatmap visualization, never raw event payloads. This ensures privacy and reduces response size.

- **FR-015**: Monitor MUST implement retry logic for failed batch submissions: exponential backoff (1s, 2s, 4s delays), maximum 3 consecutive retry attempts (configurable via `VIBETEA_SUPABASE_RETRY_LIMIT`, default: 3). After max retries, the batch is dropped and a warning is logged.

### Development & Operations Requirements

- **FR-012**: Database schema changes MUST be managed through versioned migration scripts that can be applied to new or existing Supabase projects.

- **FR-013**: CI workflow MUST include tests for persistence functionality (mocked edge function responses for unit tests).

- **FR-014**: Edge functions MUST have documented API contracts (request/response schemas) for both ingest and query endpoints.

### Key Entities

- **Event** (persisted): Same structure as real-time events - id, source, timestamp, event_type, payload. Primary key is `id`. Indexed on `timestamp` for efficient time-range queries.

- **Hourly Aggregate**: Pre-aggregated data returned by query edge function - source, date (YYYY-MM-DD), hour (0-23), event_count. This is the shape returned to clients for heatmap rendering (not raw events).

### Configuration

| Environment Variable | Component | Required | Default | Description |
|---------------------|-----------|----------|---------|-------------|
| `VIBETEA_SUPABASE_URL` | Monitor | No | - | Supabase edge function base URL. If absent, persistence disabled. |
| `VIBETEA_SUPABASE_BATCH_INTERVAL_SECS` | Monitor | No | 60 | Seconds between batch submissions (max 1000 events triggers immediate send). |
| `VIBETEA_SUPABASE_RETRY_LIMIT` | Monitor | No | 3 | Maximum consecutive retry attempts before dropping batch (min: 1, max: 10). |
| `VITE_SUPABASE_URL` | Client | No | - | Supabase edge function base URL. If absent, heatmap hidden. |

### Field Name Transformation

Events are serialized differently between JSON (TypeScript/Rust) and SQL:
- **JSON field**: `eventType` (camelCase) - used in monitor batch submission and edge function request body
- **SQL column**: `event_type` (snake_case) - stored in PostgreSQL events table

The ingest edge function transforms: `(e->>'eventType')::TEXT AS event_type` during bulk insert.

### Data Flow

1. **Ingest (Monitor → Supabase)**: Monitor batches events locally, sends JSON array to `/ingest` edge function with Ed25519 signature in headers. Edge function validates signature, inserts events.

2. **Query (Client → Supabase)**: Client requests hourly aggregates from `/query` edge function with bearer token. Edge function validates token, returns pre-aggregated counts (not raw events).

3. **Merge (Client)**: Client stores historic aggregates separately from real-time event buffer. Heatmap component merges both data sources, with real-time events taking precedence for the current hour.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can view activity patterns from up to 30 days ago on the heatmap (historic data visible within 5 seconds of page load, assuming normal network conditions). Heatmap displays loading state during fetch; shows error message if fetch exceeds 5 seconds.

- **SC-002**: Monitor successfully persists 95% or more of generated events under normal network conditions (defined as: latency <200ms, packet loss <1%, edge function availability >99%).

- **SC-003**: All database operations occur exclusively through edge functions (zero direct database connections from clients or monitors).

- **SC-004**: Enabling/disabling persistence requires only environment variable changes (no code changes or redeployment).

- **SC-005**: Real-time functionality remains unaffected when persistence fails (zero degradation of WebSocket event streaming).

- **SC-006**: Heatmap card is not visible when persistence is disabled (clean UI without empty/broken states).

## Assumptions

- Supabase edge functions (Deno) support Ed25519 signature verification via a compatible library (e.g., `@noble/ed25519`).
- The existing bearer token used for WebSocket authentication is suitable for edge function authentication.
- Hourly aggregation granularity is sufficient for heatmap visualization (no need for minute-level data).
- Event batching interval of 60 seconds provides acceptable balance between data freshness and request efficiency.
- Maximum batch size of 1000 events is sufficient for typical usage patterns.
- Edge functions use `SUPABASE_SERVICE_ROLE_KEY` (server-side only) to access the database, bypassing RLS for authenticated requests.
- **Normal network conditions** are defined as: network latency <200ms, packet loss <1%, Supabase edge function availability >99%. Edge function timeout is 30 seconds (Supabase platform limit).

## Out of Scope

- Real-time sync between multiple clients viewing the same heatmap.
- Export/download of historic event data.
- Custom retention policies beyond what Supabase provides.
- Multi-tenant support (all events from all monitors go to the same table).
- Detailed analytics beyond the activity heatmap (e.g., tool usage breakdowns, session duration analytics).
