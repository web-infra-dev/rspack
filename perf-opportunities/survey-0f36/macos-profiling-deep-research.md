# macOS Profiling Deep Research (Rspack)

## Why this document

This records practical profiling workflows and findings on macOS, where Linux
`perf` is unavailable but Rspack tracing + Instruments tooling are available.

## Tooling availability on this machine

- `perf`: unavailable (`command not found`)
- `sample`: available (`/usr/bin/sample`)
- `xctrace`: available (`/usr/bin/xctrace`)
- Instruments templates available include:
  - `Time Profiler`
  - `Allocations`
  - `System Trace`
  - `File Activity`

## Rspack-native profiling run (completed)

### A) Perfetto trace output

Run from:

- `packages/rspack-cli/tests/build/profile`

Command:

```bash
RSPACK_PROFILE=OVERVIEW \
node ../../../bin/rspack.js build -c ./rspack.config.js
```

Produced:

- `.rspack-profile-1770669744802-43975/rspack.pftrace`

### B) JSON logger traces with spans

Command:

```bash
RSPACK_PROFILE=rspack_core=trace,rspack_plugin_javascript=trace \
RSPACK_TRACE_LAYER=logger \
RSPACK_TRACE_OUTPUT=trace.log \
node ../../../bin/rspack.js build -c ./rspack.config.js
```

Produced:

- `.rspack-profile-1770669754820-44843/trace.log`

Observed representative spans:

- `NormalModule:build`
- `JavaScriptParser:parse`
- `Compilation:build_chunk_graph`
- `FlagDependencyExportsPlugin:finish_modules`
- `FlagDependencyUsagePlugin:optimize_dependencies`
- `ModuleConcatenationPlugin:optimize_chunk_modules`
- `Compilation:process_modules_runtime_requirements`
- `hook:CompilationProcessAssets`

## react-10k runs executed on this machine

Workload:

- `/Users/zackjackson/build-tools-performance/cases/react-10k`

Command:

```bash
pnpm run build:rspack
```

Observed wall times in this audit:

- 97.72s
- 98.79s

These runs used linked local packages:

- `@rspack/core@link:/Users/zackjackson/rspack/packages/rspack`
- `@rspack/cli@link:/Users/zackjackson/rspack/packages/rspack-cli`

## Deep profiling workflows to run next (macOS-native)

### 1) Time Profiler (CPU hotspots)

```bash
xctrace record \
  --template "Time Profiler" \
  --output ./rspack-time-profiler.trace \
  --launch -- \
  node ./packages/rspack-cli/bin/rspack.js build -c ./path/to/rspack.config.js
```

Focus in trace:

- `rspack_core::compilation::build_module_graph::*`
- `rspack_plugin_javascript::parser_and_generator::*`
- `rspack_core::compilation::build_chunk_graph::*`
- `rspack_core::compilation::code_generation::*`

### 2) Allocations (heap pressure + churn)

```bash
xctrace record \
  --template "Allocations" \
  --output ./rspack-allocations.trace \
  --launch -- \
  node ./packages/rspack-cli/bin/rspack.js build -c ./path/to/rspack.config.js
```

Focus in trace:

- hot allocation backtraces in parser, module graph updates, and codegen buffers
- sustained growth vs transient spikes during pass boundaries

### 3) Quick stack snapshot with `sample`

1. Launch build in one terminal.
2. Capture process id (`pgrep -f "rspack.js build"`).
3. Run:

```bash
sample <PID> 10 -file ./rspack-sample.txt
```

Use for:

- fast stack capture on long-running phases
- validating whether main thread is blocking task completion

## xctrace execution status in this environment

`xctrace` templates are available, but CLI recordings in this agent shell did
not complete into `.trace` bundles (launch and attach modes both hung).

Attempted commands included:

- `xctrace record --template "Time Profiler" --launch -- ...`
- `xctrace record --template "Allocations" --launch -- ...`
- `xctrace record --template "Time Profiler" --attach <pid> --time-limit 20s ...`

Practical implication:

- Current hard evidence for this session relies on:
  - react-10k repeated wall-time runs
  - `RSPACK_PROFILE` logger/perfetto traces
  - existing Linux perf artifacts in `profiling-results.md`

## Current remaining opportunities (macOS evidence aligned)

1. **Make phase throughput** (`rspack_core`, `rspack_plugin_javascript`)
2. **Parser/AST pass consolidation** (`rspack_plugin_javascript`, `rspack_javascript_compiler`)
3. **Chunk graph bitset + traversal efficiency** (`rspack_core`, `rspack_plugin_split_chunks`)
4. **Codegen and concatenation rework** (`rspack_core`, `rspack_plugin_javascript`)
5. **Cache and IO path efficiency** (`rspack_storage`, `rspack_cacheable`)

## Related docs

- `survey-0f36/profiling-results.md`
- `crate-notes/000-deep-audit-and-profiling-status.md`
- `crate-notes/001-all-crates-remaining-opportunities.md`
- `43-react-10k-actual-profile.md`
