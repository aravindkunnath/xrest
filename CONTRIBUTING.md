# Contributing to XRest

Thanks for wanting to contribute. This guide covers everything you need to get from clone to running code locally.

---

## Prerequisites

- **Node.js** v24+
- **Rust** (latest stable) — install via [rustup](https://www.rustup.rs/)
- **pnpm** — install via [pnpm.io](https://pnpm.io/installation)
- **Xcode Command Line Tools** (macOS) — `xcode-select --install`

---

## Local Setup

```bash
git clone https://github.com/aravindkunnath/xrest.git
cd xrest

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev
```

The first `pnpm tauri dev` will take a few minutes as cargo will compile the Rust backend.

---

## Project Structure

```
xrest/
├── src/                  # Vue 3 frontend
│   ├── components/       # UI components (shadcn-vue)
│   ├── composables/      # Vue composables for shared logic
│   ├── domains/          # Domain-specific logic and state (Collection, Service)
│   ├── infrastructure/   # Integration with Tauri commands and external APIs
│   ├── stores/           # Global Pinia state management
│   ├── views/            # Page-level components
│   ├── types/            # TypeScript type definitions
│   └── utils/            # Shared frontend utilities
├── src-tauri/            # Rust backend (Tauri)
│   ├── src/
│   │   ├── domains/      # Logic for core domains (Git, SQLite, etc.)
│   │   ├── commands.rs   # Tauri command bridge
│   │   ├── history.rs    # History management
│   │   ├── io.rs         # File I/O operations
│   │   ├── services.rs   # Service management
│   │   ├── types.rs      # Rust types and DTOs
│   │   └── main.rs       # App entry point
│   └── Cargo.toml        # Rust dependencies
├── tests/                # E2E and integration test assets
├── .github/workflows/    # CI/CD pipelines
├── package.json          # Frontend dependencies and scripts
├── vitest.config.ts      # Vitest configuration
└── pnpm-workspace.yaml   # pnpm workspace config
```

**Frontend** (Vue 3 + TypeScript) lives in `src/`. **Backend** (Rust) lives in `src-tauri/`. They communicate via Tauri's IPC — the Rust backend exposes commands, the Vue frontend invokes them.

---

## Common Commands

| Command | What it does |
|---|---|
| `pnpm install` | Install all dependencies |
| `pnpm tauri dev` | Run the app in dev mode (hot reload on frontend changes) |
| `pnpm tauri build` | Build production bundle |
| `pnpm test` | Run frontend unit/integration tests (Vitest) |
| `pnpm test:tauri` | Run all Rust backend tests |
| `pnpm test:tauri:unit` | Run Rust unit tests |
| `pnpm test:tauri:behavioral` | Run Rust behavioral tests |

---

## Running Tests

### Frontend Tests
```bash
pnpm test
```
Frontend tests use [Vitest](https://vitest.dev/). We use `jsdom` for component testing and `@vue/test-utils` for Vue-specific interactions.

### Backend Tests
```bash
pnpm test:tauri
```
Rust tests are handled via Cargo. We separate tests into `unit` (logic within modules) and `behavioral` (end-to-end Rust logic, often mocking external systems).

### E2E Tests
E2E tests using Selenium/Mocha are located in `tests/e2e`. These are used to verify the application's core flows in a real browser environment.

---

## How to Contribute

1. Fork the repo
2. Create a branch from `main` — use a descriptive name: `fix/prod-guardrail-timeout` or `feat/new-feature`
3. Make your changes
4. Run `pnpm test` — all tests should pass before you submit
5. Open a Pull Request against `main`

Keep PRs focused. One concern per PR. If you're unsure whether something is in scope, open an issue first.

---

## Good Places to Start

If you're new to the codebase, these are areas where contributions are most useful:

- **Tests** — expanding coverage, especially around service creation, environment switching, and preflight auth flows
- **Windows / Linux builds** — the app builds for these platforms but hasn't been tested. Filing bugs or fixes here will be highly appreciated.
- **Export formats** — curl, axios, Java, Kotlin exports exist but may have edge cases
- **Documentation** — inline code comments, especially in the Rust backend

Check the [Issues](https://github.com/aravindkunnath/xrest/issues) tab for labeled tasks.

---

## Reporting Bugs

Open an issue using the **Bug Report** template. Include:

- OS and version
- Steps to reproduce
- Expected vs actual behavior
- Screenshots if relevant

---

## Suggesting Features

Open an issue using the **Feature Request** template. Describe the problem you're hitting, not just the solution you have in mind.

---

## Code Style

- **Frontend**: Follow the existing Vue component patterns. TypeScript is required — no `.js` files. Use [Tailwind CSS v4](https://tailwindcss.com/blog/tailwindcss-v4-alpha) for styling.
- **Backend**: Run `cargo fmt` and `cargo clippy` before committing Rust code. Follow existing patterns in the `domains/` and `commands.rs` structure.
- **Config files**: Service definitions are YAML. Keep them human-readable.

---

## License

By contributing, you agree that your contributions will be licensed under the **MIT License**, the same license as the project.