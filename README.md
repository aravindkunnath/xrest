# XRest
**The Service‑First REST Client for Microservices.**

XRest organizes APIs as versioned "services" with built-in environment management, safe/unsafe execution controls, and optional auth pre‑flight, making microservice workflows repeatable, shareable, and safe.

> **Status:** XRest is being rebuilt as a shared Rust core plus native apps (macOS first). The previous Wails/Go/Vue app is preserved in [`legacy/`](legacy/) as a porting reference. Not ready for production use.

---

## Key features

- **One service per workspace.** A workspace is a service directory: its versioned **endpoints**, its **environments**, and a **scratchpad** for loose requests.
- **Git‑native.** Services, environments and endpoints are human-readable YAML. Sync with git; the scratchpad is gitignored by default.
- **Guardrails for production.** Mark environments (like `PROD`) as **unsafe**. The core refuses to send to them without an explicit confirmation, on every platform, and the app shows red cues.
- **Pre‑flight auth.** Define a token request once; XRest acquires, caches (per environment) and injects the token.
- **Endpoint versioning.** Every change to an endpoint's request is snapshotted.
- **Scratchpad.** Try things without committing them. Promote a scratch request into a versioned endpoint when it's worth keeping.
- **Local-first.** Your URLs, keys and payloads never leave your machine or your git repo.

## Architecture

Ghostty-style: the **core** owns everything that must behave identically across platforms; each **platform app** owns all UI.

```text
core/                    Cargo workspace
  xrest-core/            Platform-agnostic library: workspace model, YAML, interpolation,
                         environments + unsafe policy, auth, preflight, execution, versioning, cURL import
  xrest-ffi/             UniFFI bindings (staticlib + cdylib); async send maps to Swift async
  uniffi-bindgen/        Pinned binding generator
macos/                   SwiftUI app (macOS 14+), XcodeGen project, build scripts, generated bindings
legacy/                  Previous Wails v3 + Go + Vue app (reference only)
docs/architecture.md     Core/app boundary, on-disk format, FFI API
```

See [docs/architecture.md](docs/architecture.md) for details.

## Workspace format

```text
my-service/
├── service.yaml        # formatVersion, name, service-level auth, service preflight, appearance
├── environments.yaml   # DEV / STAGE / PROD: variables, color, unsafe rules
├── variables.yaml      # variables shared by every environment
├── endpoints/          # versioned requests, one YAML file each
├── scratchpad/         # loose requests (gitignored)
├── .xrest/             # per-user: selected environment, your variable overrides, version history (gitignored)
└── .gitignore          # created by XRest; ignores /scratchpad/ and /.xrest/
```

Variables come from layers, most specific first: your local override, the environment, the
workspace (every environment), your global variables, then built-ins such as `{{$uuid}}`. An
unsafe environment can confirm every request or only those that change data, and can set a
minimum countdown for everyone.

Request bodies are typed (text, JSON, form, multipart, file) and each request can override the
timeout, redirects, TLS verification and proxy. Files a request uploads must live inside the
workspace folder. Details: [docs/architecture.md](docs/architecture.md).

## Building

### Core (Linux, macOS)

Requires a recent stable Rust toolchain.

```bash
cd core
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Swift bindings

Regenerate `macos/XRest/Bindings/xrest_ffi.swift` after any change to the `xrest-ffi` API (works on Linux too):

```bash
macos/scripts/generate-bindings.sh          # regenerate
macos/scripts/generate-bindings.sh --check  # fail if stale (CI)
```

### macOS app

Requires macOS 14+, Xcode 15+, Rust (with `rustup`) and [XcodeGen](https://github.com/yonaskolb/XcodeGen).

```bash
macos/scripts/build-xcframework.sh   # universal static lib -> macos/Frameworks/XRestCore.xcframework
cd macos && xcodegen                 # generates XRest.xcodeproj from project.yml
open XRest.xcodeproj                 # build & run the XRest scheme
```

> The SwiftUI sources were written without access to a Swift toolchain and have not been compiled yet; expect to fix a few compile errors on the first build.

### Legacy app

See [`legacy/`](legacy/) (Go + Wails v3 + Vue). It is no longer built in CI.

## What is not done yet

| Area | State |
| --- | --- |
| Request history (SQLite) | Stubbed: `xrest_core::history`, TODO |
| Git integration | Stubbed: `xrest_core::git`, TODO |
| Swagger / OpenAPI import | Stubbed: `xrest_core::swagger`, TODO |
| Secrets in the OS keychain | `{{secret.KEY}}` works through a `SecretStore` trait; only an in-memory store exists |
| Editing environments, service auth, preflight in the macOS UI | Edit the YAML files for now |
| macOS release pipeline, App Sandbox, GTK app | Not started |

*XRest is in active development. Use at your own risk.*
