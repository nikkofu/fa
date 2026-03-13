#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SANDBOX_ROOT="${FA_SANDBOX_DIR:-$ROOT_DIR/sandbox}"

mkdir -p "$SANDBOX_ROOT"

(
  cd "$ROOT_DIR"
  FA_SANDBOX_DIR="$SANDBOX_ROOT" cargo test -p fa-server sandbox_safe_ -- --nocapture
)

echo "v0.2.0 sandbox smoke succeeded using sandbox dir $SANDBOX_ROOT"
