#!/usr/bin/env bash
# Stop: if core/ or the macOS scripts changed, require fmt, clippy and the e2e test to pass.
# Exit 2 sends the failure back to Claude so it fixes it before stopping.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../.."

active="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("stop_hook_active", False))' 2>/dev/null || echo False)"
[[ "$active" == "True" ]] && exit 0   # already retrying after a block; don't loop

changed="$(git status --porcelain)"
grep -qE ' (core|macos/scripts)/' <<<"$changed" || exit 0

run() {
  local name="$1"; shift
  local out
  if ! out="$("$@" 2>&1)"; then
    echo "Stop blocked: $name failed" >&2
    tail -n 60 <<<"$out" >&2
    exit 2
  fi
}

run "cargo fmt --check"  bash -c 'cd core && cargo fmt --check'
run "cargo clippy"       bash -c 'cd core && cargo clippy --workspace --all-targets -- -D warnings'
run "e2e test"           .claude/hooks/e2e.sh
if grep -q ' core/xrest-ffi/' <<<"$changed"; then
  run "swift bindings check" macos/scripts/generate-bindings.sh --check
fi
exit 0
