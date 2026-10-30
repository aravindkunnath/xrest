#!/usr/bin/env bash
# PostToolUse: rustfmt a Rust file under core/ after it is edited. Never blocks.
file="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))')"
case "$file" in
  */core/*.rs) rustfmt --edition 2021 "$file" >/dev/null 2>&1 || true ;;
esac
exit 0
