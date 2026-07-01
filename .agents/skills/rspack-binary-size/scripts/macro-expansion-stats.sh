#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${1:-"$ROOT/target/binary-size-reports/$STAMP-macro-expansion"}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_BIN="${PYTHON:-python3}"

mkdir -p "$OUT_DIR"
cd "$ROOT"

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "error: python3 is required for macro expansion statistics" >&2
  exit 1
fi

"$PYTHON_BIN" "$SCRIPT_DIR/macro-expansion-stats.py" --root "$ROOT" --out "$OUT_DIR"

echo "macro expansion report: $OUT_DIR"
