# Phase 0: Research - Monitor GitHub Actions Deployment

**Feature Branch**: `004-monitor-gh-actions`
**Date**: 2026-02-03

## Research Questions

### RQ-1: Base64 Key Encoding Format

**Question**: What base64 format should be used for `VIBETEA_PRIVATE_KEY` environment variable?

**Research**:
- Ed25519 private key seeds are 32 bytes
- Standard Base64 (RFC 4648) encodes 32 bytes as 44 characters (with padding)
- URL-safe Base64 uses `-_` instead of `+/`, commonly used in web contexts
- GitHub Secrets support arbitrary string values up to 64KB

**Decision**: Standard Base64 (RFC 4648) with `+/` alphabet and `=` padding
**Rationale**:
- Consistent with existing `public_key_base64()` implementation in crypto.rs
- `base64::prelude::BASE64_STANDARD` crate already in use
- No URL encoding needed since value is passed via env var, not URL
- Spec explicitly states "URL-safe base64 is NOT supported" (FR-021)

**Alternatives Rejected**:
- URL-safe Base64: Not needed for env vars, adds complexity
- Hex encoding: Longer (64 chars), less standard for keys
- Raw bytes: Not valid string for env vars

### RQ-2: Private Key vs Full Keypair

**Question**: Should `VIBETEA_PRIVATE_KEY` contain just the seed or the full 64-byte expanded key?

**Research**:
- Ed25519 has two key representations:
  1. **Seed** (32 bytes): The secret random value from which keys derive
  2. **Expanded key** (64 bytes): seed || public key (used by some libraries)
- `ed25519_dalek::SigningKey` expects 32-byte seed via `SigningKey::from_bytes()`
- Current `Crypto::load()` reads 32 bytes from `key.priv`

**Decision**: Use 32-byte seed only
**Rationale**:
- Matches existing file-based key format in `~/.vibetea/key.priv`
- Smaller (44 chars base64 vs 88 chars)
- Aligns with Ed25519 best practices (seed is canonical secret)
- Spec states "key seed must be exactly 32 bytes" (FR-022)

**Alternatives Rejected**:
- Full 64-byte key: Larger, redundant (public key derivable from seed)
- PEM format: Overkill for simple env var, adds parsing complexity

### RQ-3: Environment Variable Precedence

**Question**: When both env var and file key exist, which should take precedence?

**Research**:
- Common patterns:
  1. **Env var wins**: 12-factor app principle, runtime configuration overrides defaults
  2. **File wins**: File is more explicit, env var as fallback
  3. **Error on conflict**: Force explicit choice

**Decision**: Environment variable takes precedence over file
**Rationale**:
- Follows 12-factor app principles (environment-specific config)
- In CI, env var is intentional override for that specific run
- Matches common tooling patterns (AWS SDK, kubectl, etc.)
- Spec explicitly states this behavior (FR-002)

**Alternatives Rejected**:
- File wins: Counter-intuitive for CI/CD users
- Error on conflict: Annoying for local dev with both sources

### RQ-4: Error Handling for Invalid Keys

**Question**: What errors should be reported and how?

**Research**:
- Possible errors:
  1. Invalid Base64 string (not decodable)
  2. Wrong length after decode (not 32 bytes)
  3. Valid key but not registered with server (401)
- Exit codes should distinguish config errors from runtime errors

**Decision**: Exit codes per spec (FR-026)
- **Code 0**: Success
- **Code 1**: Configuration error (invalid env var, missing key)
- **Code 2**: Runtime error

**Error Messages** (per FR-004):
- Invalid Base64: "VIBETEA_PRIVATE_KEY contains invalid Base64: {decode_error}"
- Wrong length: "VIBETEA_PRIVATE_KEY must decode to exactly 32 bytes, got {n} bytes"
- Whitespace note: Silently trimmed per FR-005 (no warning)

### RQ-5: Export Command Output Format

**Question**: How should `export-key` output the key?

**Research**:
- Output destinations:
  1. stdout: Can be piped to clipboard, secret managers
  2. File: Requires file path handling
- Current `init` command prints public key to stdout with instructions

**Decision**: Output only base64 key + newline to stdout (FR-003)
**Rationale**:
- Enables direct piping: `vibetea-monitor export-key | pbcopy`
- GitHub Actions: `gh secret set VIBETEA_PRIVATE_KEY < <(vibetea-monitor export-key)`
- All diagnostic/error messages to stderr (FR-023)

**Alternatives Rejected**:
- Include instructions: Breaks piping, use stderr for help
- Output to file: Adds complexity, stdout is more versatile

### RQ-6: GitHub Actions Integration Pattern

**Question**: What's the recommended pattern for running the monitor in GitHub Actions?

**Research**:
- Options:
  1. **Binary download**: Download pre-built release binary
  2. **Build from source**: `cargo build --release` (slow)
  3. **Docker container**: Pull image with monitor (complex)
  4. **Composite action**: Wraps binary download + setup

**Decision**: Composite GitHub Action (P3 priority)
**Rationale**:
- Simplest UX: `uses: org/repo/.github/actions/vibetea-monitor@v1`
- Handles binary download, permission setting, background execution
- Can be published to GitHub Marketplace

**Pattern for P2 (workflow without action)**:
```yaml
- name: Setup VibeTea Monitor
  env:
    VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
    VIBETEA_SERVER_URL: ${{ vars.VIBETEA_SERVER_URL }}
  run: |
    curl -sSL https://github.com/org/vibetea/releases/latest/download/vibetea-monitor-linux-amd64 -o vibetea-monitor
    chmod +x vibetea-monitor
    ./vibetea-monitor run &
```

### RQ-7: Graceful Shutdown in CI

**Question**: How should the monitor handle workflow completion?

**Research**:
- GitHub Actions sends SIGTERM to all processes on job completion
- Current monitor handles SIGTERM via `tokio::signal` (Phase 6)
- Default shutdown timeout is 5 seconds (configurable per spec FR-025)

**Decision**: Existing signal handling is sufficient
**Rationale**:
- Monitor already flushes buffered events on SIGTERM
- 5-second default timeout allows final batch to complete
- No special CI handling needed beyond existing implementation

### RQ-8: Source Identifier in CI

**Question**: What should `source` be set to in CI context?

**Research**:
- Default is hostname (gethostname crate)
- CI runners have ephemeral, non-descriptive hostnames
- GitHub provides context variables: `$GITHUB_REPOSITORY`, `$GITHUB_RUN_ID`

**Decision**: Support `VIBETEA_SOURCE_ID` env var (already exists)
**Rationale**:
- Already configurable via existing config.rs
- Recommended pattern: `github-${{ github.repository }}-${{ github.run_id }}`
- Enables event correlation with specific workflows

**Example**:
```yaml
env:
  VIBETEA_SOURCE_ID: "github-myorg/myrepo-${{ github.run_id }}"
```

## Learnings from Previous Retros

### From Phase 10 Retro
- **Applicable**: Pure function patterns work well for formatting/validation
- **Applied to**: Key validation functions should be pure, side-effect free

### From Phase 8 Retro
- **Applicable**: TypeScript type narrowing issues with generics
- **Not directly applicable**: This feature is Rust-only

### From Phase 6 Retro (implicit from STACK.md)
- **Applicable**: Crypto module patterns (load, save, sign) are well-established
- **Applied to**: Follow same patterns for env var loading

## Dependencies

### Existing (no changes)
- `base64` 0.22 - Already used for public key encoding
- `ed25519-dalek` 2.1 - Already used for signing
- `subtle` 2.6 - Already used for constant-time comparison (server-side)

### New
- None required

## Security Considerations

1. **Memory Zeroing**: After constructing `SigningKey`, zero intermediate buffers (FR-020)
   - Use `zeroize` crate or manual zeroing with `volatile_write`

2. **No Logging**: Never log `VIBETEA_PRIVATE_KEY` value (FR-019)
   - Log presence/absence only: "Loading private key from environment variable"

3. **Key Fingerprint**: Log public key fingerprint for verification (FR-007)
   - First 8 characters of base64 public key

## Summary of Decisions

| Question | Decision | Key Reason |
|----------|----------|------------|
| Base64 format | Standard (RFC 4648) | Matches existing implementation |
| Key size | 32-byte seed | Ed25519 standard, matches file format |
| Precedence | Env var over file | 12-factor app principles |
| Exit codes | 0/1/2 pattern | Config vs runtime error distinction |
| Export output | Base64 + newline to stdout | Enables piping |
| CI pattern | Composite action (P3) | Best UX for consumers |
| Shutdown | Existing SIGTERM handling | Already implemented |
| Source ID | Existing env var | Already configurable |
