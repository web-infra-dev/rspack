---
name: rspack-allocative
description: Capture local Rspack allocative snapshots for Speedscope and measure explicit live-object or collection-entry counts. Use for Rust data-structure memory analysis and snapshot comparisons.
---

# Rspack allocative

Produce inspectable local artifacts: a raw `.allocative` snapshot and, when requested, measured population counts with a CSV table and a separate count-weighted folded file. Use Speedscope to view flamegraphs. Explain the capture phase and coverage with every result.

## Select the workload

By default, use the repository's built-in benchmark projects. With no project, test case, or preset specified, capture `threejs-10x-development`. Honor an explicitly requested preset or mode; for a Three.js 10x production build with sourcemaps, use `threejs-10x-production-sourcemap`. Prepare fixtures through `pnpm run bench:prepare` and use the Rust benchmark capture path below. Build the artifacts that workload consumes; the standalone Rust benchmark does not require a Node binding build.

An explicitly selected external project or repository test case takes precedence over this default. Do not use the initial working directory as the workload unless the user requests that project. Test cases use their existing harness; external projects use their actual configuration.

## Project directory argument

Accept `--project-dir <path>` in the skill request, for example:

```text
$rspack-allocative --project-dir /path/to/my-app
$rspack-allocative --project-dir "../my app" production build with sourcemap
```

These are request arguments interpreted by the agent, not options passed to the Rspack CLI or the report helper. Also accept a project directory stated in natural language.

For explicitly selected project workloads, resolve `PROJECT_DIR` from `--project-dir` first, then the project named in the request. Use the initial working directory when the user explicitly requests the current project. Resolve relative paths against that initial working directory, preserve paths containing spaces, and check that the directory exists before building. Use its configuration and requested build mode; do not substitute a built-in benchmark for an explicitly selected project.

Keep `RSPACK_ROOT` separate: it is the selected Rspack checkout used to build the local binding and JS packages, and defaults to the current Rspack worktree when available. An external `PROJECT_DIR` does not change the checkout. If the request names a config, resolve a relative `CONFIG_PATH` against `PROJECT_DIR`; otherwise inspect that project's scripts and configuration to select its actual build configuration.

When the user requests capture from a repository test case, accept the case directory alone; do not require `--project-dir tests/rspack-test`. Resolve and run it as described in [Capture a test case](#capture-a-test-case), rather than treating the fixture as a standalone project.

## Establish coverage

Use the user's selected checkout or current Rspack worktree. When invoked outside a checkout, locate the requested Rspack repository before building. Resolve paths; do not assume another checkout's binding matches the selected revision.

Check these integration points in the selected revision:

- `crates/node_binding/scripts/build.mjs`: debug (`release-debug`) and profiling builds enable the Cargo feature, `--cfg=allocative`, and the allocative compiler driver automatically; `ALLOCATIVE` opts other binding builds in.
- `scripts/allocative-driver.mjs`: builds the host driver with the pinned nightly's `rustc-dev` and wraps Cargo for Rust-only profiling. Ordinary source enums are instrumented in memory; macro-generated and borrowed/const-array enums retain explicit adapters.
- `crates/rspack_core/src/compiler/mod.rs`: the `snapshot_allocative("build", self)` call defines the capture phase.
- `crates/rspack_core/src/utils/mod.rs`: `RSPACK_ALLOCATIVE_DIR`, naming, and root traversal.
- Cargo manifests: the actual allocative package/version and feature propagation.

The current integration visits **Compiler → Compilation and other owned fields**, registered global roots, and the process-global Ustr string arena, after cache finalization and before compiler destruction. Shared allocations are charged once, below their first encountered owner; later references and cycles do not add their payload again. Hash iteration can change the first-owner path between runs. Read the snapshot metadata and warnings before interpreting a total. Older revisions visited global roots only; inspect the checkout before comparing results.

An object-tree byte profile is not RSS, peak heap usage, cumulative allocations, or a complete process heap census. Skipped fields, missing roots, and missing trait implementations limit coverage. Objects already freed will not appear in a later snapshot.

## Build and capture

Follow the checkout's AGENTS instructions and development commands. Set `RSPACK_ROOT` to its absolute path. For project or test workloads, after installing dependencies, build the local Node binding and JS packages:

```sh
cd "$RSPACK_ROOT"
ALLOCATIVE=1 pnpm run build:binding:dev
pnpm run build:js
```

For large workloads, `pnpm run build:binding:profiling` is an alternative if appropriate for the installed toolchain. Both `build:binding:debug` (`release-debug`) and `build:binding:profiling` include allocative automatically. Confirm the build log enables **both** the Cargo feature and cfg. Dev, CI, and release binding builds remain opt-in through `ALLOCATIVE`. Do not use `ALLOCATIVE=0` to disable it: the existing JavaScript build script checks truthiness. Unset it for ordinary builds. In all profiles, leave `RSPACK_ALLOCATIVE_DIR` unset to disable snapshot collection.

For project workloads, run from `PROJECT_DIR` with its real configuration. For test workloads, use the existing harness as described in [Capture a test case](#capture-a-test-case). Set `CONFIG_PATH` to the resolved absolute config path and `PROFILE_DIR` to an absolute, fresh directory for each process/run:

```sh
cd "$PROJECT_DIR"
RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  node "$RSPACK_ROOT/packages/rspack-cli/bin/rspack.js" \
  build --config "$CONFIG_PATH"
```

Verify the workload loads the newly built local binding, including config/plugin imports that resolve their own `@rspack/core`. For external projects, a debug or profiling binding built with this integration can capture snapshots without recompilation; `@rspack-debug/core` overrides are documented in the repository's debugging guide. Verify the selected package version includes this support. A published binding without allocative cannot be enabled by a runtime variable.

Snapshots typically start at `0-build.allocative` and increment within a process. Separate processes can overwrite those names if they share a directory. Check exit status, nonempty outputs, and capture phase. For watch/rebuild workloads, inspect the rebuild path instead of assuming every rebuild emits a snapshot.

For a standalone Rust benchmark, run Cargo and the benchmark from `RSPACK_ROOT`; `BENCHCASES_DIR` selects its fixtures. `ALLOCATIVE=1` alone does nothing: the Node build script consumes it. Build with both the feature and cfg, then run one existing case per fresh directory. For the walltime benchmark:

```sh
cd "$RSPACK_ROOT"
pnpm exec zx scripts/allocative-driver.mjs cargo bench -p rspack_benchmark --bench walltime --profile codspeed \
  --features allocative \
  --config 'target."cfg(all())".rustflags=["--cfg=allocative"]' --no-run
BENCH_MODE=walltime RSPACK_BENCHCASES_DIR="$BENCHCASES_DIR" \
  RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  "$WALLTIME_BINARY" threejs-10x-development --test
```

Use the executable path printed by Cargo. Prepare fixtures through the repository's bench preparation command; run `threejs-10x-production-sourcemap` separately when needed. Keep the original workload and concurrency. Profiling builds require the feature and cfg together; ordinary builds remain disabled.

## Capture a test case

Accept requests such as:

```text
$rspack-allocative capture memory for test case hotCases/source-map/disable, including all HMR updates
$rspack-allocative 使用 tests/rspack-test/hotCases/source-map/disable 测试用例抓取内存
```

Resolve `CASE_DIR` from the supplied directory: accept an absolute path, a path relative to the initial working directory, a checkout-relative path such as `tests/rspack-test/hotCases/source-map/disable`, or a harness-relative path such as `hotCases/source-map/disable`. Resolve the latter two forms against the selected `RSPACK_ROOT` and its `tests/rspack-test` directory. Check that the case exists. If an absolute case path selects another Rspack checkout and none was explicitly selected, use that checkout; if it conflicts with an explicitly selected checkout, clarify the mismatch before building.

Derive `PROJECT_DIR` from the case's containing test harness (`tests/rspack-test` for these fixtures). A case may have its own `package.json`; that does not make it the harness root. Inspect the harness configuration, matching runner, and fixture requirements. Honor a requested runner or target; otherwise select one compatible existing runner. For `hotCases/source-map/disable`, `HotNode.test.mjs` can drive the original HMR sequence. Do not run a fixture through the standalone CLI or replace its updates with a synthetic project.

After building or verifying the local binding and JS packages, set `CASE_FILTER` to the case path relative to `PROJECT_DIR`, `TEST_RUNNER` to the runner's absolute path, and `PROFILE_DIR` to a fresh absolute directory. For the current Rstest harness, run from `PROJECT_DIR`:

```sh
cd "$PROJECT_DIR"
RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  pnpm test "\"$TEST_RUNNER\"" --project base \
  -t "$CASE_FILTER" --maxWorkers 1
```

The literal double quotes in the positional argument select exactly one runner; an ordinary fuzzy filter can also select another runner and overwrite captures. Pass `-t` through the CLI: the harness derives its case filter from the command arguments. Use the selected revision's actual runner and project, rather than assuming this example applies to every test family. Snapshot collection needs only `RSPACK_ALLOCATIVE_DIR` at runtime; the test framework needs no capture-specific changes.

Keep the fixture's mode, sourcemap settings, assertions, and update sequence. Verify that the selected case executed, rather than accepting a successful run containing only skipped or filtered tests. For HMR, retain the initial `build` snapshot and every `rebuild` snapshot with matching sidecars; prepare each snapshot in a separate report directory. Confirm the loaded binding belongs to the selected checkout. Report the case directory and runner alongside the capture artifacts.

## Object counts and missing roots

For counts or unobserved structures, read [references/object-counts.md](references/object-counts.md). Add focused, cfg-gated instrumentation in the selected checkout when necessary for the authorized profiling task. Keep it separate from unrelated optimization/CI work.

The native snapshot already emits module and dependency **entry counts grouped by type**, at the same point as the byte profile. These counts cover the current compilation module graph, not all allocations or retained cache entries. For additional populations, capture counts at the **same point** as the byte snapshot. Prefer authoritative collection cardinalities or traversal with stable-identity deduplication. Record each population, counting basis, unit, snapshot identity, and scope. Allocative's folded format stores byte weights, not instance counts: never derive counts by dividing weights by `size_of::<T>()` or counting folded lines.

For a full census request, identify missing types and add counters where feasible. Do not silently substitute entries for unique objects. Missing populations mean unmeasured, not zero. Use separate byte and count artifacts.

## Prepare and inspect

The bundled zx helper uses the repository's installed dependencies to validate and prepare data for Speedscope. It does not generate SVGs or require a flamegraph renderer.

Run the helper as part of the capture workflow for every snapshot, using a new report directory for each. When a matching `.counts.json` exists, automatically prepare `summary.json`, `counts.csv`, and `counts.folded` alongside the byte snapshot; the user does not need to request or run the report command separately. For HMR, prepare the initial build and all captured rebuilds. Preserve zero-valued populations in the table and summary, and explain that count-profile weights are entries rather than bytes. If counts are missing, report them as unmeasured and prepare the byte-only report. Honor a request for raw captures only.

Resolve `SKILL_DIR` to this installed skill directory. For byte-only data:

```sh
pnpm --dir "$RSPACK_ROOT" exec zx "$SKILL_DIR/scripts/render-profile.mjs" \
  "$PROFILE_DIR/0-build.allocative" \
  --out-dir "$PROFILE_DIR/report-0"
```

For matching measured counts:

```sh
pnpm --dir "$RSPACK_ROOT" exec zx "$SKILL_DIR/scripts/render-profile.mjs" \
  "$PROFILE_DIR/0-build.allocative" \
  --counts "$PROFILE_DIR/0-build.counts.json" \
  --out-dir "$PROFILE_DIR/report-0"
```

The helper automatically discovers matching `.counts.json`, `.metadata.json`, and `.warnings.txt` sidecars; `--counts` overrides only the count path. Outputs: a copy of the byte snapshot, `summary.json`, and available metadata/warnings. With counts: also `counts.csv`, source JSON, and `counts.folded`. All-zero counts are reported in the table and summary; the empty folded file has no chart to view. The output directory must be new.

The helper records units separately as bytes and objects/entries. It rejects mismatched snapshot IDs, negative counts, duplicate populations, and ancestor/descendant count rows that would double-count totals. Distinct sibling paths can still describe overlapping populations: check their semantics yourself.

Open [Speedscope](https://www.speedscope.app/) and drag in the `.allocative` snapshot or a nonempty `counts.folded` file. Wider byte frames indicate more represented memory; wider count frames indicate larger measured populations. Folded files do not carry units, so state which metric each file contains. Count paths group by ownership scope, not allocation call stack. CSV rows retain the counting basis.

Treat `Unobserved nested allocations` warnings as missing heap coverage, not zero bytes. Nonblocking locks can also make a snapshot incomplete. Unsupported nested types and raw pointers fall back to opaque automatically; ordinary SWC options and initialized configuration caches expose supported fields. Callback captures and V8 objects remain inaccessible. The persistence backend also remains opaque because creating its reflection vtable fails on the pinned compiler. Container private metadata and allocator overhead are not fully represented; see metadata limits. An accounting warning about children exceeding a parent is a profiler bug to investigate, not an ordinary coverage limitation.

For comparisons, match input, phase, profile, roots, units, and counting methods. Keep separate run directories. Do not sum parent and child inclusive sizes. A stage-end snapshot is not evidence of maximum live size during that stage.

## Validate and report

Compile changed Rust instrumentation before executing it. Run the requested workload and prepare its artifacts for Speedscope. Explain unobserved populations and report collection/build failures. Synthetic helper tests are not Rspack measurements.

Return raw snapshots, count-weighted folded files and the counts table when collected, and findings with units, phase, and coverage. Mention the workload project directory, resolved configuration, selected local binding, and any instrumentation changes. Preserve artifacts for inspection. Profiling does not imply permission to publish data or update a PR.

After each run that produces snapshots, include a clickable absolute link to the directory containing the `.allocative` files and identify the snapshot filenames. Tell the user to open [Speedscope](https://www.speedscope.app/) and drag in a `.allocative` file to explore it with search, zoom, and the **Left Heavy** view. Explain that the weights are bytes and the paths represent ownership; the folded file does not carry a unit, and **Time Order** does not represent a timeline for these snapshots. If no snapshot was produced, report that explicitly instead of linking an expected output path.
