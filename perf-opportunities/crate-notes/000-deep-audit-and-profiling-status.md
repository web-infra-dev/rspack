# Deep Audit + Profiling Status (All Crates)

## Scope Completed

- Reviewed all crate notes in `perf-opportunities/crate-notes/*.md` (92 crates + audit indexes).
- Cross-referenced with deep-dive analyses in `perf-opportunities/` for core hotspots.
- Ran profiling that is feasible on this host and captured new trace artifacts.

## Environment Constraints

- Host is macOS (`darwin`), so Linux `perf` is unavailable (`perf: command not found`).
- Native profiling tools available: `/usr/bin/sample`, `/usr/bin/xctrace`.
- Existing Linux `perf` evidence remains available in:
  - `perf-opportunities/survey-0f36/profiling-results.md`
  - `perf-opportunities/survey-0f36/workload-react-10k.md`

## Profiling Executed In This Audit

### 1) Rspack CLI global trace (Perfetto layer)

Command executed:

```bash
RSPACK_PROFILE=OVERVIEW node ../../../bin/rspack.js build -c ./rspack.config.js
```

Working directory:

- `packages/rspack-cli/tests/build/profile`

Artifact produced:

- `packages/rspack-cli/tests/build/profile/.rspack-profile-1770669744802-43975/rspack.pftrace`

### 2) Rspack CLI logger trace (JSON lines, stack/span context)

Command executed:

```bash
RSPACK_PROFILE=rspack_core=trace,rspack_plugin_javascript=trace \
RSPACK_TRACE_LAYER=logger \
RSPACK_TRACE_OUTPUT=trace.log \
node ../../../bin/rspack.js build -c ./rspack.config.js
```

Artifact produced:

- `packages/rspack-cli/tests/build/profile/.rspack-profile-1770669754820-44843/trace.log`

Notable spans observed in trace log:

- `NormalModule:build`
- `JavaScriptParser:parse`
- `Compilation:build_chunk_graph`
- `FlagDependencyExportsPlugin:finish_modules`
- `FlagDependencyUsagePlugin:optimize_dependencies`
- `ModuleConcatenationPlugin:optimize_chunk_modules`
- `Compilation:process_modules_runtime_requirements`
- `hook:CompilationProcessAssets`

### 3) Real react-10k builds on macOS (local linked rspack)

Repository:

- `/Users/zackjackson/build-tools-performance`

Case:

- `cases/react-10k`

Command:

```bash
pnpm run build:rspack
```

Observed results in this audit:

- Run A: `Rspack compiled with 2 warnings in 97.72 s`
- Run B: `Rspack compiled with 2 warnings in 98.79 s`
- Run C (`RSPACK_PROFILE=OVERVIEW`): `18.01 s`
- Run D (`RSPACK_TRACE_LAYER=logger` filtered): `13.15 s`

Artifacts:

- `build-tools-performance/cases/react-10k/.rspack-profile-1770673656168-8888/rspack.pftrace`
- `build-tools-performance/cases/react-10k/.rspack-profile-1770673684525-10298/react10k-trace.log`

### 4) xctrace attempts (blocked in this environment)

Attempted commands:

- `xctrace record --template "Time Profiler" --launch -- ...`
- `xctrace record --template "Allocations" --launch -- ...`
- `xctrace record --template "Time Profiler" --attach <pid> --time-limit 20s ...`

Outcome:

- `xctrace` starts but never produces trace bundles in this non-interactive agent shell.
- This is likely due Instruments/TCC/session constraints (headless automation context).
- We retained Rspack-native tracing and react-10k wall-time results as primary evidence.

## Independent Per-Crate Static Scan (92 crates)

Method:

- Parsed every Rust file under `crates/*`.
- Counted static perf-signal patterns per crate (`clone`, `collect`, `DashMap`, `Mutex/RwLock`, `Box<dyn>`, `async_trait`, `to_string`, `format!`, `Arc`).

Highest signal crates (selected):

- `rspack_core` (55k LOC): clone/collect/dispatch density remains highest.
- `rspack_plugin_javascript` (39k LOC): high clone + formatting + parser/plugin path pressure.
- `rspack_binding_api` (23k LOC): interop boundary + async/hook-heavy patterns.
- `rspack_storage` (6.4k LOC): serialization/string/arc-heavy code paths.
- `rspack_plugin_runtime` (7.5k LOC): high formatting/template density.
- `rspack_plugin_mf` (7.6k LOC): high clone/mutex/async use in federation plumbing.

Reference:

- `perf-opportunities/crate-notes/002-independent-crate-static-signal-metrics.md`
- `perf-opportunities/crate-notes/003-file-level-manual-review-coverage-index.md`
- `perf-opportunities/crate-notes/004-all-crates-detailed-opportunities.md`

## Evidence-Driven Remaining Hotspots

Based on combined historical + new trace evidence:

1. **Build module graph throughput**
   - `rspack_core`, `rspack_plugin_javascript`, `rspack_loader_*`
2. **AST traversal and JS analysis pressure**
   - `rspack_plugin_javascript`, `rspack_javascript_compiler`
3. **Chunk graph/code splitting scalability**
   - `rspack_core`, `rspack_plugin_split_chunks`, `rspack_plugin_runtime`
4. **Cache/incremental and storage efficiency**
   - `rspack_storage`, `rspack_cacheable`, `rspack_core` incremental paths
5. **Interop and hook dispatch overhead**
   - `rspack_binding_api`, `node_binding`, `rspack_hook`

## Where To Continue Deep Work

- Crate-by-crate remaining opportunities:
  - `perf-opportunities/crate-notes/001-all-crates-remaining-opportunities.md`
- Per-crate notes:
  - `perf-opportunities/crate-notes/*.md`
- Macro-level deep dives and react-10k modeling:
  - `perf-opportunities/20-deep-dive-sideeffects-bottleneck.md`
  - `perf-opportunities/21-deep-dive-codesplitter-biguint.md`
  - `perf-opportunities/22-deep-dive-task-loop-main-thread.md`
  - `perf-opportunities/42-react-10k-benchmark-analysis.md`
  - `perf-opportunities/43-react-10k-actual-profile.md`
