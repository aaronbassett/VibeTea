# Known Concerns

> **Purpose**: Document technical debt, known risks, bugs, fragile areas, and improvement opportunities.
> **Generated**: 2026-02-03
> **Last Updated**: 2026-02-03

## Security Concerns

Security-related issues requiring attention:

| ID | Area | Description | Risk Level | Mitigation |
|----|------|-------------|------------|------------|
| SEC-001 | Bearer Token Storage | `VIBETEA_SUBSCRIBER_TOKEN` is a plain string with no expiration or rotation mechanism. Clients receive this via environment variables and store in localStorage/browser memory. | Medium | Implement JWT tokens with expiration and refresh mechanism. Add token rotation policy. |
| SEC-002 | Edge Function Bearer Token Comparison | `validateBearerToken()` in `supabase/functions/_shared/auth.ts` uses simple `===` comparison instead of constant-time comparison. Timing attacks possible but impractical due to network latency. | Low | Replace with constant-time comparison utility (though network timing makes this low-priority). |
| SEC-003 | Audit Logging Gaps | No comprehensive audit trail: missing IP address logging, user identity tracking, detailed event metadata. Makes post-incident forensics difficult. | Medium | Implement structured logging with user ID, IP, timestamp, action, resource, result. Integrate with log aggregation (e.g., Datadog, CloudWatch). |
| SEC-004 | Rate Limiting Not Configured | Rate limiter objects exist in `server/src/rate_limit.rs` but limits are not configured (marked TBD in SECURITY.md). No protection against brute-force or DoS at application level. | Medium | Configure rate limits: POST /events (e.g., 100 req/min per source), GET /ws (e.g., 10 connections per source). Add Redis-backed distributed rate limiting for multi-instance deployments. |
| SEC-005 | Missing Security Headers | Edge Functions cannot set CSP, HSTS, X-Frame-Options, X-Content-Type-Options headers. Relies on Supabase/load-balancer configuration. | Medium | Document required security header configuration at deployment level. Verify in production via curl/browser inspection. |
| SEC-006 | Public Key Management | `VIBETEA_PUBLIC_KEYS` format is CSV-style string parsing prone to edge cases. No validation of key format before storing. | Low | Add validation on server startup to ensure all keys are valid base64-encoded 32-byte values. Provide admin CLI for key management. |
| SEC-007 | CORS Configuration Too Open | Ingest endpoint allows `*` origin. While public endpoints are intentional, should document CORS risk and consider restricting to known monitor domains in production. | Low | Document origin whitelist approach. Consider environment-based CORS configuration for ingest endpoint. |

## Technical Debt

### High Priority

Items that should be addressed soon:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-001 | Database Functions Security | RPC functions `bulk_insert_events` and `get_hourly_aggregates` use `SECURITY DEFINER` which runs with elevated privileges. No input sanitization at function level (relies on client-side validation). | Security risk | Medium |
| TD-002 | Ingest Endpoint Input Validation | Event validation in edge function (lines 115-209 of `supabase/functions/ingest/index.ts`) could be extracted to reusable schema library. Manual validation is error-prone. | Maintainability | Medium |
| TD-003 | Missing TypeScript Strict Mode | Edge functions use implicit `any` types in places (e.g., `as Record<string, unknown>`). Should enable TypeScript strict mode. | Type safety | Low |

### Medium Priority

Items to address when working in the area:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-004 | Bearer Token Refresh | No mechanism for token refresh or rotation. Long-lived tokens pose security risk if leaked. | Security/UX | Medium |
| TD-005 | RPC Function Documentation | `bulk_insert_events` and `get_hourly_aggregates` lack inline documentation of security model (SECURITY DEFINER, input assumptions). | Maintainability | Low |
| TD-006 | Test Coverage for Auth | Ed25519 signature tests in `server/src/auth.rs` are comprehensive, but bearer token tests could be more extensive (timing attack resistance, edge cases). | Testing | Low |
| TD-007 | Error Message Disclosure | Some error messages reveal server state (e.g., "Unknown source: {sourceId}", "No public key found for source"). Could be abused for enumeration. | Security/UX | Low |

### Low Priority

Nice to have improvements:

| ID | Area | Description | Impact | Effort |
|----|------|-------------|--------|--------|
| TD-008 | Configuration Validation | No startup health check to verify `VIBETEA_PUBLIC_KEYS` format is valid before server starts. Bad config silently fails at runtime. | Debugging | Low |
| TD-009 | Cryptographic Key Rotation | Monitor key rotation process not documented. Manual public key registration has no versioning/expiration mechanism. | Operations | Medium |

## Known Bugs

No active bugs documented at this time. The following areas were tested and pass:

- Ed25519 signature verification with various key lengths and message sizes (see `server/src/auth.rs` tests)
- Bearer token constant-time comparison (see `server/src/auth.rs` tests)
- RLS enforcement on events table (see `supabase/functions/_tests/rls.test.ts`)
- Event schema validation (see `supabase/functions/ingest/index.test.ts`)

## Fragile Areas

Code areas that are brittle or risky to modify:

| Area | Why Fragile | Precautions |
|------|-------------|-------------|
| `supabase/migrations/20260203000000_create_events_table.sql` | RLS is critical security boundary. Adding policies requires careful testing. | Add integration tests before modifying. Never remove ENABLE/FORCE ROW LEVEL SECURITY. |
| `supabase/migrations/20260203000001_create_functions.sql` | SECURITY DEFINER functions with elevated privileges. Logic errors could expose data. | Thoroughly test RPC input validation. Use parameterized queries. Never trust client input. |
| `server/src/auth.rs` verify_signature() | Cryptographic verification is security-critical. Small changes could introduce timing attacks or verification bypasses. | Keep constant-time comparison. Only update if upgrading ed25519_dalek crate. Run full test suite on changes. |
| `supabase/functions/_shared/auth.ts` | Shared by both ingest and query endpoints. Changes affect both authentication flows. | Add tests for both Ed25519 and bearer token paths. Verify cross-platform compatibility. |

## Deprecated Code

No deprecated code identified at this time.

## TODO Items

Active TODO comments in codebase:

| Location | TODO | Priority |
|----------|------|----------|
| `server/src/auth.rs` (line 282) | Length comparison not constant-time, but acceptable because length differences are not sensitive (addressed in comment) | Low |
| `supabase/functions/_shared/auth.ts` (line 107-108) | Constant-time comparison comment suggests production consideration. Currently uses simple `===` due to Deno limitations. | Low |

## Improvement Opportunities

Areas that could benefit from enhancement:

| Area | Current State | Desired State | Benefit |
|------|---------------|---------------|---------|
| Public Key Management | Manual env var configuration string (`source_id:key,source_id2:key2`) | Admin API or CLI for key management | Easier operations, fewer deployment errors |
| Token Management | Fixed bearer tokens in env vars | JWT with short expiration + refresh tokens | Better security, easier rotation |
| Audit Logging | Basic error responses to clients | Structured event logging with IP, user ID, details | Forensics, compliance, debugging |
| Database Functions | Manual input validation in edge function | Shared schema validation library | Consistency, reusability, maintainability |
| Rate Limiting | Placeholder rate limiter exists | Configured limits + distributed Redis backend | Production-ready protection |
| RPC Functions | No inline documentation | Documented security model and input assumptions | Maintainability, fewer security bugs |
| Error Handling | Mixed error types and formats | Standardized error codes and messages | Client consistency, reduced enumeration risk |

## External Dependencies at Risk

Dependencies that may need attention:

| Package | Type | Concern | Action Needed |
|---------|------|---------|---------------|
| `ed25519_dalek` | Rust | Active library, regularly maintained. RFC 8032 strict compliance is security-critical. | Monitor for security advisories. Test on new major versions. |
| `@noble/ed25519` | TypeScript | Active library, reputable cryptography focus. Used in Edge Functions. | Monitor for updates. Verify RFC 8032 compliance on upgrades. |
| `subtle` | Rust | Provides `ConstantTimeEq` trait. Niche library but security-critical. | Monitor for updates. No immediate action required. |
| Supabase RLS | Service | RLS is the primary access control mechanism. Depends on Supabase implementation. | Regularly review Supabase security advisories. Test RLS on new Supabase versions. |

## Performance Concerns

| ID | Area | Description | Impact | Mitigation |
|----|------|-------------|--------|------------|
| PERF-001 | Bearer Token Comparison | Constant-time comparison with `ct_eq` adds negligible overhead but is still a tight loop. Network latency dominates. | Negligible | No action needed. |
| PERF-002 | Base64 Decoding | Multiple base64 decode operations per signature verification. Could be optimized with buffering but impact is minimal. | Minor | No action needed. |
| PERF-003 | RPC Latency | `bulk_insert_events` RPC roundtrip for each batch adds latency. Network overhead dominates. | Acceptable | No action needed unless batching causes issues. |

## Monitoring Gaps

Areas lacking proper observability:

| Area | Missing | Impact | Priority |
|------|---------|--------|----------|
| Authentication Metrics | No metrics for signature verification success/failure rates by source | Can't detect suspicious patterns or key rotation issues | Medium |
| Event Ingestion Metrics | No metrics for batch size, duplicate rate, validation failures | Hard to optimize batch settings or detect schema mismatches | Medium |
| RPC Function Performance | No metrics for `bulk_insert_events` and `get_hourly_aggregates` execution time | Can't detect performance degradation or optimization opportunities | Low |
| Rate Limiter Status | No metrics for rate limit hits or bypass patterns | Can't validate rate limiting effectiveness | Medium |
| Database RLS Enforcement | No audit trail of RLS policy rejections | Hard to debug access control issues | Low |

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
