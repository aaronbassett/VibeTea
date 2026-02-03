# CLI Command Contracts

**Feature Branch**: `004-monitor-gh-actions`
**Date**: 2026-02-03

## Commands

### export-key

Export the private key seed in base64 format for use with `VIBETEA_PRIVATE_KEY` environment variable.

#### Synopsis

```
vibetea-monitor export-key [OPTIONS]
```

#### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--path` | `-p` | PATH | `~/.vibetea` | Directory containing keypair |
| `--help` | `-h` | - | - | Show help message |

#### Output

**Success (exit code 0)**:
- stdout: `<base64-encoded-key>\n`
- stderr: (empty)

**Failure (exit code 1 - configuration error)**:
- stdout: (empty)
- stderr: Error message

**Failure (exit code 2 - runtime error)**:
- stdout: (empty)
- stderr: Error message

#### Examples

```bash
# Basic usage
$ vibetea-monitor export-key
SGVsbG8gV29ybGQhIEFCQ0RFRkdISUpLTE1OT1A=

# Custom path
$ vibetea-monitor export-key --path /custom/keys
dGVzdHByaXZhdGVrZXlzZWVkdGhhdGlzMzJi

# Error: no key found
$ vibetea-monitor export-key
Error: No key found at /home/user/.vibetea/key.priv
Run 'vibetea-monitor init' first.
$ echo $?
1

# Pipe to clipboard (macOS)
$ vibetea-monitor export-key | pbcopy

# Set as GitHub secret
$ gh secret set VIBETEA_PRIVATE_KEY < <(vibetea-monitor export-key)
```

#### Contract Tests

1. **export_key_outputs_valid_base64**: Output decodes to exactly 32 bytes
2. **export_key_only_newline_after**: Output ends with single `\n`, no extra content
3. **export_key_errors_to_stderr**: All error messages go to stderr, not stdout
4. **export_key_missing_file_exits_1**: Exit code 1 when key.priv doesn't exist
5. **export_key_custom_path_works**: `--path` option reads from specified directory
6. **export_key_roundtrip**: Exported key can be loaded via env var and produces valid signatures

---

### run (modified)

Run the monitor daemon with enhanced key loading.

#### Modified Behavior

1. Check `VIBETEA_PRIVATE_KEY` environment variable first
2. If present, load key from env var (trim whitespace, decode base64)
3. If absent, fall back to file-based key at `{VIBETEA_KEY_PATH}/key.priv`
4. Log which source was used (INFO level)
5. Log public key fingerprint for verification

#### New Log Messages

| Level | Condition | Message |
|-------|-----------|---------|
| INFO | Key from env | `Loading private key from environment variable` |
| INFO | Key from file | `Using private key from file: {path}` |
| INFO | Always | `Public key fingerprint: {first-8-chars}` |
| INFO | Both present | `File-based key available but ignored (VIBETEA_PRIVATE_KEY takes precedence)` |

#### Exit Codes

| Code | Meaning | Examples |
|------|---------|----------|
| 0 | Success | Normal operation |
| 1 | Configuration error | Invalid env var, missing key, invalid key format |
| 2 | Runtime error | I/O error reading file |

#### Environment Variables (Updated)

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `VIBETEA_SERVER_URL` | Yes | - | Server endpoint |
| `VIBETEA_PRIVATE_KEY` | No | - | Base64-encoded private key seed |
| `VIBETEA_KEY_PATH` | No | `~/.vibetea` | Key file directory (fallback) |
| `VIBETEA_SOURCE_ID` | No | hostname | Monitor identifier |
| `VIBETEA_CLAUDE_DIR` | No | `~/.claude` | Claude Code directory |
| `VIBETEA_BUFFER_SIZE` | No | 1000 | Event buffer capacity |
| `RUST_LOG` | No | info | Log level |

#### Examples

```bash
# With env var key
$ export VIBETEA_PRIVATE_KEY="SGVsbG8gV29ybGQh..."
$ vibetea-monitor run
INFO Loading private key from environment variable
INFO Public key fingerprint: dGVzdHB1
INFO Starting VibeTea Monitor

# Without env var (file fallback)
$ unset VIBETEA_PRIVATE_KEY
$ vibetea-monitor run
INFO Using private key from file: /home/user/.vibetea/key.priv
INFO Public key fingerprint: dGVzdHB1
INFO Starting VibeTea Monitor

# Invalid base64
$ export VIBETEA_PRIVATE_KEY="not-valid-base64!!"
$ vibetea-monitor run
Error: VIBETEA_PRIVATE_KEY contains invalid Base64: Invalid byte 33, offset 17
$ echo $?
1

# Wrong length
$ export VIBETEA_PRIVATE_KEY="dG9vLXNob3J0"
$ vibetea-monitor run
Error: VIBETEA_PRIVATE_KEY must decode to exactly 32 bytes, got 9 bytes
$ echo $?
1
```

#### Contract Tests

1. **run_env_key_takes_precedence**: With both env and file, env is used
2. **run_logs_key_source_env**: Logs "Loading private key from environment variable"
3. **run_logs_key_source_file**: Logs "Using private key from file:"
4. **run_logs_fingerprint**: Logs first 8 chars of public key base64
5. **run_invalid_base64_exits_1**: Invalid base64 in env var causes exit 1
6. **run_wrong_length_exits_1**: Decoded key not 32 bytes causes exit 1
7. **run_whitespace_trimmed**: Leading/trailing whitespace in env var is trimmed
8. **run_never_logs_key_value**: Private key value never appears in any log
