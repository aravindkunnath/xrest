# Contributing to XRest

Thanks for wanting to contribute. This guide covers everything you need to get from clone to running code locally.

---

## Prerequisites

- **Go** v1.25+
- **Node.js** v18+
- **pnpm** — install via [pnpm.io](https://pnpm.io/installation)
- [Wails3](https://v3.wails.io/) CLI
- [go-task](https://taskfile.dev/) — optional; lets you run tasks as a bare `task` command. Without it, use `wails3 task` instead (Wails3 ships a Taskfile runner).
- **Xcode Command Line Tools** (macOS) — `xcode-select --install`

---

## Local Setup

```bash
git clone https://github.com/aravindkunnath/xrest.git
cd xrest

# Install frontend dependencies
pnpm --filter frontend install

# Run in development mode (either works — both run the same Taskfile.yml)
task dev
# ...or
wails3 task dev
```

You can also run `wails3 dev` directly (see the README for details). The first dev build will take a few minutes.

---

## Common Commands

All task commands can be run as either `task <name>` (go-task) or `wails3 task <name>`.

| Command | What it does |
| --- | --- |
| `dev` | Run the app in dev mode (hot reload on frontend changes) |
| `build` | Build production bundle |
| `build:server` | Build server-only mode (HTTP server, no GUI) |
| `test` | Run both backend and frontend unit tests |
| `test:backend` | Run Go backend unit tests |
| `test:frontend` | Run frontend unit/component tests (Vitest) |
| `test:coverage` | Run all tests and generate a coverage HTML report |
| `test:e2e` | Run Playwright E2E tests (starts server mode automatically) |
| `package:dmg` | Package a `.dmg` installer (macOS) |
| `build:cli` / `run:cli` | Build / run the CLI client |

---

## Running Tests

### Frontend Tests

```bash
task test:frontend   # or: wails3 task test:frontend
```

Frontend tests use [Vitest](https://vitest.dev/). We use `jsdom` for component testing and `@vue/test-utils` for Vue-specific interactions.

### Backend Tests

```bash
task test:backend   # or: wails3 task test:backend
```

Go tests run via `go test ./...`. Tests are colocated with the code they exercise (e.g. `internal/adapters/*_test.go`, `cmd/wails/*_test.go`).

### E2E Tests

E2E tests use [Playwright](https://playwright.dev/) and live in `frontend/tests`. They verify the application's core flows in a real browser environment against an automatically started server mode instance.

---

## How to Contribute

1. Fork the repo
2. Create a branch from `main` — use a descriptive name: `fix/prod-guardrail-timeout` or `feat/new-feature`
3. Make your changes
4. Run `task test` (or `wails3 task test`) — all tests should pass before you submit
5. Open a Pull Request against `main`

Keep PRs focused. One concern per PR. If you're unsure whether something is in scope, open an issue first.

---

## Good Places to Start

If you're new to the codebase, these are areas where contributions are most useful:

- **Tests** — expanding coverage, especially around service creation, environment switching, and preflight auth flows
- **Windows / Linux builds** — the app builds for these platforms but hasn't been tested. Filing bugs or fixes here will be highly appreciated.
- **Import formats** — OpenAPI, Swagger, and curl importers exist but may have edge cases
- **Documentation** — inline code comments, especially in the Go backend

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

- **Frontend**: Follow the existing Vue component patterns. TypeScript is required — no `.js` files. Use Tailwind CSS utility classes within the established component style.
- **Backend**: Run `go fmt` before committing Go code. Follow existing patterns in `internal/` and the `cmd/wails` gateway structure.
- **Config files**: Service definitions are YAML. Keep them human-readable.

A pre-commit hook runs `lint-staged` (type-checking `*.ts`/`*.vue` via `vue-tsc`) before every commit, so make sure your frontend changes type-check.

---

## License

By contributing, you agree that your contributions will be licensed under the **MIT License**, the same license as the project.