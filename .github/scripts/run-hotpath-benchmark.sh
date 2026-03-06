#!/usr/bin/env bash
set -euo pipefail

BASE_SHA="$1"
METRICS_DIR="$2"
BENCHMARK_DIR="${3:-examples/react}"

ROOT_DIR="$(pwd)"
HEAD_SHA="$(git rev-parse HEAD)"
RUNNER_TEMP_DIR="${RUNNER_TEMP:-/tmp}"
BASE_WORKTREE="${RUNNER_TEMP_DIR}/rspack-hotpath-base-${BASE_SHA:0:12}"

mkdir -p "$METRICS_DIR"

cleanup() {
  git worktree remove --force "$BASE_WORKTREE" >/dev/null 2>&1 || true
}

trap cleanup EXIT

build_rspack() {
  pnpm install --frozen-lockfile
  pnpm run build:binding:dev
  pnpm --filter "@rspack/core" build
  pnpm --filter "@rspack/cli" build
}

run_benchmark() {
  local repo_dir="$1"
  local label="$2"
  local output_json="$3"

  echo "==> Benchmarking ${label}"
  pushd "$repo_dir" >/dev/null
  build_rspack

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

git worktree add --force --detach "$BASE_WORKTREE" "$BASE_SHA"
run_benchmark "$BASE_WORKTREE" "base (${BASE_SHA})" "${METRICS_DIR}/base.json"
