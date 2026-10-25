# Claude Code harness for xrest — design

Date: 2026-10-23 · Status: awaiting review

## Purpose

A project-level Claude Code harness that enforces the repo's rules while Claude works and refuses to
let a session finish with a broken core. The centrepiece is a core end-to-end test, driven only
through the `xrest-ffi` surface, that runs from the `Stop` hook.

## Scope

In: hook scripts, project `.claude/settings.json`, one new e2e test file.
Out: macOS `xcodebuild` / UI smoke tests, the full `cargo test --workspace` suite (CI owns it), and any
product feature work (E2, D2, B1, B4).

## Layout

```
.claude/
  settings.json          # hook wiring (committed)
  hooks/
    session-start.sh     # prints architecture rules
    guard-paths.sh       # PreToolUse: blocks generated / reference paths
    fmt-rust.sh          # PostToolUse: rustfmt the touched file
    stop-gate.sh         # Stop: fmt --check, clippy, e2e, bindings --check
    e2e.sh               # runs the e2e test; shared by the Stop hook and humans
core/xrest-ffi/tests/e2e.rs
```

Scripts are bash with `set -euo pipefail`, read the hook JSON from stdin with `python3 -c` (no `jq`
dependency), and resolve the repo root from `CLAUDE_PROJECT_DIR`, falling back to git.

## Hooks

| Event | Matcher | Behaviour |
|---|---|---|
| `SessionStart` | all | Print the architecture rules from `docs/architecture.md`'s key points: shared behaviour belongs in the core, UI only in the app, no secrets in YAML, never hand-edit generated files. |
| `PreToolUse` | `Edit\|Write\|MultiEdit` | Exit 2 with a reason when the target path is under `legacy/`, is `macos/XRest.xcodeproj/**`, or is `macos/XRest/Bindings/**`. Message names the right action (read-only reference; regenerate with `macos/scripts/generate-bindings.sh`; edit `macos/project.yml` and run xcodegen). |
| `PostToolUse` | `Edit\|Write\|MultiEdit` | If the file is `core/**/*.rs`, run `rustfmt --edition 2021` on that file only. Never fails the tool call. |
| `Stop` | all | See below. |

## Stop gate

1. If `stop_hook_active` is true in the input, exit 0. This prevents an infinite block loop.
2. Compute changed paths (`git status --porcelain` plus diff against `HEAD`). If nothing under `core/`
   or `macos/scripts/` changed, exit 0.
3. Run, in order, stopping at the first failure:
   1. `cargo fmt --check`
   2. `cargo clippy --workspace --all-targets -- -D warnings`
   3. `e2e.sh`
   4. `macos/scripts/generate-bindings.sh --check`, only if `core/xrest-ffi/` changed
4. On failure, print the trimmed tail (last 60 lines) of the failing step to stderr and exit 2, so
   Claude must fix it before stopping. On success, exit 0 silently.
5. Each step runs under `timeout` (clippy and e2e: 600 s). A timeout counts as a failure with its
   own message.

The gate is cheap when nothing relevant changed and warm-cache fast otherwise (cargo's incremental
build; the e2e uses an in-process wiremock server, no external services).

## E2E test (`core/xrest-ffi/tests/e2e.rs`)

Uses only `xrest_ffi::*` (the same surface Swift sees), a tempdir, and `wiremock`. One scenario per
test function, each independent:

1. **Unsafe guard.** Workspace with `DEV` and an `isUnsafe` `PROD`. `send_policy()` reports
   `requires_confirmation` only for `PROD`. An unconfirmed send to `PROD` returns
   `ConfirmationRequired` and the mock records zero requests, preflight included. A confirmed send
   succeeds.
2. **Variables and secrets.** `{{BASE_URL}}` resolves per environment. A `{{secret.KEY}}` with no
   value fails with `MissingSecrets` naming exactly the missing keys before any request. After
   `set_secret`, the send succeeds and the secret value is absent from history and code export.
3. **Preflight.** A service preflight fetches a token, sets `Authorization: Bearer …`, and is
   cached: two sends cause one token request. Switching environment does not reuse the cached
   token. `clear_token_cache` forces a refetch.
4. **Persistence and history.** Create, update and delete endpoints; reopen the workspace from disk
   and see the same endpoints and version history; `restore_endpoint_version` round-trips; sends
   are queryable through the history store.

Tests that duplicate coverage already in `xrest-ffi/tests/ffi.rs` are not copied. The e2e adds the
cross-feature, reopen-from-disk flow that the unit-style FFI tests do not.

## Error handling and safety

- Hooks never read or print file contents beyond the tool path; they act on paths only.
- A hook script bug must not wedge a session: every script traps errors and exits 0 for hooks that are
  advisory (`SessionStart`, `PostToolUse`); only `PreToolUse` and `Stop` block, deliberately.
- `settings.json` sets `timeout` on every hook (`Stop`: 900 s).

## Verification

- Each hook is exercised by piping a sample JSON payload into it and checking exit code and message.
- The Stop gate is checked for: no-op when clean, block on an introduced fmt error, block on a
  failing e2e assertion, pass when green, and no loop when `stop_hook_active` is true.
- The e2e is checked red-first by temporarily breaking the unsafe guard, then green.
- `cargo fmt --check`, `clippy -D warnings`, and the e2e all pass on the final tree.

## Open items

None. Decisions taken: the bindings check runs only when `xrest-ffi` changed; the full workspace test
suite stays in CI.
