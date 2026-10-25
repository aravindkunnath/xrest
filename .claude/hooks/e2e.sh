#!/usr/bin/env bash
# Core end-to-end test through the FFI surface. Used by the Stop hook; also runnable by hand.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/../../core"
exec cargo test -p xrest-ffi --test e2e
