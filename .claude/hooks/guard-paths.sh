#!/usr/bin/env bash
# PreToolUse: block edits to generated or reference-only paths.
file="$(python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("file_path",""))')"
case "$file" in
  */legacy/*)                 msg="legacy/ is a read-only reference, do not edit it." ;;
  */macos/XRest.xcodeproj/*)  msg="XRest.xcodeproj is generated; edit macos/project.yml and run xcodegen." ;;
  */macos/XRest/Bindings/*)   msg="Bindings are generated; run macos/scripts/generate-bindings.sh." ;;
  *) exit 0 ;;
esac
echo "$msg" >&2
exit 2
