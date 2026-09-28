---
name: rspack-memory-optimize
description: Run a bounded, evidence-based Rspack Rust memory optimization loop using Allocative snapshots, jemalloc live bytes, and default-allocator process RSS. Use when investigating or reducing retained Rspack memory in a local app or repository fixture.
---

# Rspack memory optimization

Use this workflow when the goal is to reduce memory that remains after a Rspack compilation. It is designed for one local app workload and the public Rust benchmark fixtures; never put a private app's source, command output, environment, or reports into CI artifacts.

## Measurement model

- Allocative snapshots (`RSPACK_ALLOCATIVE_DIR`) show selected reachable Rust object graphs. Read the adjacent `*.coverage.json`; this is not a complete heap census. At most 10 snapshots are written per process by default. Start with `python3 scripts/memory/allocative_summary.py <snapshot.allocative>`.
- `rust_live_bytes` in the memory report is jemalloc's live allocated byte counter and is available only in the profiling binding. Do not use that process's RSS as the product RSS result.
- Default-allocator process reports (`RSPACK_MEMORY_REPORT_DIR`) supply a 100 ms sampled `peak_rss_bytes` for the compilation, process-lifetime peak RSS, current RSS, V8 heap, external bytes, and compilation duration. These include Node and every native allocator in the process.
- The memory report is written from the compiler's `done` hook. The first entry is the full build; later entries are rebuilds. Treat cold build as the primary result and rebuild as supporting evidence.
- Keep whole alloc/free traces and jemalloc call-stack profiles off by default. Use them only after an object snapshot points to a subsystem and keep their output local.

## Worktree and loop

1. Inspect `git status` and the current diff. Keep the user's original checkout untouched. Create an isolated worktree at the same HEAD, apply the tracked diff from `git diff HEAD` and copy any relevant untracked files into it. Do not drop staged or unstaged work from the source checkout.
2. Prepare the local target once, then capture three cold-build samples on the accepted baseline with the same app revision, environment, Node version, machine, and thread limits. Run a default-allocator build for process RSS and duration. Run the jemalloc profiling binding separately for Rust live bytes and Allocative snapshots. Keep reports under local temporary directories.
3. Summarize the latest baseline snapshots and form one concrete hypothesis tied to a retained type, field, or collection. Record the expected byte reduction and the correctness behavior that must remain unchanged.
4. Make one candidate change. Build the artifacts required by the check, run focused correctness checks, then capture three candidate samples using the same paired measurement runs.
5. Compare medians with:

   ```sh
   python3 scripts/memory/compare.py \
     --baseline-rust /tmp/rspack-memory/baseline/rust \
     --candidate-rust /tmp/rspack-memory/candidate/rust \
     --baseline-process /tmp/rspack-memory/baseline/process \
     --candidate-process /tmp/rspack-memory/candidate/process
   ```

   Keep a candidate only when jemalloc live bytes and default-allocator peak RSS each fall by at least 5%, and median compilation duration regresses by no more than 5%. If a check fails or the result is inconclusive, revert only that candidate and preserve earlier accepted commits.

6. Write a round log with hypothesis, code change, correctness checks, three-run medians, snapshot evidence, decision, and any revert reason. Repeat from the latest accepted state, one change per round, for at most 10 rounds. Stop earlier when no credible candidate remains. Do not push or create a pull request.

## Local app commands

The build:dev app command, its private environment variables, and all reports remain local. Set `RSPACK_MEMORY_REPORT_DIR` for each app process; also set `RSPACK_ALLOCATIVE_DIR` on the profiling run. Build the profiling binding with the repository's `pnpm run build:binding:profiling` command and jemalloc profile settings documented in `.agents/DEVELOPMENT.md`. Use the default binding separately for product RSS. Do not collect every jemalloc allocation event for the large app.

For CSS-heavy cases, inspect the process `external_bytes` and `array_buffers_bytes` as Node/native context. Those counters do not attribute Tailwind's native allocator to Rspack.

## Repository CI fixtures

Use only public fixtures in `xtask/benchmark/cases`. The memory benchmark target includes a high-cardinality module-graph build and CSS/Tailwind development build. CI uses CodSpeed's memory instrument on `ubuntu-24.04`; it runs separately from the one-pass Allocative capture so snapshot traversal and disk writes do not distort CodSpeed measurements. CI uploads only bounded Allocative summaries and coverage metadata. Run the fixture locally with:

```sh
pnpm run bench:prepare
pnpm run build:bench:memory
BENCH_MODE=memory \
RSPACK_BENCHCASES_DIR="$PWD/.bench/rspack-benchcases" \
cargo codspeed run --bench memory
```

Capture one-pass snapshots separately:

```sh
BENCH_MODE=memory \
RSPACK_BENCHCASES_DIR="$PWD/.bench/rspack-benchcases" \
RSPACK_ALLOCATIVE_DIR=/tmp/rspack-allocative \
RSPACK_ALLOCATIVE_MAX_SNAPSHOTS=3 \
RSPACK_ALLOCATIVE_SNAPSHOT_MODULE_GRAPH=1 \
cargo bench --profile codspeed -p rspack_benchmark --bench memory --features rspack_benchmark/allocative -- --test
```

CodSpeed allocation totals and peak memory describe the benchmark run; the post-compilation Allocative snapshot answers which selected objects remain. Use both views and state their scopes separately.
