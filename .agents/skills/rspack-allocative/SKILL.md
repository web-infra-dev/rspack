---
name: rspack-allocative
description: Capture local Rspack allocative snapshots for Speedscope and measure explicit live-object or collection-entry counts. Use for Rust data-structure memory analysis and snapshot comparisons.
---

# Rspack allocative

Produce inspectable local artifacts: a raw `.allocative` snapshot and, when requested, measured population counts with a CSV table and a separate count-weighted folded file. Use Speedscope to view flamegraphs. Explain the capture phase and coverage with every result.

## Establish coverage

Use the user's selected checkout or current Rspack worktree. When invoked outside a checkout, locate the requested Rspack repository before building. Resolve paths; do not assume another checkout's binding matches the selected revision.

Check these integration points in the selected revision:

- `crates/node_binding/scripts/build.mjs`: debug (`release-debug`) and profiling builds enable the Cargo feature and `--cfg=allocative` automatically; `ALLOCATIVE` opts other binding builds in.
- `crates/rspack_core/src/compiler/mod.rs`: the `snapshot_allocative("build", self)` call defines the capture phase.
- `crates/rspack_core/src/utils/mod.rs`: `RSPACK_ALLOCATIVE_DIR`, naming, and root traversal.
- Cargo manifests: the actual allocative package/version and feature propagation.

The current integration visits **Compiler → Compilation and other owned fields**, registered global roots, and the process-global Ustr string arena, after cache finalization and before compiler destruction. Shared allocations are charged once, below their first encountered owner; later references and cycles do not add their payload again. Hash iteration can change the first-owner path between runs. Read the snapshot metadata and warnings before interpreting a total. Older revisions visited global roots only; inspect the checkout before comparing results.

An object-tree byte profile is not RSS, peak heap usage, cumulative allocations, or a complete process heap census. Skipped fields, missing roots, and missing trait implementations limit coverage. Objects already freed will not appear in a later snapshot.

## Build and capture

Follow the checkout's AGENTS instructions and development commands. Set `RSPACK_ROOT` to its absolute path. After installing project dependencies, build the local Node binding and JS packages:

```sh
cd "$RSPACK_ROOT"
ALLOCATIVE=1 pnpm run build:binding:dev
pnpm run build:js
```

For large workloads, `pnpm run build:binding:profiling` is an alternative if appropriate for the installed toolchain. Both `build:binding:debug` (`release-debug`) and `build:binding:profiling` include allocative automatically. Confirm the build log enables **both** the Cargo feature and cfg. Dev, CI, and release binding builds remain opt-in through `ALLOCATIVE`. Do not use `ALLOCATIVE=0` to disable it: the existing JavaScript build script checks truthiness. Unset it for ordinary builds. In all profiles, leave `RSPACK_ALLOCATIVE_DIR` unset to disable snapshot collection.

Run from the target project's working directory with its real configuration. Set `PROFILE_DIR` to an absolute, fresh directory for each process/run:

```sh
RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  node "$RSPACK_ROOT/packages/rspack-cli/bin/rspack.js" \
  build --config "$CONFIG_PATH"
```

Verify the workload loads the newly built local binding, including config/plugin imports that resolve their own `@rspack/core`. For external projects, a debug or profiling binding built with this integration can capture snapshots without recompilation; `@rspack-debug/core` overrides are documented in the repository's debugging guide. Verify the selected package version includes this support. A published binding without allocative cannot be enabled by a runtime variable.

Snapshots typically start at `0-build.allocative` and increment within a process. Separate processes can overwrite those names if they share a directory. Check exit status, nonempty outputs, and capture phase. For watch/rebuild workloads, inspect the rebuild path instead of assuming every rebuild emits a snapshot.

For a standalone Rust benchmark, `ALLOCATIVE=1` alone does nothing: the Node build script consumes it. Build with both the feature and cfg, then run one existing case per fresh directory. For the walltime benchmark:

```sh
cargo bench -p rspack_benchmark --bench walltime --profile codspeed \
  --features allocative \
  --config 'target."cfg(all())".rustflags=["--cfg=allocative"]' --no-run
BENCH_MODE=walltime RSPACK_BENCHCASES_DIR="$BENCHCASES_DIR" \
  RSPACK_ALLOCATIVE_DIR="$PROFILE_DIR" \
  "$WALLTIME_BINARY" threejs-10x-development --test
```

Use the executable path printed by Cargo. Prepare fixtures through the repository's bench preparation command; run `threejs-10x-production-sourcemap` separately when needed. Keep the original workload and concurrency. Profiling builds require the feature and cfg together; ordinary builds remain disabled.

## Object counts and missing roots

For counts or unobserved structures, read [references/object-counts.md](references/object-counts.md). Add focused, cfg-gated instrumentation in the selected checkout when necessary for the authorized profiling task. Keep it separate from unrelated optimization/CI work.

The native snapshot already emits module and dependency **entry counts grouped by type**, at the same point as the byte profile. These counts cover the current compilation module graph, not all allocations or retained cache entries. For additional populations, capture counts at the **same point** as the byte snapshot. Prefer authoritative collection cardinalities or traversal with stable-identity deduplication. Record each population, counting basis, unit, snapshot identity, and scope. Allocative's folded format stores byte weights, not instance counts: never derive counts by dividing weights by `size_of::<T>()` or counting folded lines.

For a full census request, identify missing types and add counters where feasible. Do not silently substitute entries for unique objects. Missing populations mean unmeasured, not zero. Use separate byte and count artifacts.

## Prepare and inspect

The bundled zx helper uses the repository's installed dependencies to validate and prepare data for Speedscope. It does not generate SVGs or require a flamegraph renderer.

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

Treat `Unobserved nested allocations` warnings as missing heap coverage, not zero bytes. Nonblocking locks can also make a snapshot incomplete. External regex engines, callback captures, V8 objects, and private persistence internals are examples of explicit boundaries. Container private metadata and allocator overhead are not fully represented; see metadata limits. An accounting warning about children exceeding a parent is a profiler bug to investigate, not an ordinary coverage limitation.

For comparisons, match input, phase, profile, roots, units, and counting methods. Keep separate run directories. Do not sum parent and child inclusive sizes. A stage-end snapshot is not evidence of maximum live size during that stage.

## Validate and report

Compile changed Rust instrumentation before executing it. Run the requested workload and prepare its artifacts for Speedscope. Explain unobserved populations and report collection/build failures. Synthetic helper tests are not Rspack measurements.

Return raw snapshots, count-weighted folded files and the counts table when collected, and findings with units, phase, and coverage. Mention the selected local binding and any instrumentation changes. Preserve artifacts for inspection. Profiling does not imply permission to publish data or update a PR.

After each run that produces snapshots, include a clickable absolute link to the directory containing the `.allocative` files and identify the snapshot filenames. Tell the user to open [Speedscope](https://www.speedscope.app/) and drag in a `.allocative` file to explore it with search, zoom, and the **Left Heavy** view. Explain that the weights are bytes and the paths represent ownership; the folded file does not carry a unit, and **Time Order** does not represent a timeline for these snapshots. If no snapshot was produced, report that explicitly instead of linking an expected output path.
