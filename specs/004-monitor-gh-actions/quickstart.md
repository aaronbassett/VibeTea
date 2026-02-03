# Quickstart: Monitor GitHub Actions Deployment

**Feature Branch**: `004-monitor-gh-actions`
**Date**: 2026-02-03

## Overview

This feature enables running the VibeTea monitor in GitHub Actions workflows to track Claude Code events during CI/CD operations (PR reviews, code generation, etc.).

## Prerequisites

- VibeTea monitor binary (built or downloaded)
- Existing keypair generated with `vibetea-monitor init`
- Access to a running VibeTea server
- GitHub repository with Actions enabled

## Quick Start

### 1. Export Your Existing Key

If you have an existing local keypair:

```bash
# Export your private key for CI use
vibetea-monitor export-key
# Output: SGVsbG8gV29ybGQhIEFCQ0RFRkdISUpLTE1OT1A= (example)

# Copy directly to clipboard (macOS)
vibetea-monitor export-key | pbcopy

# Copy directly to clipboard (Linux with xclip)
vibetea-monitor export-key | xclip -selection clipboard
```

### 2. Add GitHub Secrets

In your repository settings, add the following secrets:

| Secret Name | Value |
|-------------|-------|
| `VIBETEA_PRIVATE_KEY` | Output from `export-key` command |

And these repository variables:

| Variable Name | Value |
|---------------|-------|
| `VIBETEA_SERVER_URL` | Your VibeTea server URL (e.g., `https://vibetea.fly.dev`) |

### 3. Add to Your Workflow

```yaml
name: CI with VibeTea Monitoring

on:
  pull_request:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      # Start VibeTea monitor in background
      - name: Setup VibeTea Monitor
        env:
          VIBETEA_PRIVATE_KEY: ${{ secrets.VIBETEA_PRIVATE_KEY }}
          VIBETEA_SERVER_URL: ${{ vars.VIBETEA_SERVER_URL }}
          VIBETEA_SOURCE_ID: "github-${{ github.repository }}-${{ github.run_id }}"
        run: |
          curl -sSL https://github.com/org/vibetea/releases/latest/download/vibetea-monitor-linux-amd64 -o vibetea-monitor
          chmod +x vibetea-monitor
          ./vibetea-monitor run &

      # Your normal CI steps here
      - name: Build
        run: cargo build

      - name: Test
        run: cargo test
```

## Environment Variables

### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `VIBETEA_PRIVATE_KEY` | Base64-encoded Ed25519 private key seed | From `export-key` command |
| `VIBETEA_SERVER_URL` | VibeTea server URL | `https://vibetea.fly.dev` |

### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `VIBETEA_SOURCE_ID` | Hostname | Custom identifier for this monitor |
| `RUST_LOG` | `info` | Log level (debug, info, warn, error) |

## Verifying Setup

When the monitor starts, it logs the key source and fingerprint:

```
INFO Loading private key from environment variable
INFO Public key fingerprint: dGVzdHB1 (verify this matches server configuration)
INFO Starting VibeTea Monitor
```

Compare the fingerprint with your local key:
```bash
# Show your local public key
cat ~/.vibetea/key.pub
# First 8 characters should match the fingerprint in CI logs
```

## Development Setup

### Building the Monitor

```bash
# Clone and build
git clone https://github.com/org/vibetea.git
cd vibetea

# Build release binary
cargo build --release -p vibetea-monitor

# Binary at target/release/vibetea-monitor
```

### Running Tests

```bash
# Run all monitor tests
cargo test -p vibetea-monitor

# Run tests sequentially (required for env var tests)
cargo test -p vibetea-monitor -- --test-threads=1

# Run specific test module
cargo test -p vibetea-monitor env_key
cargo test -p vibetea-monitor export_key
```

### Local Testing with Environment Variable

```bash
# Export your existing key
export VIBETEA_PRIVATE_KEY=$(vibetea-monitor export-key)

# Run with env var (file key will be ignored)
VIBETEA_SERVER_URL=http://localhost:8080 vibetea-monitor run
```

## Troubleshooting

### "Invalid Base64" Error

```
Error: VIBETEA_PRIVATE_KEY contains invalid Base64
```

**Solution**: Ensure the secret contains only the base64 string, no extra whitespace or quotes.

### "Must be 32 bytes" Error

```
Error: VIBETEA_PRIVATE_KEY must decode to exactly 32 bytes, got N bytes
```

**Solution**: The key should be exactly 44 characters (32 bytes base64-encoded). Re-export with `export-key`.

### "401 Unauthorized" Error

```
ERROR Authentication failed: Invalid signature
```

**Solution**: The public key on the server doesn't match your private key. Verify:
1. The key fingerprint in CI matches your local key
2. The server's `VIBETEA_PUBLIC_KEYS` includes your public key

### Events Not Appearing

Check:
1. Monitor is running (`ps aux | grep vibetea`)
2. Server URL is correct and reachable
3. Network allows outbound HTTPS to server
4. Source ID matches what server expects (if filtering)

## Available Scripts

From the repository root:

```bash
# Build monitor
cargo build --release -p vibetea-monitor

# Run tests
cargo test -p vibetea-monitor

# Run with debug logging
RUST_LOG=debug cargo run -p vibetea-monitor -- run

# Generate new keypair
cargo run -p vibetea-monitor -- init

# Export existing key
cargo run -p vibetea-monitor -- export-key
```

## Next Steps

- [Spec](./spec.md) - Full feature specification
- [Data Model](./data-model.md) - Key entities and state transitions
- [Research](./research.md) - Design decisions and alternatives
