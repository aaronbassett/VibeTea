# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04 (Phase 2 Supabase auth)

## Security Concerns

### High Priority

| ID | Area | Description | Risk Level | Mitigation | Status |
|----|------|-------------|------------|------------|--------|
| SEC-001 | WebSocket authentication | Single static token for all clients allows no per-client revocation or auditing | High | Plan token rotation mechanism or client certificates | Open |
| SEC-002 | Token management | Subscriber token hardcoded in environment variable with no expiration or rotation | High | Implement periodic token rotation and audit logging | Open |
| SEC-003 | CORS policy | All origins allowed, no CORS validation in place | Medium | Add configurable CORS origin whitelist | Open |
| SEC-013 | Session capacity (Phase 2) | In-memory session store limited to 10,000 concurrent sessions; exceeding raises HTTP 503 | Medium | Monitor capacity; consider persistent session storage for Phase 3+ | Open |

### Medium Priority

| ID | Area | Description | Risk Level | Mitigation | Status |
|----|------|-------------|------------|------------|---------|
| SEC-004 | Signature header validation | X-Signature header parsed as-is with minimal format validation | Low | Already mitigated by base64 decoding and cryptographic verification | Mitigated |
| SEC-005 | Private key environment variable | `VIBETEA_PRIVATE_KEY` env var alternative now documented and implemented | Low | Documented in SECURITY.md; same validation as file-based keys | Resolved |
| SEC-006 | Rate limiting overhead | Per-source token bucket tracking could consume memory with many unique sources | Medium | Implement stale entry cleanup (partially done); add configurable limits | Open |
| SEC-007 | Event persistence | Events are in-memory only; no persistence means events lost on restart | Medium | Document as design decision; recommend replay mechanism at application level | Open |
| SEC-008 | Export-key stdout purity | Diagnostic/error messages must be stderr-only to enable safe piping | Low | Export-key explicitly prints errors to stderr; only key goes to stdout | Mitigated (Phase 4) |
| SEC-009 | GitHub Actions secret exposure | Private key accessible in GitHub Actions environment; potential exposure via leaked logs | Medium | Use GitHub secret masking; never log VIBETEA_PRIVATE_KEY; minimize output from monitor process | Mitigated (Phase 5) |
| SEC-010 | Composite action error handling | Action warns on network failure but continues workflow; potential silent monitoring failures | Medium | Document in README; monitor logs for warnings; consider explicit failure modes | Mitigated (Phase 6) |
| SEC-011 | Key backup operation atomicity (Phase 9) | Private key backed up successfully but public key rename fails leaves orphaned backup | Medium | Best-effort restore implemented; consider explicit rollback transaction | Open |
| SEC-012 | Key option display logic (Phase 9) | Conditional rendering based on `existing_keys_found` may allow invalid state if flag not properly set | Low | State machine should enforce invariant; current approach adequate | Mitigated |
| SEC-014 | Supabase outage (Phase 2) | JWT validation via remote endpoint; if Supabase is down, authentication fails | High | Implement public key caching with 30-second refresh; fallback to cached keys | Mitigated |
| SEC-015 | Session token timing attack (Phase 2) | Session tokens compared as strings (not constant-time) during validation | Low | Token comparison is via HashMap lookup (not user-controlled); low risk | Low risk |

## Security Improvements (Phase 1-9)

### Phase 1 Features (baseline)

Foundation for signature-based authentication and WebSocket security.

### Phase 2 Features (Supabase Authentication)

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-018 | Supabase JWT validation | Remote validation via `/auth/v1/user` endpoint | Implemented | `server/src/supabase.rs:244-318` |
| FR-019 | Session token generation | 32-byte random, base64-url encoded (43 chars) | Implemented | `server/src/session.rs:564-568` |
| FR-020 | Session TTL enforcement | 5-minute default with optional grace period | Implemented | `server/src/session.rs:50-54` |
| FR-021 | Session capacity limits | 10,000 concurrent sessions with HTTP 503 on exceed | Implemented | `server/src/session.rs:57,268-277` |
| FR-022 | One-time TTL extension | WebSocket TTL extends once for 30 seconds | Implemented | `server/src/session.rs:166-176` |
| FR-023 | Public key refresh | 30-second cache refresh with exponential backoff | Implemented | `server/src/supabase.rs:410-452` |
| FR-024 | Session store cleanup | Lazy cleanup on access + batch cleanup via API | Implemented | `server/src/session.rs:514-531` |
| FR-025 | Privacy compliance | No tokens logged at any level; verified via tests | Implemented | `server/tests/auth_privacy_test.rs` |

### Phase 3 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-019 | Never log private key value | Private key seed never converted to string for logging | Implemented | `monitor/src/crypto.rs` - no logging of sensitive values |
| FR-020 | Memory zeroing for key material | Zeroize crate wipes intermediate buffers after SigningKey construction | Implemented | `monitor/src/crypto.rs:120,173,235,289` |
| FR-021 | Standard Base64 RFC 4648 | All key encoding uses standard (not URL-safe) base64 | Implemented | `monitor/src/crypto.rs:152,216` uses `BASE64_STANDARD` |
| FR-022 | Validate key material is exactly 32 bytes | Strict validation on load/decode, clear error messages | Implemented | `monitor/src/crypto.rs:161-168,219-226,276-283` |

### Phase 4 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-003 | Export-key command | CLI subcommand outputs base64-encoded private key + single newline | Implemented | `monitor/src/main.rs` |
| FR-023 | Stderr for diagnostics | All diagnostic/error messages go to stderr; stdout is key-only | Implemented | `monitor/src/main.rs` - errors print to eprintln! |
| FR-026 | Exit code semantics | 0 for success, 1 for configuration error (missing key) | Implemented | `monitor/src/main.rs` |
| FR-027 | Integration tests | Tests verify exported key roundtrips via `VIBETEA_PRIVATE_KEY` | Implemented | `monitor/tests/key_export_test.rs:148-221` |
| FR-028 | Signature consistency | Ed25519 deterministic; tests verify identical signatures after export-import | Implemented | `monitor/tests/key_export_test.rs:229-264` |

### Phase 5 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-029 | GitHub Actions workflow example | Example CI workflow showing monitor integration with export-key setup | Implemented | `.github/workflows/ci-with-monitor.yml` |
| FR-030 | Dynamic source ID in Actions | Source ID includes repo and run ID for traceability | Implemented | `.github/workflows/ci-with-monitor.yml:39` |
| FR-031 | Graceful monitor shutdown | SIGTERM handler flushes buffered events before exit | Documented | `.github/workflows/ci-with-monitor.yml:105-113` |

### Phase 6 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-032 | Composite GitHub Action | Reusable action wrapper for monitor binary download and startup | Implemented | `.github/actions/vibetea-monitor/action.yml` |
| FR-033 | Action inputs/outputs | Parameterized inputs (server-url, private-key, version) and outputs (monitor-pid, started) | Implemented | `.github/actions/vibetea-monitor/action.yml:24-55` |
| FR-034 | Action documentation | README updated with action usage, inputs, outputs, and examples | Implemented | `README.md:212-292` |
| FR-035 | Non-blocking action errors | Network/config failures log warnings but don't fail workflow | Implemented | `.github/actions/vibetea-monitor/action.yml:101-120` |
| FR-036 | Dynamic source ID interpolation | Action default source ID uses repo and run_id for uniqueness | Implemented | `.github/actions/vibetea-monitor/action.yml:96` |

### Phase 9 Features

| ID | Feature | Implementation | Status | Location |
|----|---------|-----------------|--------|----------|
| FR-015 | Key backup on generation | `backup_existing_keys()` backs up prior keys with timestamp suffix | Implemented | `monitor/src/crypto.rs:404-440` |
| FR-037 | Key option conditional display | Setup form shows key option based on `existing_keys_found` flag | Implemented | `monitor/src/tui/widgets/setup_form.rs:309-353` |
| FR-038 | Generate with backup API | `generate_with_backup()` method provides high-level backup + generate | Implemented | `monitor/src/crypto.rs:480-489` |

## Technical Debt

### High Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-001 | Cleanup task | Rate limiter cleanup task in main.rs never terminates; cleanup_handle is dropped without cancellation | Cleanup runs until server shutdown | Low | Open |
| TD-002 | Error handling | Some auth errors (InvalidPublicKey) could reveal server configuration details in logs | Debugging difficulty | Low | Open |
| TD-011 | Key backup filesystem (Phase 9) | Backup operation not atomic at filesystem level; private key rename succeeds but public key fails | Data inconsistency risk | Medium | Open |
| TD-014 | Session error details (Phase 2) | SessionError variants don't distinguish between "not found" and "expired" for debugging | Reduced observability | Low | Open |
| TD-015 | Supabase client retry logging (Phase 2) | Backoff delays logged but actual retry intervals not visible for debugging | Operational difficulty | Low | Open |

### Medium Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-003 | Configuration validation | VIBETEA_PUBLIC_KEYS parsing doesn't validate that decoded base64 is exactly 32 bytes | Confusing error messages at runtime | Low | Open |
| TD-004 | Type safety | EventPayload uses untagged enum which could be fragile with certain JSON structures | API contract ambiguity | Medium | Open |
| TD-005 | Logging | Some debug/trace logs are verbose and could impact performance under load | Performance in high-traffic scenarios | Low | Open |
| TD-008 | Export-key path handling | Currently requires --path flag; no automatic .env file detection for fallback keys | Developer friction | Low | Open |
| TD-009 | Composite action cleanup | Post-job cleanup requires manual SIGTERM step; no automatic cleanup mechanism | Potential zombie processes | Medium | Open |
| TD-012 | Key option logic (Phase 9) | Complex conditional rendering based on `existing_keys_found` flag; hard to reason about state | Maintenance burden | Low | Open |
| TD-016 | Session store persistence (Phase 2) | In-memory HashMap means all sessions lost on restart; no recovery mechanism | Service degradation on restart | High | Open |
| TD-017 | Supabase response caching (Phase 2) | Public key response cached in local HashMap; stale keys persist until refresh interval | Delayed key rotation | Medium | Open |

### Low Priority

| ID | Area | Description | Impact | Effort | Status |
|----|------|-------------|--------|--------|--------|
| TD-006 | Documentation | VIBETEA_PRIVATE_KEY environment variable now documented in SECURITY.md | Developer confusion | Low | Resolved |
| TD-007 | Error response codes | Health endpoint always returns 200 even during degradation; no status codes for partial failure | Monitoring complexity | Low | Open |
| TD-013 | Key backup duplication (Phase 9) | `load_with_fallback()` duplicates env var decoding logic | Code duplication | Low | Open |
| TD-018 | SessionStore RwLock contention (Phase 2) | All session operations serialize on single RwLock; could be bottleneck under high concurrency | Reduced throughput | Medium | Open |

## Known Bugs

| ID | Description | Workaround | Severity | Status |
|----|-------------|------------|----------|--------|
| BUG-001 | EnvGuard in tests modifies global env var state; tests must use `#[serial]` to avoid race conditions | Use `#[serial]` decorator on all env-var-touching tests | Medium | Mitigated in code |
| BUG-002 | WebSocket client lagging causes skipped events (lagged count logged but events discarded) | No workaround; clients must reconnect to resume from current position | Medium | Documented in trace log |
| BUG-003 | Export-key command integration tests expected to FAIL (implementation pending) | Implement CLI subcommand per `key_export_test.rs` spec | Medium | Pending (Phase 4) |
| BUG-004 | Session store test parallelism (Phase 2) | Tests pass but cleanup timing could race under parallel execution | Use single-threaded test runner | Low | Mitigated via test setup |

## Fragile Areas

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `server/src/auth.rs` | Critical security-sensitive code; base64/length validation is subtle | Extensive test coverage (43 tests); use RFC 8032 strict verification |
| `server/src/routes.rs` | High complexity with multiple auth paths and error cases | Test all auth combinations; validate error responses |
| `monitor/src/crypto.rs` | Cryptographic key handling; file permissions and memory management matter | Tests verify file permissions on Unix; tests verify zeroization; regenerate if compromised |
| `monitor/src/crypto.rs:404-440` | Key backup operation; filesystem atomicity is critical (Phase 9) | Backup test suite verifies permissions preserved; restore-on-failure mitigates public key issues |
| `monitor/src/tui/widgets/setup_form.rs:309-353` | Key option conditional display depends on `existing_keys_found` flag (Phase 9) | Test both branches (keys found vs not found); ensure state consistency |
| `server/src/config.rs` | Configuration parsing with environment variables; tests required `#[serial]` | Never modify without running full test suite with `--test-threads=1` |
| `monitor/tests/env_key_test.rs` | Environment variable tests must serialize to avoid race conditions | All tests use `#[serial]` decorator (24 env-var-touching tests) |
| `monitor/tests/key_export_test.rs` | Export-key tests modify env vars and spawn subprocesses; must use `#[serial]` | All tests use `#[serial]` decorator (15 export-key tests) |
| `monitor/src/main.rs` | New export-key logic handles private key material and must not log it | Verify stdout purity in tests; all key writes are stderr only |
| `.github/workflows/ci-with-monitor.yml` | Workflow manages private key and process; signal handling is critical | Test with dry-run first; ensure SIGTERM properly terminates and flushes |
| `.github/actions/vibetea-monitor/action.yml` | Composite action manages binary download and monitor process lifecycle | Ensure secret masking works; test with actual GitHub Actions runner |
| `server/src/session.rs` (Phase 2) | Session store with RwLock and TTL; concurrent access requires careful testing | Test concurrent operations; verify TTL enforcement; test cleanup logic |
| `server/src/supabase.rs` (Phase 2) | Supabase client with network I/O and retry logic; mock server tests critical | Test all error paths; verify retry backoff; test timeout handling |
| `server/tests/auth_privacy_test.rs` (Phase 2) | Privacy tests verify no token leakage; custom tracing subscriber infrastructure | Keep test coverage comprehensive; verify at TRACE level; test all code paths |

## Deprecated Code

| Area | Deprecation Reason | Removal Target | Replacement |
|------|-------------------|----------------|-------------|
| None identified | - | - | - |

## TODO Items

| Location | TODO | Priority | Status |
|----------|------|----------|--------|
| `monitor/tests/privacy_test.rs:319` | TODO regex in test assertion for security match | Medium | Open |
| `monitor/tests/key_export_test.rs:29` | Implement `export-key` CLI subcommand | High | In progress (Phase 4) |
| `server/src/supabase.rs` | TODO: Implement public key caching strategy for offline resilience (Phase 3) | High | Planned |
| `server/src/session.rs` | TODO: Add metrics/observability for session store utilization (Phase 3) | Medium | Planned |

## Dependency Concerns

### At-Risk Dependencies

| Package | Concern | Action Needed | Status |
|---------|---------|---------------|--------|
| `ed25519_dalek` | Cryptographic library; monitor for security advisories | Subscribe to GitHub security alerts | Open |
| `tokio` | Runtime; heavy async dependency with many transitive deps | Keep updated; monitor for CVEs | Open |
| `base64` | Decoding; generally stable but validate error handling | No immediate action needed | Resolved |
| `zeroize` | Critical for memory safety; wipes sensitive key material | Monitor for updates and best practices | Open |
| `chrono` | Used for backup timestamp generation (Phase 9) | Monitor for updates; generally stable | Open |
| `reqwest` (Phase 2) | HTTP client for Supabase API calls; network-facing | Monitor for security updates; use latest minor version | Open |
| `rand` (Phase 2) | Random number generation for session tokens; critical for security | Keep updated; use cryptographically secure RNG only | Open |

## Performance Concerns

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|-----------|
| PERF-001 | Rate limiter memory | Hash map of source_id -> TokenBucket grows unbounded until cleanup | Memory leak over time | Cleanup task removes stale entries (60s timeout) |
| PERF-002 | Event broadcast | Broadcaster uses bounded channel; lagging subscribers lose events | Client experience degrades | Expected behavior; clients reconnect |
| PERF-003 | JSON serialization | Every event serialized per WebSocket subscriber | CPU under high load | No mitigation; consider compression |
| PERF-004 | GitHub Actions binary download | Release binary download on every workflow run | Network overhead | Consider caching binary or building from source |
| PERF-005 | Composite action overhead | Action adds step overhead for binary download and validation | Minimal workflow slowdown | Overhead is ~5-10 seconds per workflow; acceptable for CI |
| PERF-006 | Filesystem operations (Phase 9) | Key backup involves multiple rename calls; may impact startup time | Brief UI lag on setup | Acceptable: one-time operation; run on dedicated thread if needed |
| PERF-007 | Session store RwLock (Phase 2) | All session operations serialize on single RwLock; could be bottleneck | Reduced throughput under load | Consider sharding or concurrent data structure if needed |
| PERF-008 | Supabase retry delays (Phase 2) | Exponential backoff could delay server startup by up to 10s | Startup latency | Acceptable for reliability; monitor in production |

## Monitoring Gaps

| Area | Missing | Impact | Notes |
|------|---------|--------|-------|
| Private key source tracking | No metrics on whether file vs env var is used | Can't audit key source at runtime | KeySource enum added in Phase 3; logging at startup recommended |
| Rate limiter state | No metrics on bucket count or token refill rates | Hard to debug rate limiting issues | Consider adding Prometheus metrics |
| Authentication success rate | No metrics on auth successes vs failures | Can't detect brute force attempts | Would require counter instrumentation |
| WebSocket health | No metrics on connection duration or message throughput | Hard to diagnose subscription issues | Consider adding connection metrics |
| Export-key usage | No audit trail of key exports | Can't track which systems have exported keys | Consider adding telemetry or structured logging |
| GitHub Actions monitor | No metrics on monitor process uptime/failures in CI | Can't detect if monitoring silently fails | Consider structured logging to Actions output |
| Composite action usage | No telemetry on adoption or failure rates | Can't track action usage patterns | Could add optional telemetry to action |
| Key backup operations (Phase 9) | No metrics on successful/failed backups | Can't detect if key rotation is working | Consider adding structured logging |
| Session store utilization (Phase 2) | No metrics on active sessions, TTL distribution, cleanup frequency | Can't detect capacity issues in advance | Consider adding Prometheus metrics |
| Supabase client health (Phase 2) | No metrics on JWT validation latency, retry frequency, cache hit rate | Can't diagnose auth performance issues | Consider adding histograms for request latency |

## Improvement Opportunities

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Token management | Static string in environment | JWT or token service with expiration | Better security and client management |
| Configuration validation | Happens at startup only | Happens at startup with detailed validation | Catch misconfigurations earlier |
| Error logging | Mix of debug/warn levels | Structured error types with contextual data | Better observability |
| Rate limiting | Per-source only | Support per-IP and per-endpoint limits | Finer-grained DoS protection |
| Event filtering | Client-side by query params | Server-side filtering with ACLs | Reduced bandwidth, better security |
| Private key rotation | Manual process | Automated rotation with versioning | Reduced risk of key compromise |
| Export-key defaults | Explicit --path flag required | Auto-discovery of ~/.vibetea or env var | Smoother UX for end users |
| GitHub Actions integration | Manual secret setup | Documentation or automated secret creation script | Easier onboarding for CI/CD |
| Composite action | Basic functionality | Advanced features (log output, retry logic) | Better debugging and resilience |
| Key backup atomicity (Phase 9) | Best-effort restore | Transactional backup with rollback | Guarantee consistency |
| Session persistence (Phase 2) | In-memory only | Optional Redis/database backend | Survive server restarts |
| Public key refresh (Phase 2) | 30-second refresh interval | Configurable interval with monitoring | Better balance of freshness vs load |

## Security Debt Items

| ID | Area | Description | Mitigation Strategy | Status |
|----|------|-------------|------------|--------|
| DEBT-001 | WebSocket token | Same token for all clients across all time | Implement token rotation every N days or on deployment | Open |
| DEBT-002 | Key registration | Public keys hardcoded in environment variable | Implement key management API with dynamic updates | Open |
| DEBT-003 | Audit trail | Minimal logging of auth events | Add structured audit logging to database/file | Open |
| DEBT-004 | Client identity | WebSocket clients are anonymous beyond token | Add optional client ID/name for audit purposes | Open |
| DEBT-005 | Key export audit | No record of when/where keys are exported | Implement export logging with timestamp/system info | Open |
| DEBT-006 | GitHub Actions secret usage | Monitor process has access to private key; potential logging risk | Implement log filtering to never output env vars | Phase 5 risk |
| DEBT-007 | Composite action versioning | Action pinned to @main; no semantic versioning | Implement version tags and GitHub releases | Phase 6 opportunity |
| DEBT-008 | Key backup retention (Phase 9) | No automatic cleanup of old backup files | Implement retention policy (e.g., keep last N backups) | Open |
| DEBT-009 | Session token rotation (Phase 2) | Session tokens generated once, never rotated | Implement periodic token refresh on request | Phase 3 improvement |
| DEBT-010 | Public key distribution (Phase 2) | Keys distributed via Supabase edge function; no signature verification | Implement signed public key responses | Phase 3 improvement |

## Potential Attack Vectors

| Vector | Mitigation | Status |
|--------|-----------|--------|
| Signature bypass (wrong message signed) | Signature verifies full request body | Mitigated |
| Timing attack on token comparison | Constant-time comparison with `subtle` crate | Mitigated |
| Timing attack on key material buffers | Zeroize crate wipes intermediate buffers | Mitigated (Phase 3) |
| Rate limit bypass (multiple sources) | Per-source limiting doesn't prevent N sources | Partially mitigated by total capacity |
| Private key logging | Private key never converted to string for logging | Mitigated (Phase 3) |
| Invalid key length in env var | Strict validation: exactly 32 bytes required | Mitigated (Phase 3) |
| WebSocket replay (reuse old token) | Static token never expires | Not mitigated |
| Source ID spoofing | Must provide valid signature for registered source | Mitigated |
| Base64 decoding errors | Handled with explicit error cases | Mitigated |
| Whitespace in env var key | Trimmed before base64 decoding | Mitigated (Phase 3) |
| Export-key output leakage | Diagnostic messages sent to stderr only | Mitigated (Phase 4) |
| Key export in cleartext | Keys exported as base64; assumes secure transport (HTTPS/CI secrets) | Requires operator discipline |
| GitHub Actions log leakage | Private key is env var, subject to accidental logging | Partially mitigated by GitHub secret masking (Phase 5) |
| Composite action binary tampering | Binary downloaded from GitHub releases without signature verification | Partially mitigated by HTTPS; recommend checksum verification |
| Man-in-the-middle on binary download | Binary download from GitHub releases via HTTP curl | Mitigated by HTTPS (curl -fsSL) |
| Key backup file leakage | Backup files have same permissions as originals (0600) | Mitigated by file permissions |
| Backup restore collision | Backup timestamp could theoretically collide if gen twice per second | Mitigated by timestamp format; highly unlikely |
| Session token guessing (Phase 2) | Tokens are 32 bytes of cryptographically random data | Very low probability; 256-bit entropy |
| JWT token forgery (Phase 2) | JWTs validated remotely via Supabase; server never validates signature locally | Relies on Supabase security |
| Supabase service compromise (Phase 2) | If Supabase is compromised, JWTs could be forged | Mitigation: public key caching as fallback (Phase 3 improvement) |
| Session table enumeration (Phase 2) | Session tokens are opaque; no sequential IDs or patterns | Tokens are cryptographically random; enumeration infeasible |
| Session fixation (Phase 2) | Client requests session, server generates random token | Client cannot choose token; session fixation not possible |

---

## Concern Severity Guide

| Level | Definition | Response Time |
|-------|------------|----------------|
| Critical | Production impact, security breach | Immediate |
| High | Degraded functionality, security risk | This sprint |
| Medium | Developer experience, minor issues | Next sprint |
| Low | Nice to have, cosmetic | Backlog |

---

*This document tracks what needs attention. Update when concerns are resolved or discovered. Last updated with Phase 2 Supabase authentication analysis.*
