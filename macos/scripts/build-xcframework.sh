#!/usr/bin/env bash
# Build XRestCore.xcframework: the Rust core as a universal (arm64 + x86_64) macOS static library,
# with the UniFFI C header and module map, and refresh the Swift bindings.
#
# Requirements (macOS only): Xcode command line tools, rustup/cargo.
# Output: macos/Frameworks/XRestCore.xcframework   (gitignored)
#
# Usage: macos/scripts/build-xcframework.sh [--debug]
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: build-xcframework.sh only runs on macOS (needs lipo and xcodebuild)." >&2
  exit 1
fi
for tool in cargo rustup lipo xcodebuild; do
  command -v "$tool" >/dev/null || { echo "error: '$tool' not found in PATH" >&2; exit 1; }
done

PROFILE="release"
PROFILE_DIR="release"
CARGO_PROFILE_FLAG="--release"
if [[ "${1:-}" == "--debug" ]]; then
  PROFILE="debug"
  PROFILE_DIR="debug"
  CARGO_PROFILE_FLAG=""
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CORE_DIR="$REPO_ROOT/core"
OUT_DIR="$REPO_ROOT/macos/Frameworks"
XCFRAMEWORK="$OUT_DIR/XRestCore.xcframework"
STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

# Keep in sync with deploymentTarget in macos/project.yml.
export MACOSX_DEPLOYMENT_TARGET="14.0"

TARGETS=(aarch64-apple-darwin x86_64-apple-darwin)

echo "==> Ensuring Rust targets are installed"
rustup target add "${TARGETS[@]}"

TARGET_DIR="$(cargo metadata --manifest-path "$CORE_DIR/Cargo.toml" --format-version 1 --no-deps |
  python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')"

LIBS=()
for target in "${TARGETS[@]}"; do
  echo "==> Building xrest-ffi ($PROFILE) for $target"
  # shellcheck disable=SC2086
  cargo build --manifest-path "$CORE_DIR/Cargo.toml" -p xrest-ffi --target "$target" $CARGO_PROFILE_FLAG
  LIBS+=("$TARGET_DIR/$target/$PROFILE_DIR/libxrest_ffi.a")
done

echo "==> Creating universal static library"
mkdir -p "$STAGE/lib"
lipo -create "${LIBS[@]}" -output "$STAGE/lib/libxrest_ffi.a"
lipo -info "$STAGE/lib/libxrest_ffi.a"

echo "==> Regenerating Swift bindings and headers"
HEADERS_DIR="$STAGE/headers" "$SCRIPT_DIR/generate-bindings.sh"

echo "==> Creating $XCFRAMEWORK"
rm -rf "$XCFRAMEWORK"
mkdir -p "$OUT_DIR"
xcodebuild -create-xcframework \
  -library "$STAGE/lib/libxrest_ffi.a" \
  -headers "$STAGE/headers" \
  -output "$XCFRAMEWORK"

echo "==> Done: $XCFRAMEWORK"
echo "    Next: cd macos && xcodegen && open XRest.xcodeproj"
