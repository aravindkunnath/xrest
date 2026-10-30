# Contributing to XRest

XRest is a shared Rust core (`core/`) plus a native macOS SwiftUI app (`macos/`). The previous
Wails/Go/Vue app lives in `legacy/` as a reference and is not built any more.
Read [docs/architecture.md](docs/architecture.md) first; the key rule is that anything which must
behave identically across platforms belongs in the core, and all UI belongs in the platform app.

## Core

Prerequisite: a recent stable Rust toolchain.

```bash
cd core
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

If you change the public API of `xrest-ffi`, regenerate the committed Swift bindings:

```bash
macos/scripts/generate-bindings.sh
```

CI fails when the committed bindings are stale.

## macOS app

Prerequisites: macOS 14+, Xcode 15+, rustup, [XcodeGen](https://github.com/yonaskolb/XcodeGen).

```bash
macos/scripts/build-xcframework.sh
cd macos && xcodegen && open XRest.xcodeproj
```

Do not commit `XRest.xcodeproj`; it is generated from `macos/project.yml`.

## Commits

Use small, logical commits with imperative subjects.
