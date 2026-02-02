# Technology Stack

**Status**: Greenfield project - stack defined, not yet implemented
**Last Updated**: 2026-02-02

## Languages

| Component | Language   | Version | Rationale                                           |
| --------- | ---------- | ------- | --------------------------------------------------- |
| Monitor   | Rust       | 2021    | Small binary, low memory, native file watching      |
| Server    | Rust       | 2021    | High performance async, single-binary deployment    |
| Client    | TypeScript | 5.x     | Type safety, React ecosystem compatibility          |

## Frameworks & Runtimes

### Rust (Monitor & Server)

| Framework/Library | Purpose                     | Notes                            |
| ----------------- | --------------------------- | -------------------------------- |
| tokio             | Async runtime               | Standard for async Rust          |
| axum              | HTTP/WebSocket server       | Well-integrated with tokio       |
| notify            | File watching               | Cross-platform (inotify/FSEvents)|
| ed25519-dalek     | Ed25519 signing/verification| Cryptographic authentication     |
| serde             | Serialization               | JSON handling                    |
| reqwest           | HTTP client                 | Monitor → Server communication   |
| thiserror/anyhow  | Error handling              | Per constitution standards       |

### TypeScript (Client)

| Framework/Library | Purpose          | Notes                          |
| ----------------- | ---------------- | ------------------------------ |
| React             | UI framework     | Functional components + hooks  |
| TypeScript        | Language         | Strict mode enabled            |
| Vite              | Build tool       | Fast dev server, optimized build|
| TBD               | State management | May not need external library  |

## Development Tools

### Rust

| Tool     | Purpose    | Configuration          |
| -------- | ---------- | ---------------------- |
| rustfmt  | Formatting | Default settings       |
| clippy   | Linting    | Default lints minimum  |
| cargo    | Build/test | Workspace for monorepo |

### TypeScript

| Tool     | Purpose    | Configuration           |
| -------- | ---------- | ----------------------- |
| Prettier | Formatting | Default + project rules |
| ESLint   | Linting    | Recommended + React     |
| Vitest   | Testing    | Fast, Vite-native       |

## Deployment

| Component | Target    | Format           | Notes                     |
| --------- | --------- | ---------------- | ------------------------- |
| Server    | Fly.io    | Docker container | Single Rust binary        |
| Client    | CDN       | Static files     | Netlify/Vercel/Cloudflare |
| Monitor   | Local     | Binary           | User downloads and runs   |

## Communication Protocols

| Interface          | Protocol       | Format |
| ------------------ | -------------- | ------ |
| Monitor → Server   | HTTPS POST     | JSON   |
| Server → Client    | WebSocket      | JSON   |
| Monitor auth       | Ed25519 sigs   | Base64 |
| Client auth        | Bearer token   | String |

## Not Yet Determined

- State management library for Client (may use React Context)
- Specific versions for all dependencies (to be pinned at implementation)
- CI/CD tooling (GitHub Actions likely)
