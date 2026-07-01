#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${1:-"$ROOT/target/binary-size-reports/$STAMP-duplicate-deps"}"
PACKAGE="${PACKAGE:-rspack_node}"

mkdir -p "$OUT_DIR"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required for duplicate dependency statistics" >&2
  exit 1
fi

cargo tree -p "$PACKAGE" -d > "$OUT_DIR/cargo-tree-duplicates.txt" 2>"$OUT_DIR/cargo-tree-duplicates.stderr.txt" || true
cargo tree -p "$PACKAGE" -e features > "$OUT_DIR/cargo-tree-features.txt" 2>"$OUT_DIR/cargo-tree-features.stderr.txt" || true
cargo metadata --format-version 1 --locked > "$OUT_DIR/cargo-metadata.json" 2>"$OUT_DIR/cargo-metadata.stderr.txt" || true

if command -v jq >/dev/null 2>&1 && [[ -s "$OUT_DIR/cargo-metadata.json" ]]; then
  jq -r '.packages[] | [.name, .version, (.source // "path")] | @tsv' "$OUT_DIR/cargo-metadata.json" \
    | sort \
    > "$OUT_DIR/packages.tsv"

  awk -F'\t' '
    {
      versions[$1][$2] = 1
    }
    END {
      for (name in versions) {
        n = 0
        line = name
        for (version in versions[name]) {
          n++
          line = line "\t" version
        }
        if (n > 1) {
          print line
        }
      }
    }
  ' "$OUT_DIR/packages.tsv" | sort > "$OUT_DIR/duplicate-package-versions.tsv"

  jq -r '
    .resolve.nodes[]
    | .features[]? as $feature
    | [.id, $feature] | @tsv
  ' "$OUT_DIR/cargo-metadata.json" \
    | sort \
    > "$OUT_DIR/resolved-features.tsv" || true
else
  cat > "$OUT_DIR/jq.skipped.txt" <<'EOF'
jq was not found or cargo metadata failed.
Use cargo-tree reports in this directory, or install jq for duplicate-package-versions.tsv.
EOF
fi

{
  echo "# Duplicate dependency report"
  echo "package: $PACKAGE"
  echo
  echo "# cargo tree duplicate summary"
  sed -n '1,160p' "$OUT_DIR/cargo-tree-duplicates.txt" || true
  echo
  echo "# Common large dependency feature markers"
  for pattern in napi napi-derive napi-sys tokio serde serde_json swc_core lightningcss rspack_plugin rspack_loader rspack_tracing; do
    printf '%s\t' "$pattern"
    grep -F "$pattern" "$OUT_DIR/cargo-tree-features.txt" 2>/dev/null | wc -l | tr -d ' '
  done
} > "$OUT_DIR/summary.txt"

echo "duplicate dependency report: $OUT_DIR"
