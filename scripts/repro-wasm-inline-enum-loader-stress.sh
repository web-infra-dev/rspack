#!/usr/bin/env bash
# Stress-run the config case that has been observed to trip WASM loader deserialization
# (`RuntimeError: memory access out of bounds` in `Promise<JsLoaderContext>` / emnapi Worker).
#
# Prereqs: same as CI "Test WASM" — prebuilt wasm bindings under crates/node_binding/,
# `pnpm i` at repo root. From repo root:
#   bash scripts/repro-wasm-inline-enum-loader-stress.sh 30
#
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT}/tests/rspack-test"
export NODE_NO_WARNINGS=1
export WASM=1
export RSPACK_LOADER_WORKER_THREADS=1
export NODE_OPTIONS='--max_old_space_size=8192 --stack-trace-limit=100'
RUNS="${1:-20}"
for i in $(seq 1 "${RUNS}"); do
	echo "=== iteration ${i}/${RUNS} ==="
	pnpm run test -- -t inline-enum/enum-module-in-concate-root
done
