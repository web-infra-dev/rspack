---
name: rspack-perf-profiling
description: Run Rspack performance profiling on Linux using perf (with DWARF call stacks), generate perf.data, and analyze hotspots. Use when you need CPU-level bottlenecks, kernel symbol resolution, or repeatable profiling for rspack build/bench cases. Includes optional samply import with per-CPU threads for visualization, but primary analysis is perf-based.
---

# Rspack Perf Profiling

## Overview
Profile Rspack builds on Linux using perf with DWARF call graphs, capture kernel + user stacks, and analyze hotspots directly from perf.data. Optionally import into samply for per-CPU threads visualization.

## Workflow

### 1) Build profiling-enabled binding (once per code change)
```sh
pnpm run build:binding:profiling
```

### 2) Enable kernel symbols (recommended)
```sh
echo 0 | sudo tee /proc/sys/kernel/kptr_restrict
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```
Optional: install vmlinux debug symbols or pass a vmlinux path to perf report.

### 3) Record perf profile (example: 10000 case)
```sh
# if benchmark repo isn't present yet
git clone https://github.com/web-infra-dev/rspack-ecosystem-benchmark.git

perf record -o ./rspack-ecosystem-benchmark/cases/10000/perf.data \
  -e cycles:uk -F 4000 --call-graph dwarf -- \
  node --perf-prof --perf-basic-prof --interpreted-frames-native-stack \
  ./packages/rspack-cli/bin/rspack.js \
  -c ./rspack-ecosystem-benchmark/cases/10000/rspack.config.js
```
Notes:
- `cycles:uk` captures user + kernel cycles.
- Increase `-F` for higher sample density; expect large perf.data.
- Ensure `--call-graph dwarf` for readable Rust stacks.

### 4) Analyze perf.data (perf-based)
Top hotspots (flat view):
```sh
perf report -i ./rspack-ecosystem-benchmark/cases/10000/perf.data \
  --stdio --no-children -g none --percent-limit 0.5 | head -n 100
```
Callgraph (if needed):
```sh
perf report -i ./rspack-ecosystem-benchmark/cases/10000/perf.data \
  --stdio --no-children -g graph,0.5,caller,function,percent | head -n 120
```

### 5) Optional: import into samply with per-CPU threads
```sh
samply import ./rspack-ecosystem-benchmark/cases/10000/perf.data \
  --per-cpu-threads -o ./rspack-ecosystem-benchmark/cases/10000/perf.profile.json.gz \
  --no-open
```
Use this only for visualization; keep analysis perf-first.

## Variants
- For other cases, swap `-c <case>/rspack.config.js`.
- For heavier workloads, wrap the rspack command in a loop to amplify time.
- If kernel symbols are still missing, pass `-k /path/to/vmlinux` to perf report.
