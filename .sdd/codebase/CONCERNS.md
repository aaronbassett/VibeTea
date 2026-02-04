# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-04

## Security Concerns

Security-related issues requiring attention:

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-001 | Bearer Token Storage | `VIBETEA_SUBSCRIBER_TOKEN` is a plain string with no expiration or rotation mechanism. Clients receive this via environment variables and store in localStorage/browser memory. | Medium | Implement JWT tokens with expiration and refresh mechanism. Add token rotation policy. |
| SEC-002 | Edge Function Bearer Token Comparison | `validateBearerToken()` in `supabase/functions/_shared/auth.ts` uses simple `===` comparison instead of constant-time comparison. Timing attacks possible but impractical due to network latency. | Low | Replace with constant-time comparison utility (though network timing makes this low-priority). |
| SEC-003 | Client-Side Token in localStorage | Client bearer token (`vibetea_token`) stored in browser localStorage without encryption. Vulnerable to XSS attacks if malicious script executes. | Medium | Consider session-based storage or encrypted localStorage. Implement Content-Security-Policy to reduce XSS risk. |
| SEC-004 | Environment Variables in Client Bundle | `VITE_SUPABASE_URL` and `VITE_SUPABASE_TOKEN` are build-time environment variables visible in client code. Token cannot be rotated without redeployment. | Medium | Move token to secure HTTP-only cookie set by backend, or implement backend proxy for query endpoint. |
| SEC-005 | Audit Logging Gaps | No comprehensive audit trail: missing IP address logging, user identity tracking, detailed event metadata. Makes post-incident forensics difficult. | Medium | Implement structured logging with user ID, IP, timestamp, action, resource, result. Integrate with log aggregation (e.g., Datadog, CloudWatch). |
| SEC-006 | Rate Limiting Not Tuned | Rate limiter is configured (100 tokens/sec, 100 burst) but limits are hardcoded. No adaptive limits based on deployment size or environment. | Medium | Move rate limit constants to environment variables. Test limits under production load. Consider distributed Redis backend for multi-instance deployments. |
| SEC-007 | Missing Security Headers | Edge Functions cannot set CSP, HSTS, X-Frame-Options, X-Content-Type-Options headers. Relies on Supabase/load-balancer configuration. | Medium | Document required security header configuration at deployment level. Verify in production via curl/browser inspection. |
| SEC-008 | Public Key Management | `VIBETEA_PUBLIC_KEYS` format is CSV-style string parsing prone to edge cases. No validation of key format before storing. | Low | Add validation on server startup to ensure all keys are valid base64-encoded 32-byte values. Provide admin CLI for key management. |
| SEC-009 | CORS Configuration Too Open | Ingest endpoint allows `*` origin. While public endpoints are intentional, should document CORS risk and consider restricting to known monitor domains in production. | Low | Document origin whitelist approach. Consider environment-based CORS configuration for ingest endpoint. |
| SEC-010 | Batch Signature Coverage | Single signature covers entire batch. If attacker can intercept and modify individual events within batch, server will reject entire batch (not partial modification). Design is secure but means batch atomicity is signature-level, not event-level. | Low | Document this behavior in API contracts. Consider per-event MAC if fine-grained verification becomes required. |
| SEC-011 | Non-Blocking Persistence Risk | Events accepted (HTTP 202/200) before database persistence completes. Client has no guarantee events were stored. Network/database failures could lose events. | Medium | Implement idempotency keys and exponential backoff on client. Add monitoring for persistence failures. Consider request payload storage for audit trail. |
| SEC-012 | Client Token Expiration Not Enforced | Bearer token for query endpoint has no expiration. If token is compromised, attacker has indefinite access to historic data. | Medium | Implement token expiration and refresh mechanism. Add ability to revoke tokens server-side without redeployment. |

## Technical Debt

### High Priority

Items that should be addressed soon:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | Database Functions Security | RPC functions `bulk_insert_events` and `get_hourly_aggregates` use `SECURITY DEFINER` which runs with elevated privileges. No input sanitization at function level (relies on client-side validation). | Security risk | Medium |
| TD-002 | Ingest Endpoint Input Validation | Event validation in edge function (lines 115-209 of `supabase/functions/ingest/index.ts`) could be extracted to reusable schema library. Manual validation is error-prone. | Maintainability | Medium |
| TD-004 | Bearer Token Refresh | No mechanism for token refresh or rotation. Long-lived tokens pose security risk if leaked. | Security/UX | Medium |
| TD-011 | Client-Side Token Security | Token stored in plain localStorage without encryption or secure transmission. Environment variables embedded in build. | Security | Medium |

### Medium Priority

Items to address when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-003 | Missing TypeScript Strict Mode | Edge functions use implicit `any` types in places (e.g., `as Record<string, unknown>`). Should enable TypeScript strict mode. | Type safety | Low |
| TD-005 | RPC Function Documentation | `bulk_insert_events` and `get_hourly_aggregates` lack inline documentation of security model (SECURITY DEFINER, input assumptions). | Maintainability | Low |
| TD-006 | Test Coverage for Auth | Ed25519 signature tests in `server/src/auth.rs` are comprehensive (22 tests), but bearer token tests could be more extensive (timing attack resistance, edge cases). | Testing | Low |
| TD-007 | Error Message Disclosure | Some error messages reveal server state (e.g., "Unknown source: {sourceId}", "No public key found for source"). Could be abused for enumeration. | Security/UX | Low |
| TD-012 | Client Environment Variables | `VITE_SUPABASE_TOKEN` visible in client bundle and source maps. Sensitive values should not be in build artifacts. | Security | Medium |

### Low Priority

Nice to have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-008 | Configuration Validation | No startup health check to verify `VIBETEA_PUBLIC_KEYS` format is valid before server starts. Bad config silently fails at runtime. | Debugging | Low |
| TD-009 | Cryptographic Key Rotation | Monitor key rotation process not documented. Manual public key registration has no versioning/expiration mechanism. | Operations | Medium |
| TD-010 | Retry Backoff on Client | WebSocket reconnection uses exponential backoff but no jitter option for production deployments with many clients. All clients could reconnect simultaneously. | Performance | Low |
| TD-013 | Query Endpoint Error Messages | Error responses from `/query` endpoint could leak information about token validity. Should return generic 401/403 without detailed messages. | Security | Low |

## Known Bugs

No active bugs documented at this time. The following areas were tested and pass:

- Ed25519 signature verification with various key lengths and message sizes (see `server/src/auth.rs` tests)
- Bearer token constant-time comparison (see `server/src/auth.rs` tests)
- RLS enforcement on events table (see `supabase/functions/_tests/rls.test.ts`)
- Event schema validation (see `supabase/functions/ingest/index.test.ts`)
- Batch processing with duplicate event IDs (duplicates properly skipped)
- Rate limiter token bucket refill and exhaustion (see `server/src/rate_limit.rs` tests)
- Client-side historic data fetch with stale-while-revalidate pattern (see `client/src/__tests__/hooks/useHistoricData.test.tsx`)
- Token form with localStorage persistence and cross-tab synchronization (see `client/src/__tests__/components/TokenForm.test.tsx`)

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `supabase/migrations/20260203000000_create_events_table.sql` | RLS is critical security boundary. Adding policies requires careful testing. | Add integration tests before modifying. Never remove ENABLE/FORCE ROW LEVEL SECURITY. |
| `supabase/migrations/20260203000001_create_functions.sql` | SECURITY DEFINER functions with elevated privileges. Logic errors could expose data. | Thoroughly test RPC input validation. Use parameterized queries. Never trust client input. |
| `server/src/auth.rs` verify_signature() | Cryptographic verification is security-critical. Small changes could introduce timing attacks or verification bypasses. | Keep constant-time comparison. Only update if upgrading ed25519_dalek crate. Run full test suite on changes. |
| `supabase/functions/_shared/auth.ts` | Shared by both ingest and query endpoints. Changes affect both authentication flows. | Add tests for both Ed25519 and bearer token paths. Verify cross-platform compatibility. |
| `server/src/rate_limit.rs` | Token bucket algorithm must maintain correctness across concurrent requests. Race conditions could bypass limits. | All modifications require thorough testing under high concurrency. Use `RwLock` as-is. Consider distributed backing store for prod. |
| `client/src/hooks/useEventStore.ts` fetchHistoricData | Makes HTTP requests with bearer token auth. Errors expose configuration issues. Sensitive auth flow. | All changes require testing for auth failure scenarios. Validate error handling paths. |
| `client/src/components/TokenForm.tsx` | Handles sensitive token input. localStorage changes affect entire app. Cross-tab communication via storage events. | Test localStorage persistence across browsers. Verify storage event handling. Test token update flow. |

## Deprecated Code

No deprecated code identified at this time.

## TODO Items

Active TODO comments in codebase:

| Location | TODO | Priority |
|----------|------|----------|
| `server/src/auth.rs` (line 282) | Length comparison not constant-time, but acceptable because length differences are not sensitive (addressed in comment) | Low |
| `supabase/functions/_shared/auth.ts` (line 107-108) | Constant-time comparison comment suggests production consideration. Currently uses simple `===` due to Deno limitations. | Low |
| `server/src/rate_limit.rs` (line 135) | Rate limit values marked TBD but now configured as 100 req/sec, 100 burst (per spec). Should update documentation if adjusting. | Low |

## Improvement Opportunities

Areas that could benefit from enhancement:

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Public Key Management | Manual env var configuration string (`source_id:key,source_id2:key2`) | Admin API or CLI for key management | Easier operations, fewer deployment errors |
| Token Management | Fixed bearer tokens in env vars (server) or localStorage (client) | JWT with short expiration + refresh tokens | Better security, easier rotation |
| Client Token Storage | Browser localStorage | Session storage or HTTP-only cookie set by backend | Better security against XSS |
| Audit Logging | Basic error responses to clients | Structured event logging with IP, user ID, details | Forensics, compliance, debugging |
| Database Functions | Manual input validation in edge function | Shared schema validation library | Consistency, reusability, maintainability |
| Rate Limiting | Hardcoded limits (100 req/sec) | Environment-based configuration + Redis backing | Flexibility, multi-instance support |
| RPC Functions | No inline documentation | Documented security model and input assumptions | Maintainability, fewer security bugs |
| Error Handling | Mixed error types and formats | Standardized error codes and messages | Client consistency, reduced enumeration risk |
| Persistence Resilience | Non-blocking with no client feedback | Idempotency keys + exponential backoff on client | Better reliability, fewer lost events |
| Client Token Security | Environment variables in build + localStorage | Backend proxy with secure token refresh | Better defense-in-depth |

## External Dependencies at Risk

Dependencies that may need attention:

| Package | Concern | Action Needed |
|---------|---------|---------------|
| `ed25519_dalek` | Cryptographic library with active maintenance; monitor for security advisories | Subscribe to security updates, test upgrades |
| `@noble/ed25519` | Audited cryptographic library (noble security); maintained actively | Monitor for updates and breaking changes |
| `subtle` | Timing-attack resistant comparison utilities | Keep updated, verify constant-time properties |
| `axum` | Web framework with async/await; monitor for concurrency bugs | Keep updated, test under load |
| `tokio` | Runtime library; critical for async/rate-limiting correctness | Keep updated, watch for scheduler bugs |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact |
|------|---------|--------|
| Signature verification failures | No metrics on failure rates (unknown source vs. invalid sig) | Can't detect coordinated attacks or key mismanagement |
| Rate limit exhaustion | No metrics on how often limits are hit per source | Can't detect DDoS early or tune limits effectively |
| Database persistence | No metrics on RPC call success/failure rates | Can't detect database issues or data loss |
| WebSocket reconnections | No metrics on reconnection frequency/reasons | Can't detect client-side issues or network problems |
| Client-side auth failures | No metrics on query endpoint 401/403 rates | Can't detect compromised tokens or misconfiguration |
| Historic data fetch latency | No metrics on fetch duration or cache hit/miss rates | Can't detect performance issues or optimize caching |

---

## Concern Severity Guide

| Level | Definition | Response Time |
|-------|------------|----------------|
| Critical | Production impact, security breach | Immediate |
| High | Degraded functionality, security risk | This sprint |
| Medium | Developer experience, minor issues | Next sprint |
| Low | Nice to have, cosmetic | Backlog |

---

*This document tracks what needs attention. Update when concerns are resolved or discovered.*
