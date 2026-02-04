# VibeTea Project Guidelines

## Critical Learnings

### Test Parallelism (Phase 3)
- **Rust tests modifying environment variables** must run with `--test-threads=1` or use the `serial_test` crate
- The `EnvGuard` RAII pattern in `server/src/config.rs` saves/restores env vars during tests
- CI workflow uses `cargo test --workspace --test-threads=1` to prevent env var interference

### Security Patterns (Phase 3)
- Use `subtle::ConstantTimeEq` for token comparison to prevent timing attacks
- Use `ed25519_dalek::VerifyingKey::verify_strict()` for RFC 8032 compliant signature verification

### Client Testing (Phase 5)
- **MSW v2 API**: Use `http.get()` and `HttpResponse.json()` (not v1's `rest.get()` and `res(ctx.json())`)
- **Zustand hook testing**: Mock store actions via `store.setState()`, reset in `beforeEach`
