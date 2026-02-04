# VibeTea Project Guidelines

## Tech Stack

### Server (Rust)
- **Framework**: Axum 0.8 with Tokio 1.43 async runtime
- **Auth**: Supabase JWT validation via `/auth/v1/user` endpoint
- **Cryptography**: ed25519-dalek 2.1 for signature verification, subtle for constant-time comparison
- **HTTP Client**: reqwest 0.12 for Supabase API calls

### Client (TypeScript/React)
- **Framework**: React 19 with Vite 7.3
- **State**: Zustand 5.0
- **Auth**: @supabase/supabase-js for GitHub OAuth

### Supabase
- **Edge Functions**: Deno/TypeScript for public-keys endpoint
- **Database**: PostgreSQL for monitor_public_keys table

## Commands

### Development
```bash
# Start local Supabase
supabase start

# Run server
cd server && cargo run

# Run client
cd client && npm run dev
```

### Testing
```bash
# Server tests (single-threaded required)
cd server && cargo test --test-threads=1

# Client tests
cd client && npm test
```

### Linting
```bash
# Server
cd server && cargo clippy -- -D warnings

# Client
cd client && npm run lint
```

## Critical Learnings

### Test Parallelism (Phase 3, Phase 11)
- **Rust tests modifying environment variables** must run with `--test-threads=1` or use the `serial_test` crate
- The `EnvGuard` RAII pattern in `server/src/config.rs` saves/restores env vars during tests
- CI workflow uses `cargo test --workspace --test-threads=1` to prevent env var interference
- **Cargo test flag placement**: The `--test-threads` flag must come AFTER the `--` separator: `cargo test -p crate -- --test-threads=1` (not before `--`)

### Security Patterns (Phase 3)
- Use `subtle::ConstantTimeEq` for token comparison to prevent timing attacks
- Use `ed25519_dalek::VerifyingKey::verify_strict()` for RFC 8032 compliant signature verification
- Use `zeroize` crate for any intermediate buffers containing private key material
- Memory should be zeroed on **both** success and error paths for defense in depth

### Supabase Authentication (002-supabase-auth)
- **JWT Validation**: Use remote validation via `GET {SUPABASE_URL}/auth/v1/user` (simpler, handles revocation)
- **Session Tokens**: 32 bytes random, base64-url encoded (43 chars), 5-minute TTL
- **Public Key Refresh**: Every 30 seconds from Supabase edge function, fallback to cached keys on failure
- **Token Generation**: Use `rand::rng().fill_bytes()` for cryptographically secure random bytes

### TUI Features (Phase 11)
- **Default mode**: Running `vibetea-monitor` without arguments launches the TUI
- **Headless mode**: Use `vibetea-monitor run` for scripting/CI
- **Terminal safety**: RAII-based terminal restoration via `Tui` struct with panic hook
- **NO_COLOR support**: Set `NO_COLOR` environment variable (any value) to disable colors
- **Minimum terminal size**: 80x24 characters required for proper display
- **60ms tick rate**: `DEFAULT_TICK_RATE_MS` in `tui/app.rs` for ~16 FPS rendering
- **NFR-005 compliance**: Logging is suppressed in TUI mode to avoid display corruption
