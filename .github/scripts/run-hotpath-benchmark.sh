#!/usr/bin/env bash
set -euo pipefail

BASE_SHA="$1"
METRICS_DIR="$2"
BENCHMARK_DIR="${3:-examples/react}"

ROOT_DIR="$(pwd)"
HEAD_SHA="$(git rev-parse HEAD)"
mkdir -p "$METRICS_DIR"

require_binding() {
  local repo_dir="$1"

  if compgen -G "${repo_dir}/crates/node_binding/*.node" > /dev/null; then
    return
  fi

  echo "Missing binding artifact in ${repo_dir}/crates/node_binding" >&2
  echo "Hotpath benchmark only reuses downloaded bindings and does not build them in CI." >&2
  return 1
}

build_js() {
  pnpm install --frozen-lockfile
  pnpm --filter "@rspack/core" build
  pnpm --filter "@rspack/cli" build
}

run_benchmark() {
  local repo_dir="$1"
  local label="$2"
  local output_json="$3"

  echo "==> Benchmarking ${label}"
  pushd "$repo_dir" >/dev/null
  require_binding "$repo_dir"
  build_js

  pushd "$BENCHMARK_DIR" >/dev/null
  rm -rf dist
  NO_COLOR=1 \
  CI=true \
  RSPACK_PROFILE=OVERVIEW \
  RSPACK_TRACE_LAYER=hotpath \
  RSPACK_TRACE_OUTPUT="$output_json" \
  pnpm exec rspack build
  popd >/dev/null

  popd >/dev/null
}

run_benchmark "$ROOT_DIR" "head (${HEAD_SHA})" "${METRICS_DIR}/head.json"

if [ "$BASE_SHA" = "$HEAD_SHA" ]; then
  cp "${METRICS_DIR}/head.json" "${METRICS_DIR}/base.json"
  exit 0
fi

BASE_WORKTREE="${HOTPATH_BASE_WORKTREE:?HOTPATH_BASE_WORKTREE is required when comparing against a different base SHA}"
run_benchmark "$BASE_WORKTREE" "base (${BASE_SHA})" "${METRICS_DIR}/base.json"
