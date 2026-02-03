<!--
==============================================================================
SYNC IMPACT REPORT
==============================================================================
Version change: N/A → 1.0.0 (initial sync from project CONSTITUTION.md)
Date: 2026-02-02

Modified principles: N/A (initial creation)
Added sections: All (synced from existing CONSTITUTION.md)
Removed sections: None

Templates requiring updates:
  ✅ plan-template.md - Constitution Check section compatible
  ✅ spec-template.md - Requirements align with privacy-first approach
  ✅ tasks-template.md - Test phases align with "Test What Matters" principle

Follow-up TODOs: None - all placeholders resolved

Source: /home/ubuntu/Projects/VibeTea/CONSTITUTION.md (authoritative)
==============================================================================
-->

# VibeTea Constitution

## Core Principles

### I. Privacy by Design (NON-NEGOTIABLE)

User privacy and code confidentiality are paramount. Developers using VibeTea may be working on proprietary, sensitive, or confidential code.

**Rules:**
- Never log, track, store, or transmit source code content
- Never log file contents, code snippets, or agent outputs
- Never include code in error messages, crash reports, or telemetry
- Sanitize all logs to exclude potentially sensitive information
- When in doubt, don't log it
- No analytics or telemetry that could reveal what users are working on
- All data stays local unless user explicitly chooses otherwise

**Rationale:** Developers trust VibeTea with their work. Violating that trust, even accidentally, destroys the project's value proposition.

**Enforcement:**
- Code reviews MUST verify no sensitive data in logs
- Tests MUST explicitly verify nothing sensitive is logged
- CI/CD pipelines MUST include log sanitization checks

### II. Unix Philosophy

Each component does one thing well. Components communicate through well-defined interfaces.

**Rules:**
- Monitor monitors, Server serves, Client displays - clear separation
- Text-based protocols between components where practical
- Prefer composition over monolithic features
- Each component should be independently testable and deployable
- Predictable, documented interfaces between components
- Avoid hidden dependencies between components

**Rationale:** A multi-component system stays maintainable when boundaries are clear. Unix philosophy has proven this for 50+ years.

**Enforcement:**
- Architecture reviews verify single responsibility
- Integration tests prove components work independently

### III. Keep It Simple (KISS/YAGNI)

Build the simplest thing that works. Add complexity only when proven necessary.

**Rules:**
- No speculative features - build when needed, not "just in case"
- Prefer boring, well-understood solutions over clever ones
- If you can't explain a design decision in one sentence, reconsider it
- Premature abstraction is as harmful as premature optimization
- Rule of Three: Don't generalize until the third repetition
- Refactor when it hurts, not before

**Rationale:** Solo-maintained open source projects die from complexity. Every abstraction is maintenance burden. Ship features, not frameworks.

**Enforcement:**
- Complexity additions require justification in PR description
- New abstractions require documented third repetition

### IV. Event-Driven Communication

Components communicate asynchronously through events and messages, not direct coupling.

**Rules:**
- Monitor emits events; it doesn't call server methods directly
- Server processes events and maintains state
- Client subscribes to state changes; it doesn't poll
- Events are the source of truth for what happened
- Components can be restarted independently without breaking others
- Design for eventual consistency where appropriate

**Rationale:** Event-driven architecture enables loose coupling, easier testing, and graceful degradation when components restart or fail.

**Enforcement:**
- Architecture reviews verify event-driven patterns
- No direct cross-component method calls allowed

### V. Test What Matters

Comprehensive testing focused on catching real bugs, not achieving coverage metrics.

**Rules:**
- Test critical paths and user-facing workflows thoroughly
- Integration tests prove components work together correctly
- Unit tests for complex logic and edge cases
- Don't test framework behavior or trivial code
- Tests must be deterministic - no flaky tests allowed
- Test privacy guarantees explicitly (verify nothing sensitive is logged)

**Rationale:** Comprehensive testing catches regressions and enables confident refactoring. But testing trivialities wastes time and creates maintenance burden.

**Enforcement:**
- Tests required for critical paths
- Flaky tests must be fixed or removed within 24 hours

### VI. Fail Fast & Loud

When something goes wrong, fail immediately with clear, actionable information.

**Rules:**
- Crash early with context rather than limping along in bad state
- Error messages explain what went wrong and suggest fixes
- No silent failures - if something fails, the user knows
- Errors go to stderr, success output to stdout (for CLI components)
- Log errors at appropriate levels (but never log sensitive data - see Principle I)
- Graceful degradation: if a component is down, say so clearly

**Rationale:** Debugging silent failures wastes hours. Clear errors save time and reduce frustration.

**Enforcement:**
- Error handling review in code reviews
- User-facing errors must include remediation suggestions

### VII. Modularity & Clear Boundaries

Well-defined boundaries between components with explicit dependencies.

**Rules:**
- Each component has a single, clear responsibility
- Dependencies between components are explicit and documented
- No circular dependencies
- Shared code lives in shared modules, not duplicated
- Changes to one component shouldn't require changes to others (unless interfaces change)
- Public APIs are stable; internal implementation can change freely

**Rationale:** Clear boundaries enable independent development, testing, and evolution of components. This is essential for a multi-language codebase.

**Enforcement:**
- Dependency graphs reviewed quarterly
- Interface changes require migration plan

## Multi-Language Standards

### Rust (Monitor & Server)

- Follow Rust 2021 edition idioms
- Use `clippy` with default lints at minimum
- Format with `rustfmt`
- Prefer `Result` over panics for recoverable errors
- Use `thiserror` for library errors, `anyhow` for application errors
- Async runtime: document which one and why

### TypeScript/React (Client)

- Strict TypeScript (`strict: true` in tsconfig)
- Functional components with hooks
- Format with Prettier, lint with ESLint
- Type all public interfaces - no `any` except when truly necessary
- Prefer composition over prop drilling

### Cross-Language

- JSON for data exchange between components
- Semantic versioning for all components
- Consistent naming: if it's called "session" in Rust, call it "session" in TypeScript
- Shared types documented in one place

## Open Source Standards

### Versioning

- Semantic Versioning (MAJOR.MINOR.PATCH)
- MAJOR: Breaking changes to public APIs or behavior
- MINOR: New features, backward compatible
- PATCH: Bug fixes, backward compatible
- Pre-1.0: API may change between minor versions

### Commits & History

- Conventional Commits format: `type(scope): subject`
- Types: feat, fix, docs, refactor, test, chore
- Scope: monitor, server, client, or omit for cross-cutting
- Keep commits atomic and focused

### Documentation

- README: Installation, quick start, basic usage
- CONTRIBUTING: How to set up dev environment, submit changes
- Code comments explain "why", not "what"
- Public APIs documented with examples

## Governance

This constitution defines the principles that guide VibeTea's development. All contributions should align with these principles.

**Amendment Process:**
1. Propose changes via GitHub issue or PR
2. Document rationale for the change
3. Update version and changelog
4. Ensure dependent practices are updated

**Compliance:**
- PRs should be reviewed against these principles
- When principles conflict, Privacy by Design (I) takes precedence
- Complexity must be justified against KISS (III)
- When in doubt, refer to the rationale for each principle

**Version**: 1.0.0 | **Ratified**: 2026-02-02 | **Last Amended**: 2026-02-02

---

## Implementation Checklist

Before considering work complete, verify:

- [ ] **Privacy verified**: No source code, file contents, or sensitive data in logs, errors, or telemetry
- [ ] **Single responsibility**: Each component/function does one thing well with documented interfaces
- [ ] **Simplicity justified**: No speculative features; complexity additions explained in PR description
- [ ] **Event-driven**: Components communicate via events, not direct method calls; no polling
- [ ] **Tests meaningful**: Critical paths tested; integration tests for component interaction; no flaky tests
- [ ] **Errors actionable**: All errors logged with context and remediation suggestions; no silent failures
- [ ] **No circular deps**: Dependencies explicit and documented; shared code in shared modules
- [ ] **Language standards**: Rust uses clippy/rustfmt; TypeScript is strict with no `any`; Prettier/ESLint pass
- [ ] **Conventional commits**: Format `type(scope): subject` with atomic, focused changes
- [ ] **Privacy test exists**: Explicit test verifying nothing sensitive is logged (Principle I compliance)
