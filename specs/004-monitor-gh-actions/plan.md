# Implementation Plan: Monitor GitHub Actions Deployment

**Branch**: `004-monitor-gh-actions` | **Date**: 2026-02-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-monitor-gh-actions/spec.md`

## Summary

Enable VibeTea monitor deployment in GitHub Actions workflows by supporting private key loading from environment variables (via GitHub Secrets) and providing a key export command. This allows tracking Claude Code events during CI/CD workflows for PR reviews and code generation tasks.

## Technical Context

**Language/Version**: Rust 2021 edition (Monitor)
**Primary Dependencies**: `ed25519-dalek` 2.1, `base64` 0.22, `tokio` 1.43, `clap` (or manual CLI parsing)
**Storage**: N/A (keys from env vars or filesystem)
**Testing**: `cargo test` with `--test-threads=1` (env var tests), `tempfile` for key file tests
**Target Platform**: Linux GitHub Actions runners (x86_64)
**Project Type**: Multi-component (Monitor CLI extension)
**Performance Goals**: Key loading <10ms, startup ready immediately
**Constraints**: Non-blocking monitoring (workflow succeeds even if events fail to send)
**Scale/Scope**: Single command additions to existing Monitor CLI

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Privacy by Design | ✅ PASS | Private key material zeroed after SigningKey construction; never logged |
| II. Unix Philosophy | ✅ PASS | Single responsibility: key loading, key export, monitor operation |
| III. KISS/YAGNI | ✅ PASS | Minimal changes: env var key source + export command |
| IV. Event-Driven | ✅ PASS | No changes to event model |
| V. Test What Matters | ✅ PASS | Integration test: export → env load → sign → verify |
| VI. Fail Fast & Loud | ✅ PASS | Exit code 1 on auth failure (401), clear error messages |
| VII. Modularity | ✅ PASS | Key loading abstracted in crypto module |

## Project Structure

### Documentation (this feature)

```text
specs/004-monitor-gh-actions/
├── plan.md              # This file
├── research.md          # Phase 0 research (key formats, best practices)
├── data-model.md        # Phase 1 data model (key entity)
├── quickstart.md        # Phase 1 quickstart guide
├── contracts/           # Phase 1 CLI contracts
└── tasks.md             # Phase 2 task list
```

### Source Code (repository root)

```text
monitor/
├── src/
│   ├── main.rs          # CLI entry point (add export-key command)
│   ├── config.rs        # Config (add VIBETEA_PRIVATE_KEY env var)
│   ├── crypto.rs        # Key management (add load_from_env, export_key)
│   ├── sender.rs        # HTTP sender (unchanged)
│   └── ...
└── tests/
    ├── key_export_test.rs    # New: export → env load round-trip
    └── env_key_test.rs       # New: env var key loading tests

.github/
├── actions/
│   └── vibetea-monitor/
│       └── action.yml   # Composite action (P3 scope)
└── workflows/
    └── ci.yml           # Example workflow demonstrating monitor usage
```

**Structure Decision**: Extend existing Monitor structure with new crypto functionality and CLI command. GitHub Action in `.github/actions/` following standard composite action pattern.

## Complexity Tracking

No constitution violations requiring justification. Implementation follows existing patterns.
