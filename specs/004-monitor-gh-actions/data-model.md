# Phase 1: Data Model - Monitor GitHub Actions Deployment

**Feature Branch**: `004-monitor-gh-actions`
**Date**: 2026-02-03

## Key Entities

### PrivateKeySeed

The 32-byte Ed25519 private key seed used for signing events.

| Field | Type | Description |
|-------|------|-------------|
| bytes | `[u8; 32]` | Raw 32-byte Ed25519 seed |

**Encoding**: Base64 standard (RFC 4648) when stored in `VIBETEA_PRIVATE_KEY`

**Sources** (in precedence order):
1. `VIBETEA_PRIVATE_KEY` environment variable (base64-encoded)
2. `{VIBETEA_KEY_PATH}/key.priv` file (raw 32 bytes)

**Validation Rules**:
- Must decode to exactly 32 bytes
- Base64 string must use standard alphabet (`A-Za-z0-9+/`) with `=` padding
- Whitespace is trimmed before decoding

### KeySource

Enum indicating where the private key was loaded from.

```rust
pub enum KeySource {
    EnvironmentVariable,
    File(PathBuf),
}
```

**Usage**: Logged at startup (INFO level) per FR-007

### PublicKeyFingerprint

First 8 characters of base64-encoded public key for verification.

| Field | Type | Description |
|-------|------|-------------|
| fingerprint | `String` | First 8 chars of base64 public key |

**Example**: `"dGVzdHB1"` (first 8 chars of `"dGVzdHB1YmtleQ=="`)

**Usage**: Logged at startup for key verification (FR-007)

## State Transitions

### Key Loading State Machine

```
┌─────────────────────────────────────────────────────────┐
│                    Monitor Startup                       │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│         Check VIBETEA_PRIVATE_KEY env var               │
└─────────────────────────────────────────────────────────┘
                            │
              ┌─────────────┴─────────────┐
              │                           │
          [Present]                   [Absent]
              │                           │
              ▼                           ▼
┌─────────────────────┐     ┌─────────────────────────────┐
│ Trim whitespace     │     │ Check VIBETEA_KEY_PATH file │
└─────────────────────┘     └─────────────────────────────┘
              │                           │
              ▼                   ┌───────┴───────┐
┌─────────────────────┐       [Exists]        [Missing]
│ Decode Base64       │           │                │
└─────────────────────┘           │                ▼
              │                   │      ┌─────────────────┐
      ┌───────┴───────┐           │      │ Exit code 1     │
  [Success]       [Fail]          │      │ "No key found"  │
      │               │           │      └─────────────────┘
      ▼               ▼           ▼
┌──────────┐  ┌────────────┐ ┌──────────┐
│ Validate │  │ Exit code 1│ │ Load 32  │
│ 32 bytes │  │ "Invalid   │ │ bytes    │
└──────────┘  │ Base64"    │ └──────────┘
      │       └────────────┘      │
  ┌───┴───┐                       │
[32B]  [≠32B]                     │
  │       │                       │
  │       ▼                       │
  │  ┌────────────┐               │
  │  │ Exit code 1│               │
  │  │ "Must be   │               │
  │  │ 32 bytes"  │               │
  │  └────────────┘               │
  │                               │
  ▼                               ▼
┌─────────────────────────────────────────────────────────┐
│           Create SigningKey from seed                   │
│           Log key source + fingerprint (INFO)           │
│           Zero intermediate buffer                       │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│                  Monitor Ready                          │
└─────────────────────────────────────────────────────────┘
```

## Configuration Extensions

### New Environment Variable

| Variable | Type | Default | Required |
|----------|------|---------|----------|
| `VIBETEA_PRIVATE_KEY` | String (Base64) | None | No |

**Behavior**:
- When set: Load key from this value, ignore file
- When empty/whitespace: Treated as unset
- When missing: Fall back to file-based key

### Modified Config Struct

```rust
pub struct Config {
    pub server_url: String,
    pub source_id: String,
    pub key_path: PathBuf,       // File path for key.priv
    pub claude_dir: PathBuf,
    pub buffer_size: usize,
    pub basename_allowlist: Option<Vec<String>>,
    // New field (derived at runtime, not stored in config):
    // key_source: KeySource is determined during Crypto::load()
}
```

Note: `VIBETEA_PRIVATE_KEY` is not stored in Config; it's consumed directly by `Crypto::load_from_env()` or equivalent.

## CLI Contracts

### export-key Command

**Syntax**:
```
vibetea-monitor export-key [--path <PATH>]
```

**Arguments**:
| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `--path` | Path | `~/.vibetea` | Directory containing key.priv |

**Output** (stdout):
- Single line: Base64-encoded 32-byte private key seed
- Followed by single newline character
- No other text (enables piping)

**Errors** (stderr):
| Condition | Message | Exit Code |
|-----------|---------|-----------|
| Key file not found | "Error: No key found at {path}/key.priv\nRun 'vibetea-monitor init' first." | 1 |
| File read error | "Error: Could not read key file: {io_error}" | 2 |
| Invalid key format | "Error: Key file has invalid format" | 1 |

**Example Usage**:
```bash
# Copy to clipboard (macOS)
vibetea-monitor export-key | pbcopy

# Set as GitHub secret
gh secret set VIBETEA_PRIVATE_KEY < <(vibetea-monitor export-key)

# Custom path
vibetea-monitor export-key --path /custom/keys
```

### run Command (Modified)

**New Behavior**:
- Checks `VIBETEA_PRIVATE_KEY` first
- Falls back to file if env var not set
- Logs which source was used (INFO level)
- Logs public key fingerprint for verification

**New Log Messages**:
```
INFO Loading private key from environment variable
INFO Using private key from file: ~/.vibetea/key.priv
INFO Public key fingerprint: dGVzdHB1 (verify this matches server configuration)
INFO File-based key available but ignored (VIBETEA_PRIVATE_KEY takes precedence)
```

## Serialization Formats

### Private Key in Environment Variable

| Format | Example |
|--------|---------|
| Base64 Standard | `SGVsbG8gV29ybGQhIEFCQ0RFRkdISUpLTE1OT1A=` |

**Encoding**:
- Alphabet: `A-Za-z0-9+/`
- Padding: `=` (required)
- Length: 44 characters for 32-byte input

**Decoding**:
```rust
use base64::prelude::*;

let key_b64 = std::env::var("VIBETEA_PRIVATE_KEY")
    .map(|s| s.trim().to_string())?;
let seed_bytes = BASE64_STANDARD.decode(&key_b64)?;
if seed_bytes.len() != 32 {
    return Err(CryptoError::InvalidKey(
        format!("Expected 32 bytes, got {}", seed_bytes.len())
    ));
}
```

## Relationship to Existing Entities

### Crypto Module Extensions

```rust
impl Crypto {
    // Existing methods (unchanged)
    pub fn generate() -> Self;
    pub fn load(dir: &Path) -> Result<Self, CryptoError>;
    pub fn save(&self, dir: &Path) -> Result<(), CryptoError>;
    pub fn exists(dir: &Path) -> bool;
    pub fn sign(&self, message: &[u8]) -> String;
    pub fn public_key_base64(&self) -> String;

    // New methods
    pub fn load_from_env() -> Result<Option<Self>, CryptoError>;
    pub fn export_key_base64(&self) -> String;
    pub fn public_key_fingerprint(&self) -> String;
    pub fn load_with_fallback(dir: &Path) -> Result<(Self, KeySource), CryptoError>;
}
```

### CryptoError Extensions

```rust
pub enum CryptoError {
    // Existing variants
    Io(std::io::Error),
    InvalidKey(String),
    Base64(base64::DecodeError),
    KeyExists(String),

    // New variant (or reuse InvalidKey)
    // No new variants needed - InvalidKey covers env var errors
}
```
