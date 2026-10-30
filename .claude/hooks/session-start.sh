#!/usr/bin/env bash
# Print the repo's ground rules at session start.
cat <<'MSG'
xrest rules:
- Anything that must behave the same on every platform belongs in core/; platform apps own only UI.
- Never put secrets in YAML. Never hand-edit macos/XRest/Bindings (run macos/scripts/generate-bindings.sh)
  or macos/XRest.xcodeproj (edit macos/project.yml). legacy/ is read-only reference.
- Finishing a session that touched core/ runs fmt, clippy and the e2e test (.claude/hooks/e2e.sh).
MSG
