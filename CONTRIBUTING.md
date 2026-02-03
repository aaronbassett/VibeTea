# Contributing to VibeTea

Thank you for your interest in contributing to VibeTea! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Code Style](#code-style)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment. Please be considerate of others and focus on constructive collaboration.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/VibeTea.git
   cd VibeTea
   ```
3. Add the upstream repository as a remote:
   ```bash
   git remote add upstream https://github.com/aaronbassett/VibeTea.git
   ```

## Development Setup

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Node.js 20+** - Install via [nvm](https://github.com/nvm-sh/nvm) or [official installer](https://nodejs.org/)
- **pnpm** - Install via `npm install -g pnpm`

### Building the Project

#### Rust Components (Server and Monitor)

```bash
# Build all Rust packages
cargo build --workspace

# Build in release mode
cargo build --workspace --release
```

#### Client (React Dashboard)

```bash
cd client

# Install dependencies
pnpm install

# Start development server
pnpm dev

# Build for production
pnpm build
```

### Running Locally

1. **Start the Server:**
   ```bash
   export VIBETEA_AUTH_TOKEN="dev-token"
   cargo run --package vibetea-server
   ```

2. **Start the Monitor:**
   ```bash
   export VIBETEA_SERVER_URL="ws://localhost:3000/ws"
   export VIBETEA_AUTH_TOKEN="dev-token"
   cargo run --package vibetea-monitor
   ```

3. **Start the Client:**
   ```bash
   cd client
   pnpm dev
   ```

## Making Changes

1. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. Make your changes, following the [code style guidelines](#code-style)

3. Write or update tests as needed

4. Commit your changes with clear, descriptive messages:
   ```bash
   git commit -m "feat: add support for custom event filters"
   # or
   git commit -m "fix: resolve WebSocket reconnection issue"
   ```

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

## Code Style

### Rust

- Follow the official [Rust style guidelines](https://doc.rust-lang.org/nightly/style-guide/)
- Use `cargo fmt` to format code
- Use `cargo clippy` to catch common issues
- All warnings should be treated as errors (`-D warnings`)

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets -- -D warnings
```

### TypeScript/React

- Use TypeScript strict mode
- Follow the ESLint configuration in the project
- Use Prettier for formatting

```bash
cd client

# Lint code
pnpm lint

# Format code
pnpm format

# Type check
pnpm typecheck
```

### Security Patterns

When working with security-sensitive code:

- Use `subtle::ConstantTimeEq` for token comparison to prevent timing attacks
- Use `ed25519_dalek::VerifyingKey::verify_strict()` for RFC 8032 compliant signature verification

## Testing

### Rust Tests

**Important:** Rust tests that modify environment variables must run single-threaded:

```bash
# Run all tests (single-threaded for env var safety)
cargo test --workspace -- --test-threads=1

# Run tests for a specific package
cargo test --package vibetea-server -- --test-threads=1
cargo test --package vibetea-monitor -- --test-threads=1
```

The project uses the `EnvGuard` RAII pattern in `server/src/config.rs` to save/restore environment variables during tests.

### Client Tests

```bash
cd client

# Run tests once
pnpm test

# Run tests in watch mode
pnpm test:watch
```

### Writing Tests

- Write unit tests for new functionality
- Include integration tests for API endpoints
- Test error cases and edge conditions
- Aim for meaningful test coverage, not just high percentages

## Submitting Changes

1. **Ensure all tests pass:**
   ```bash
   cargo test --workspace -- --test-threads=1
   cd client && pnpm test
   ```

2. **Ensure code quality checks pass:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets -- -D warnings
   cd client && pnpm lint && pnpm typecheck
   ```

3. **Push your branch:**
   ```bash
   git push origin feature/your-feature-name
   ```

4. **Create a Pull Request:**
   - Go to the repository on GitHub
   - Click "New Pull Request"
   - Select your branch
   - Fill out the PR template with:
     - A clear description of the changes
     - Any related issue numbers
     - Testing steps

### Pull Request Guidelines

- Keep PRs focused on a single concern
- Update documentation if needed
- Add tests for new functionality
- Respond to review feedback promptly
- Squash commits if requested

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- System information (OS, Rust version, Node version)
- Relevant logs or error messages

### Feature Requests

When requesting features, please include:

- A clear description of the feature
- The problem it would solve
- Any alternative solutions you've considered

## Questions?

If you have questions about contributing:

1. Check existing issues and discussions
2. Open a new issue with the "question" label
3. Reach out to maintainers

Thank you for contributing to VibeTea!
