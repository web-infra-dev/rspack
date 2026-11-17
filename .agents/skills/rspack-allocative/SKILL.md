---
name: rspack-allocative
description: Capture local Rspack Rust and JavaScript memory snapshots for Speedscope, inspect cross-language object links, and measure explicit live-object or collection-entry counts.
---

# Rspack allocative

Workflow: **Select the workload → Build → Capture → Convert → Validate → Deliver**.

Produce local memory snapshots and reports for Speedscope. Include counts reports whenever matching measured counts are available; additional populations may require instrumentation. Honor requests for raw captures only.

## Select the workload

By default, use the repository's built-in benchmark projects. With no project, test case, or preset specified, capture `threejs-10x-development`. Honor an explicitly requested preset or mode; for a Three.js 10x production build with sourcemaps, use `threejs-10x-production-sourcemap`. Prepare fixtures through `pnpm run bench:prepare`. For JavaScript/V8 or cross-language analysis, use the local Node binding and CLI from `.bench/rspack-benchcases/threejs-10x`, with its `rspack.config.js` and `--mode development` (or `--mode production --devtool source-map`). For Rust-only analysis, use the same Node binding capture and inspect its native `.allocative` output. Standalone Rust benchmarks no longer capture automatically.

An explicitly selected external project or repository test case takes precedence over this default. Do not use the initial working directory as the workload unless the user requests that project. Test cases use their existing harness; external projects use their actual configuration.

### Project directory argument

Accept `--project-dir <path>` in the skill request, for example:

```text
$rspack-allocative --project-dir /path/to/my-app
$rspack-allocative --project-dir "../my app" production build with sourcemap
```

These are request arguments interpreted by the agent, not options passed to the Rspack CLI or the report helper. Also accept a project directory stated in natural language.

For explicitly selected project workloads, resolve `PROJECT_DIR` from `--project-dir` first, then the project named in the request. Use the initial working directory when the user explicitly requests the current project. Resolve relative paths against that initial working directory, preserve paths containing spaces, and check that the directory exists before building. Use its configuration and requested build mode.

Keep `RSPACK_ROOT` separate: it is the selected Rspack checkout used to build the local binding and JS packages, and defaults to the current Rspack worktree when available. An external `PROJECT_DIR` does not change the checkout. If the request names a config, resolve a relative `CONFIG_PATH` against `PROJECT_DIR`; otherwise inspect that project's scripts and configuration to select its actual build configuration.

When the user requests capture from a repository test case, accept the case directory alone; do not require `--project-dir tests/rspack-test`. Resolve and run it as described in [Test cases](#test-cases), rather than treating the fixture as a standalone project.

## Build

Follow the checkout's AGENTS instructions and development commands. Set `RSPACK_ROOT` to its absolute path. For project or test workloads, after installing dependencies, build the local Node binding and JS packages:

```sh
cd "$RSPACK_ROOT"
ALLOCATIVE=1 pnpm run build:binding:dev
pnpm run build:js
```

For large workloads, `pnpm run build:binding:profiling` is an alternative if appropriate for the installed toolchain. Both `build:binding:debug` (`release-debug`) and `build:binding:profiling` include allocative automatically. Confirm the build log enables **both** the Cargo feature and cfg. Dev, CI, and release binding builds remain opt-in through `ALLOCATIVE`. Do not use `ALLOCATIVE=0` to disable it: the existing JavaScript build script checks truthiness. Unset it for ordinary builds. In all profiles, leave `RSPACK_ALLOCATIVE_DIR` unset to disable snapshot collection.

## Capture

For project workloads, run from `PROJECT_DIR` with its real configuration. For test workloads, use the existing harness as described in [Test cases](#test-cases). Set `CONFIG_PATH` to the resolved absolute config path and `PROFILE_DIR` to an absolute, fresh directory for each process/run:

```sh
cd "$PROJECT_DIR"
RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  node --expose-gc "$RSPACK_ROOT/packages/rspack-cli/bin/rspack.js" \
  build --config "$CONFIG_PATH"
```

Verify the workload loads the matching JS package and native binding, including config/plugin imports that resolve their own `@rspack/core`. For paired captures, use the debug or profiling packages, or set `ALLOCATIVE=1` when building both the binding and JS for a custom release. The ordinary release JS build removes the snapshot code. Set `RSPACK_ALLOCATIVE_DIR` before creating the first compiler so the JS package caches the enabled state. See the debugging guide for `@rspack-debug/core` overrides.

Captures run after build/rebuild and before the JS completion callback. Shared allocations are counted once; first-owner attribution can vary between runs. For missing captures, instrumentation changes, or cross-revision differences, consult [references/implementation.md](references/implementation.md).

Snapshots typically start at `0-build.allocative` and increment within a process. Check exit status, nonempty outputs, and capture phase. For watch/rebuild workloads, verify which updates produced snapshots. Raw V8 snapshots are written under `v8/`; keep them and Rust identity files local when sharing reports.

### Test cases

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
  -t "$CASE_FILTER"
```

The literal double quotes in the positional argument select exactly one runner; an ordinary fuzzy filter can also select another runner and overwrite captures. Pass `-t` through the CLI: the harness derives its case filter from the command arguments. Use the selected revision's actual runner and project, rather than assuming this example applies to every test family. For paired captures, expose GC in the Node process running the test. If that Node version accepts `--expose-gc` in `NODE_OPTIONS`, use it so the flag reaches the test process.

Keep the fixture's mode, sourcemap settings, assertions, and update sequence. Verify that the selected case executed, rather than accepting a successful run containing only skipped or filtered tests. For HMR, retain the initial `build` snapshot and every `rebuild` snapshot with matching sidecars. Confirm the loaded binding belongs to the selected checkout. Report the case directory and runner alongside the capture artifacts.

## Convert

For paired Rust/V8 captures:

```sh
node "$RSPACK_ROOT/.agents/skills/rspack-allocative/scripts/v8-to-allocative.mjs" \
  "$PROFILE_DIR/v8/0-build.heapsnapshot"
```

The converter writes `.js.allocative`, `.combined.allocative`, `.links.json`, and per-category `.js.<category>.allocative` files under `reports/`. It preserves object names by default and omits string-node contents while retaining their byte weights. Use `--clear-class-names` to remove custom constructor names.

Prepare every captured snapshot, including HMR rebuilds, in a separate new report directory. The helper discovers matching counts automatically; if absent, produce a byte-only report and report counts as unmeasured.

Resolve `SKILL_DIR` to this installed skill directory. Prepare each native snapshot with:

```sh
pnpm --dir "$RSPACK_ROOT" exec zx "$SKILL_DIR/scripts/render-profile.mjs" \
  "$PROFILE_DIR/0-build.allocative" \
  --out-dir "$PROFILE_DIR/report-0"
```

The helper automatically discovers matching `.counts.json`, `.metadata.json`, and `.warnings.txt` sidecars; `--counts` overrides only the count path. Outputs: a copy of the byte snapshot, `summary.json`, and available metadata/warnings. With counts: also `counts.csv`, source JSON, and `counts.folded`. All-zero counts are reported in the table and summary; the empty folded file has no chart to view. The output directory must be new.

## Validate

Check metadata and warnings before interpreting totals. A byte profile is not RSS, peak usage, cumulative allocations, or a complete heap census. Missing roots and inaccessible allocations limit coverage; freed objects do not appear in later snapshots. `Unobserved nested allocations` and unavailable nonblocking locks indicate missing coverage, not zero bytes. Children exceeding a parent's declared size indicate an accounting bug.

For paired captures, validate that category totals equal the JS total and combined bytes equal Rust plus JS bytes. Identity matches alone do not prove native memory was measured: inspect `measured` and warnings. For ownership-link analysis or unmatched objects, read [references/cross-language.md](references/cross-language.md).

The report helper rejects mismatched snapshot IDs, negative counts, duplicate populations, and ancestor/descendant count rows. Check that sibling populations do not overlap. Preserve zero-valued populations in tables; an empty folded file for all-zero counts has no chart.

For comparisons, match workload, phase, build profile, roots, units, and counting methods. Do not sum parent and child inclusive sizes.

### Additional populations

For counts or unobserved structures, read [references/object-counts.md](references/object-counts.md). Add focused, cfg-gated instrumentation in the selected checkout when necessary for the authorized profiling task. Keep it separate from unrelated optimization/CI work.

The native snapshot already emits module and dependency **entry counts grouped by type**, at the same point as the byte profile. These counts cover the current compilation module graph, not all allocations or retained cache entries. For additional populations, capture counts at the **same point** as the byte snapshot. Prefer authoritative collection cardinalities or traversal with stable-identity deduplication. Record each population, counting basis, unit, snapshot identity, and scope. Allocative's folded format stores byte weights, not instance counts: never derive counts by dividing weights by `size_of::<T>()` or counting folded lines.

For a full census request, identify missing types and add counters where feasible. Do not silently substitute entries for unique objects. Missing populations mean unmeasured, not zero. Use separate byte and count artifacts.

## Deliver

Return clickable absolute links to the snapshot/report directories, identify filenames, and summarize findings with units, capture phase, and coverage. Include counts tables and count-weighted profiles when available. State the workload, resolved configuration, selected local binding, and any instrumentation changes. Preserve artifacts. If collection failed, report the failure instead of linking expected files. Profiling does not authorize publishing data or updating a PR.

Tell the user to open [Speedscope](https://www.speedscope.app/) and drag in a `.allocative` file or nonempty `counts.folded`. Use search, zoom, and **Left Heavy** to inspect large paths. State whether weights are bytes, objects, or entries: folded files do not carry units. Paths describe ownership, and **Time Order** is not a timeline.
