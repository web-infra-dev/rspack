#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
STAMP="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="${OUT_DIR:-"$ROOT/target/binary-size-reports/$STAMP-generic-expansion"}"
BINARY="${1:-}"

mkdir -p "$OUT_DIR"
cd "$ROOT"

SYMBOLS_RAW="$OUT_DIR/symbols.raw.txt"
SYMBOLS="$OUT_DIR/symbols.demangled.txt"

collect_symbols() {
  local target="$1"
  if [[ -z "$target" || ! -e "$target" ]]; then
    return 1
  fi
  nm -C --defined-only "$target" 2>/dev/null | awk '{$1=$2=""; sub(/^  */, ""); print}' >> "$SYMBOLS_RAW" || true
}

: > "$SYMBOLS_RAW"

if [[ -n "$BINARY" ]]; then
  collect_symbols "$BINARY" || true
else
  while IFS= read -r candidate; do
    collect_symbols "$candidate" || true
  done < <(
    find target -type f \( -name 'librspack_node.so' -o -name '*.node' -o -name '*.rlib' \) 2>/dev/null | sort
  )
fi

if [[ -s "$SYMBOLS_RAW" ]]; then
  cp "$SYMBOLS_RAW" "$SYMBOLS"
  awk '
    /</ && />/ {
      name = $0
      while (gsub(/<[^<>]*>/, "<...>", name)) {}
      count[name]++
    }
    END {
      for (name in count) {
        print count[name] "\t" name
      }
    }
  ' "$SYMBOLS" | sort -nr > "$OUT_DIR/generic-groups.tsv"

  {
    echo "# Generic marker counts"
    for pattern in \
      'ThreadsafeFunction<' \
      'Function<' \
      'Promise<' \
      'Either<' \
      'Either3<' \
      'Either4<' \
      'Either5<' \
      'FromNapiValue' \
      'ToNapiValue' \
      'ValidateNapiValue' \
      'TypeName' \
      'CallbackInfo<' \
      'Vec<' \
      'Option<' \
      'Result<'
    do
      count="$(grep -F "$pattern" "$SYMBOLS" 2>/dev/null | wc -l | tr -d ' ')"
      printf '%s\t%s\n' "$pattern" "$count"
    done
    echo
    echo "# Top normalized generic groups"
    head -80 "$OUT_DIR/generic-groups.tsv"
  } > "$OUT_DIR/summary.txt"
else
  cat > "$OUT_DIR/summary.txt" <<'EOF'
No symbols were collected.

Build an unstripped or profiling binary, or run this script after Rust artifacts exist in target/.
Stripped release binaries usually do not contain enough symbol information for generic expansion analysis.
EOF
fi

if [[ "${RUN_CARGO_BLOAT:-0}" == "1" ]] && command -v cargo-bloat >/dev/null 2>&1; then
  package="${CARGO_BLOAT_PACKAGE:-rspack_node}"
  cargo bloat --release -p "$package" --crates > "$OUT_DIR/cargo-bloat-crates.txt" 2>"$OUT_DIR/cargo-bloat-crates.stderr.txt" || true
  cargo bloat --release -p "$package" --functions > "$OUT_DIR/cargo-bloat-functions.txt" 2>"$OUT_DIR/cargo-bloat-functions.stderr.txt" || true
elif [[ "${RUN_CARGO_BLOAT:-0}" == "1" ]]; then
  echo "RUN_CARGO_BLOAT=1 was set, but cargo-bloat was not found." > "$OUT_DIR/cargo-bloat.skipped.txt"
else
  echo "cargo-bloat was skipped. Set RUN_CARGO_BLOAT=1 for section-level attribution." > "$OUT_DIR/cargo-bloat.skipped.txt"
fi

echo "generic expansion report: $OUT_DIR"
