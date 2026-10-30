#!/usr/bin/env bash
# Quit the running XRest, rebuild the Debug app, and launch it again.
#
#   macos/scripts/restart.sh            # build + relaunch
#   macos/scripts/restart.sh --no-build # just relaunch the last build
set -euo pipefail

cd "$(dirname "$0")/.."

derived="$HOME/Library/Developer/Xcode/DerivedData"
build=1
[[ "${1:-}" == "--no-build" ]] && build=0

if pgrep -x XRest >/dev/null; then
  osascript -e 'tell application "XRest" to quit' >/dev/null 2>&1 || true
  for _ in $(seq 1 20); do
    pgrep -x XRest >/dev/null || break
    sleep 0.25
  done
  pkill -x XRest 2>/dev/null || true
fi

if [[ $build == 1 ]]; then
  log=$(mktemp)
  if ! xcodebuild -project XRest.xcodeproj -scheme XRest -configuration Debug \
    -destination 'platform=macOS' -derivedDataPath "$derived/XRest-local" build >"$log" 2>&1; then
    grep -E "error:" "$log" | head -20
    echo "Build failed; full log: $log" >&2
    exit 1
  fi
  rm -f "$log"
fi

app="$derived/XRest-local/Build/Products/Debug/XRest.app"
[[ -d "$app" ]] || { echo "No build at $app; run without --no-build" >&2; exit 1; }
open "$app"
echo "Launched $app"
