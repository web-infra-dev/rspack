#!/usr/bin/env bash
# Measure the second identical-commit build's cache coverage with the production-shaped
# two-layer cache (dependency artifacts kept fresh by the target cache + sccache for the
# workspace crates the target cache strips). Used by sccache-poc.yml on Linux and Windows.
set -uo pipefail

members=$(cargo metadata --no-deps --format-version 1 \
  | node -e 'const d=JSON.parse(require("fs").readFileSync(0,"utf8"));console.log(d.packages.map(p=>p.name).join(" "))')
echo "workspace members: $(echo "$members" | wc -w)"

build() { cargo build --profile ci -p rspack_node --no-default-features --features plugin,perfetto; }

echo "### build1 (cold) ###"
build
sccache --show-stats | grep -iE 'hits rate|Cache (hits|misses) +[0-9]' || true

# Simulate the dependency-cache restore: strip ONLY workspace members so they recompile,
# while every dependency artifact (swc_core, mimalloc, ...) stays fresh and is NOT rebuilt.
for m in $members; do cargo clean --profile ci -p "$m" 2>/dev/null || true; done
sccache --zero-stats

echo "### build2 (second identical-commit build) ###"
build

echo "### COVERAGE = build2 sccache hit rate ###"
sccache --show-stats | grep -iE 'hits rate|Cache (hits|misses) +[0-9]|Non-cacheable calls' || true
