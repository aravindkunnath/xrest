#!/usr/bin/env bash
# Regenerate the Swift bindings for xrest-ffi with UniFFI (library mode).
#
# Works on macOS and Linux: the bindings are derived from metadata embedded in the host-built
# cdylib, so no Swift toolchain is needed. Output:
#   macos/XRest/Bindings/xrest_ffi.swift          (committed)
#   $HEADERS_DIR/{xrest_ffiFFI.h,module.modulemap} (only if HEADERS_DIR is set; used by build-xcframework.sh)
#
# Usage: macos/scripts/generate-bindings.sh [--check]
#   --check   fail if the committed bindings are out of date (for CI)
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CORE_DIR="$REPO_ROOT/core"
SWIFT_OUT="$REPO_ROOT/macos/XRest/Bindings"
CHECK=0
[[ "${1:-}" == "--check" ]] && CHECK=1

echo "==> Building xrest-ffi (host, debug) for binding metadata"
cargo build --manifest-path "$CORE_DIR/Cargo.toml" -p xrest-ffi

TARGET_DIR="$(cargo metadata --manifest-path "$CORE_DIR/Cargo.toml" --format-version 1 --no-deps |
  python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"
LIB=""
for candidate in "$TARGET_DIR/debug/libxrest_ffi.dylib" "$TARGET_DIR/debug/libxrest_ffi.so"; do
  [[ -f "$candidate" ]] && LIB="$candidate" && break
done
[[ -n "$LIB" ]] || { echo "error: cdylib not found in $TARGET_DIR/debug" >&2; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> Generating Swift bindings from $LIB"
# uniffi-bindgen runs `cargo metadata` in the current directory, so run from the workspace.
(cd "$CORE_DIR" && cargo run --quiet -p uniffi-bindgen -- \
  generate "$LIB" --language swift --no-format --out-dir "$TMP")

if [[ $CHECK -eq 1 ]]; then
  if ! diff -u "$SWIFT_OUT/xrest_ffi.swift" "$TMP/xrest_ffi.swift"; then
    echo "error: committed Swift bindings are stale; run macos/scripts/generate-bindings.sh" >&2
    exit 1
  fi
  echo "Bindings are up to date."
  exit 0
fi

mkdir -p "$SWIFT_OUT"
cp "$TMP/xrest_ffi.swift" "$SWIFT_OUT/xrest_ffi.swift"
echo "==> Wrote $SWIFT_OUT/xrest_ffi.swift"

if [[ -n "${HEADERS_DIR:-}" ]]; then
  mkdir -p "$HEADERS_DIR"
  cp "$TMP/xrest_ffiFFI.h" "$HEADERS_DIR/xrest_ffiFFI.h"
  # Xcode/SwiftPM expect the module map to be called module.modulemap.
  cp "$TMP/xrest_ffiFFI.modulemap" "$HEADERS_DIR/module.modulemap"
  echo "==> Wrote headers to $HEADERS_DIR"
fi
