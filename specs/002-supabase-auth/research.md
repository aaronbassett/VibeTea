# Research: Supabase Authentication for VibeTea

**Date**: 2026-02-04
**Branch**: `002-supabase-auth`
**Status**: Complete

## Table of Contents

1. [Supabase JWT Validation from Rust Server](#1-supabase-jwt-validation-from-rust-server)
2. [Session Token Management Best Practices](#2-session-token-management-best-practices)
3. [Supabase Edge Functions](#3-supabase-edge-functions)
4. [HTTP Client Patterns in Rust](#4-http-client-patterns-in-rust)
5. [Recommendations for VibeTea](#5-recommendations-for-vibetea)

---

## 1. Supabase JWT Validation from Rust Server

### Overview

There are two primary approaches to validate Supabase JWTs from a Rust server:

1. **Remote validation** via Supabase's `/auth/v1/user` endpoint
2. **Local validation** using the `jsonwebtoken` crate

### Approach A: Remote Validation via Supabase API

Call Supabase's `/auth/v1/user` endpoint with the JWT in the Authorization header.

```rust
// Pseudocode
async fn validate_jwt_remote(
    client: &reqwest::Client,
    supabase_url: &str,
    jwt: &str,
) -> Result<User, AuthError> {
    let response = client
        .get(format!("{}/auth/v1/user", supabase_url))
        .header("Authorization", format!("Bearer {}", jwt))
        .header("apikey", &supabase_anon_key)
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => Ok(response.json::<User>().await?),
        StatusCode::UNAUTHORIZED => Err(AuthError::InvalidToken),
        _ => Err(AuthError::ServiceUnavailable),
    }
}
```

**Pros:**
- Always reflects current token state (handles revocation immediately)
- No need to manage JWT secrets or JWKS
- Simpler implementation - Supabase handles all validation logic
- Strongly recommended by Supabase for HS256/shared-secret tokens

**Cons:**
- Network latency on every validation (~50-200ms)
- Dependency on Supabase availability
- Rate limits may apply for high-volume validation

### Approach B: Local JWT Validation with `jsonwebtoken` Crate

Parse and validate the JWT locally using cryptographic verification.

```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SupabaseClaims {
    sub: String,           // User ID (UUID)
    aud: String,           // "authenticated"
    role: String,          // "authenticated"
    exp: usize,            // Expiration timestamp
    iat: usize,            // Issued at
    email: Option<String>, // User email
}

fn validate_jwt_local(jwt: &str, jwt_secret: &str) -> Result<SupabaseClaims, AuthError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_audience(&["authenticated"]);

    let token_data = decode::<SupabaseClaims>(
        jwt,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    )?;

    Ok(token_data.claims)
}
```

**Pros:**
- Fast - no network call (~microseconds)
- Works offline / when Supabase is unavailable
- No rate limit concerns
- Lower latency for high-frequency validation

**Cons:**
- Doesn't detect token revocation until expiry
- Requires secure storage of JWT secret
- Must handle JWKS rotation manually (for asymmetric keys)
- Supabase is transitioning to asymmetric keys (RS256/ES256) by late 2026

### JWKS-Based Validation (Future-Proof Approach)

Supabase is migrating from HS256 to asymmetric JWT signing:
- **Now**: JWKS available as opt-in
- **October 2025**: New projects use asymmetric JWTs by default
- **Late 2026**: All projects expected to transition

For JWKS-based validation:

```rust
// Using supabase-jwt crate or manual JWKS fetch
let jwks_url = format!("{}/.well-known/jwks.json", supabase_url);
// Fetch and cache JWKS (cache for ~10 minutes per Supabase docs)
// Validate using public key from JWKS
```

The [`supabase-jwt`](https://crates.io/crates/supabase-jwt) crate provides JWKS caching support.

### Recommendation for VibeTea

**Use remote validation via `/auth/v1/user` endpoint** because:
1. Spec requires single validation per session token exchange (not per request)
2. Simpler implementation aligns with Constitution (Keep It Simple)
3. Handles token revocation correctly
4. No secret management required
5. Supabase explicitly recommends this for HS256 tokens

The latency (~100ms) is acceptable since validation only happens on session token exchange, not per WebSocket message.

---

## 2. Session Token Management Best Practices

### Token Generation

#### Cryptographic Requirements

Per [OWASP Session Management guidelines](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html):
- **Minimum entropy**: 128 bits (256 bits recommended)
- **Source**: Cryptographically secure random number generator (CSPRNG)

#### Rust Implementation

```rust
use rand::RngCore;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};

/// Generate a 32-byte (256-bit) cryptographically secure session token
fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
```

**Token Format Recommendation:**
- **Size**: 32 bytes (256 bits of entropy)
- **Encoding**: Base64-URL (43 characters, URL-safe, no padding)
- **Alternative**: Hex encoding (64 characters, simpler but longer)

The `rand` crate's `thread_rng()` implements `CryptoRng` and uses ChaCha12, which is cryptographically secure.

### In-Memory Store with TTL

#### Architecture Pattern

```rust
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct SessionStore {
    sessions: RwLock<HashMap<String, SessionEntry>>,
    ttl: Duration,
    max_capacity: usize,
}

struct SessionEntry {
    user_id: String,
    created_at: Instant,
    expires_at: Instant,
}

impl SessionStore {
    pub fn new(ttl: Duration, max_capacity: usize) -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            ttl,
            max_capacity,
        }
    }

    /// Insert a new session, returns the token
    pub fn create(&self, user_id: String) -> Result<String, SessionError> {
        let token = generate_session_token();
        let now = Instant::now();
        let entry = SessionEntry {
            user_id,
            created_at: now,
            expires_at: now + self.ttl,
        };

        let mut sessions = self.sessions.write().unwrap();

        // Check capacity before inserting
        if sessions.len() >= self.max_capacity {
            return Err(SessionError::CapacityExceeded);
        }

        sessions.insert(token.clone(), entry);
        Ok(token)
    }

    /// Validate and optionally extend TTL
    pub fn validate(&self, token: &str, extend_ttl: bool) -> Option<String> {
        let mut sessions = self.sessions.write().unwrap();

        if let Some(entry) = sessions.get_mut(token) {
            let now = Instant::now();
            // Include grace period for clock skew (30 seconds per spec)
            let grace = Duration::from_secs(30);

            if entry.expires_at + grace > now {
                if extend_ttl {
                    entry.expires_at = now + self.ttl;
                }
                return Some(entry.user_id.clone());
            }
        }
        None
    }

    /// Remove expired sessions
    pub fn cleanup(&self) {
        let mut sessions = self.sessions.write().unwrap();
        let now = Instant::now();
        sessions.retain(|_, entry| entry.expires_at > now);
    }
}
```

### Cleanup Strategies

1. **Lazy Expiration**: Check expiry on access (already in `validate()`)
2. **Periodic Sweep**: Background task removes expired entries

```rust
// Spawn cleanup task in main.rs
let store = Arc::new(SessionStore::new(Duration::from_secs(300), 10_000));
let cleanup_store = store.clone();

tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        cleanup_store.cleanup();
    }
});
```

### Token Size Recommendations

| Bits | Hex Length | Base64 Length | Security Level |
|------|------------|---------------|----------------|
| 128  | 32 chars   | 22 chars      | Minimum acceptable |
| 192  | 48 chars   | 32 chars      | Good |
| 256  | 64 chars   | 43 chars      | Recommended |

**VibeTea Spec Requirement**: At least 32 bytes (256 bits) - FR-021

---

## 3. Supabase Edge Functions

### Overview

Supabase Edge Functions are server-side TypeScript functions running on Deno at the edge. They're ideal for the public keys endpoint because:
- Low latency (globally distributed)
- Direct database access
- No authentication required for public endpoints

### Creating an Edge Function

#### Step 1: Initialize

```bash
# In project root
supabase init  # If not already initialized
supabase functions new public-keys
```

This creates `supabase/functions/public-keys/index.ts`.

#### Step 2: Implement the Function

```typescript
// supabase/functions/public-keys/index.ts
import { createClient } from 'https://esm.sh/@supabase/supabase-js@2'

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Headers': 'authorization, x-client-info, apikey, content-type',
}

Deno.serve(async (req) => {
  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    return new Response('ok', { headers: corsHeaders })
  }

  try {
    // Create Supabase client with service role key (bypasses RLS)
    const supabaseUrl = Deno.env.get('SUPABASE_URL')!
    const serviceRoleKey = Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!

    const supabase = createClient(supabaseUrl, serviceRoleKey)

    // Query the monitor_public_keys table
    const { data, error } = await supabase
      .from('monitor_public_keys')
      .select('source_id, public_key')
      .eq('active', true)

    if (error) {
      console.error('Database error:', error)
      return new Response(
        JSON.stringify({ error: 'Failed to fetch public keys' }),
        { status: 500, headers: { ...corsHeaders, 'Content-Type': 'application/json' } }
      )
    }

    return new Response(
      JSON.stringify({ keys: data }),
      { headers: { ...corsHeaders, 'Content-Type': 'application/json' } }
    )
  } catch (err) {
    console.error('Unexpected error:', err)
    return new Response(
      JSON.stringify({ error: 'Internal server error' }),
      { status: 500, headers: { ...corsHeaders, 'Content-Type': 'application/json' } }
    )
  }
})
```

### Direct PostgreSQL Access (Alternative)

For more complex queries or transactions, use the Deno Postgres driver:

```typescript
import { Pool } from 'https://deno.land/x/postgres@v0.17.0/mod.ts'

const pool = new Pool({
  tls: { enabled: true },
}, 10) // max 10 connections

Deno.serve(async (req) => {
  const client = await pool.connect()
  try {
    const result = await client.queryObject`
      SELECT source_id, public_key
      FROM monitor_public_keys
      WHERE active = true
    `
    return new Response(JSON.stringify({ keys: result.rows }))
  } finally {
    client.release()
  }
})
```

### Deployment

```bash
# Local testing
supabase start
supabase functions serve public-keys

# Deploy to production
supabase login
supabase link --project-ref YOUR_PROJECT_ID
supabase functions deploy public-keys --no-verify-jwt
```

The `--no-verify-jwt` flag is essential for public endpoints that don't require authentication.

### Authentication Patterns

| Pattern | Use Case | Key Used |
|---------|----------|----------|
| Anon key | Client-side, respects RLS | `SUPABASE_ANON_KEY` |
| Service role | Server-side, bypasses RLS | `SUPABASE_SERVICE_ROLE_KEY` |
| User JWT | User context, respects RLS | Passed in Authorization header |

**For public-keys endpoint**: Use service role key (bypasses RLS) since this is a public endpoint returning non-sensitive data.

### Database Migration

```sql
-- supabase/migrations/001_public_keys.sql
CREATE TABLE monitor_public_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Index for active keys lookup
CREATE INDEX idx_monitor_public_keys_active ON monitor_public_keys(active) WHERE active = true;

-- Update timestamp trigger
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER monitor_public_keys_updated_at
    BEFORE UPDATE ON monitor_public_keys
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at();
```

---

## 4. HTTP Client Patterns in Rust

### Using reqwest

The project already includes `reqwest 0.12` with JSON feature.

#### Basic Setup

```rust
use reqwest::Client;
use std::time::Duration;

fn create_http_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .pool_max_idle_per_host(5)
        .build()
        .expect("Failed to create HTTP client")
}
```

#### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SupabaseError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed")]
    Unauthorized,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),
}

async fn call_supabase(
    client: &Client,
    url: &str,
    jwt: &str,
) -> Result<serde_json::Value, SupabaseError> {
    let response = client
        .get(url)
        .bearer_auth(jwt)
        .send()
        .await?;

    match response.status() {
        status if status.is_success() => {
            Ok(response.json().await?)
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            Err(SupabaseError::Unauthorized)
        }
        reqwest::StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            Err(SupabaseError::RateLimited(retry_after))
        }
        status if status.is_server_error() => {
            Err(SupabaseError::ServiceUnavailable)
        }
        _ => {
            Err(SupabaseError::InvalidResponse(
                format!("Unexpected status: {}", response.status())
            ))
        }
    }
}
```

### Retry with Exponential Backoff

#### Option 1: Using `backoff` Crate

```toml
# Cargo.toml
[dependencies]
backoff = { version = "0.4", features = ["tokio"] }
```

```rust
use backoff::{ExponentialBackoff, Error as BackoffError};
use backoff::future::retry;
use std::time::Duration;

async fn fetch_with_retry<T, F, Fut>(
    operation: F,
) -> Result<T, SupabaseError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, SupabaseError>>,
{
    let backoff = ExponentialBackoff {
        initial_interval: Duration::from_millis(100),
        max_interval: Duration::from_secs(10),
        max_elapsed_time: Some(Duration::from_secs(60)),
        multiplier: 2.0,
        randomization_factor: 0.5,
        ..Default::default()
    };

    retry(backoff, || async {
        match operation().await {
            Ok(value) => Ok(value),
            Err(SupabaseError::Network(_)) => {
                Err(BackoffError::transient(SupabaseError::ServiceUnavailable))
            }
            Err(SupabaseError::ServiceUnavailable) => {
                Err(BackoffError::transient(SupabaseError::ServiceUnavailable))
            }
            Err(SupabaseError::RateLimited(_)) => {
                Err(BackoffError::transient(SupabaseError::ServiceUnavailable))
            }
            Err(e) => Err(BackoffError::permanent(e)),
        }
    }).await
}
```

#### Option 2: Using `reqwest-retry` Middleware

```toml
# Cargo.toml
[dependencies]
reqwest-middleware = "0.4"
reqwest-retry = "0.7"
```

```rust
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};

fn create_retry_client() -> ClientWithMiddleware {
    let retry_policy = ExponentialBackoff::builder()
        .build_with_max_retries(3);

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}
```

### Startup Retry Pattern (FR-018, FR-019, FR-020)

```rust
use std::time::Duration;
use tokio::time::sleep;

pub async fn fetch_public_keys_with_startup_retry(
    client: &Client,
    url: &str,
    max_retries: u32,
) -> Result<Vec<PublicKey>, SupabaseError> {
    let mut attempt = 0;
    let base_delay = Duration::from_secs(1);

    loop {
        attempt += 1;

        match fetch_public_keys(client, url).await {
            Ok(keys) => return Ok(keys),
            Err(e) if attempt >= max_retries => {
                tracing::error!(
                    "Failed to fetch public keys after {} attempts: {}",
                    max_retries,
                    e
                );
                return Err(e);
            }
            Err(e) => {
                // Exponential backoff with jitter
                let delay = base_delay * 2u32.pow(attempt - 1);
                let jitter = rand::random::<f64>() * 0.5 + 0.75; // 0.75 to 1.25
                let delay_with_jitter = delay.mul_f64(jitter);

                tracing::warn!(
                    "Failed to fetch public keys (attempt {}/{}): {}. Retrying in {:?}",
                    attempt,
                    max_retries,
                    e,
                    delay_with_jitter
                );

                sleep(delay_with_jitter).await;
            }
        }
    }
}
```

---

## 5. Recommendations for VibeTea

### Summary of Decisions

| Area | Recommendation | Rationale |
|------|----------------|-----------|
| JWT Validation | Remote via `/auth/v1/user` | Simpler, handles revocation, Supabase recommended |
| Session Token Size | 32 bytes (256 bits) | OWASP recommended, spec requirement |
| Token Encoding | Base64-URL | URL-safe, compact (43 chars) |
| Session Store | `RwLock<HashMap>` with periodic cleanup | Simple, sufficient for single-server |
| HTTP Client | `reqwest` with manual retry | Already in workspace, avoid new dependencies |
| Edge Function | Deno + supabase-js | Simplest approach, good docs |

### New Dependencies

```toml
# server/Cargo.toml additions
[dependencies]
# Already in workspace: reqwest, rand, base64, tokio, serde, thiserror

# Optional - for retry middleware (alternative to manual retry)
# backoff = { version = "0.4", features = ["tokio"] }
```

No new crate dependencies required. The existing workspace dependencies (`reqwest`, `rand`, `base64`, `tokio`) are sufficient.

### Configuration

New environment variables required:

```bash
# Server
SUPABASE_URL=https://YOUR_PROJECT.supabase.co
SUPABASE_ANON_KEY=your-anon-key
SUPABASE_PUBLIC_KEYS_URL=https://YOUR_PROJECT.supabase.co/functions/v1/public-keys

# Session token configuration
SESSION_TOKEN_TTL_SECS=300         # 5 minutes
SESSION_TOKEN_MAX_CAPACITY=10000   # Maximum concurrent sessions
SESSION_CLEANUP_INTERVAL_SECS=60   # Cleanup every minute

# Client (Vite)
VITE_SUPABASE_URL=https://YOUR_PROJECT.supabase.co
VITE_SUPABASE_ANON_KEY=your-anon-key
```

### Security Considerations

1. **Never log tokens**: Session tokens and JWTs must not appear in logs
2. **Constant-time comparison**: Use `subtle::ConstantTimeEq` for token validation (already in workspace)
3. **HTTPS only**: All Supabase communication over HTTPS
4. **Service role key**: Only used in edge function, never exposed to clients

### Testing Strategy

1. **Unit tests**: Session store operations, token generation
2. **Integration tests**: JWT validation with wiremock (already in workspace)
3. **E2E tests**: Full OAuth flow (manual or Playwright)

---

## Sources

### Supabase Documentation
- [JSON Web Token (JWT)](https://supabase.com/docs/guides/auth/jwts)
- [JWT Signing Keys](https://supabase.com/docs/guides/auth/signing-keys)
- [JWT Claims Reference](https://supabase.com/docs/guides/auth/jwt-fields)
- [Edge Functions](https://supabase.com/docs/guides/functions)
- [Edge Functions Quickstart](https://supabase.com/docs/guides/functions/quickstart)
- [Connecting to Postgres from Edge Functions](https://supabase.com/docs/guides/functions/connect-to-postgres)
- [Understanding API Keys](https://supabase.com/docs/guides/api/api-keys)

### Rust Crates
- [jsonwebtoken](https://crates.io/crates/jsonwebtoken) - JWT encoding/decoding
- [supabase-jwt](https://crates.io/crates/supabase-jwt) - Supabase-specific JWT validation with JWKS
- [backoff](https://crates.io/crates/backoff) - Exponential backoff and retry
- [reqwest-retry](https://docs.rs/reqwest-retry) - Retry middleware for reqwest
- [rand](https://docs.rs/rand) - Cryptographically secure random number generation

### Security Guidelines
- [OWASP Session Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html)
- [OWASP Insufficient Session ID Length](https://owasp.org/www-community/vulnerabilities/Insufficient_Session-ID_Length)
- [Rust Rand Security](https://github.com/rust-random/rand/blob/master/SECURITY.md)

### Community Resources
- [Verifying Supabase JWT - GitHub Discussion](https://github.com/orgs/supabase/discussions/20763)
- [Building Authentication in Rust - Shuttle Blog](https://www.shuttle.dev/blog/2022/08/11/authentication-tutorial)
- [Transactions and RLS in Supabase Edge Functions](https://marmelab.com/blog/2025/12/08/supabase-edge-function-transaction-rls.html)
