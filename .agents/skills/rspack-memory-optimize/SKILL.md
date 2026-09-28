---
name: rspack-memory-optimize
description: Run a bounded Rspack memory optimization loop using jemalloc live bytes, sampled heap profiles, and default-allocator process RSS. Use when investigating or reducing retained Rspack memory in a local app or repository fixture.
---

# Rspack memory optimization

Use this workflow when the goal is to reduce memory that remains after a Rspack compilation. It is designed for one local app workload and the public Rust benchmark fixtures; never put a private app's source, command output, environment, or reports into CI artifacts.

## Measurement model

- `rust_live_bytes` in the memory report is jemalloc's live allocated byte counter and is available only in the profiling binding. Do not use that process's RSS as the product RSS result.
- Default-allocator process reports (`RSPACK_MEMORY_REPORT_DIR`) supply a 100 ms sampled `peak_rss_bytes` for the compilation, process-lifetime peak RSS, current RSS, V8 heap, external bytes, and compilation duration. These include Node and every native allocator in the process.
- The memory report is written from the compiler's `done` hook. The first entry is the full build; later entries are rebuilds. Treat cold build as the primary result and rebuild as supporting evidence.
- Sampled jemalloc heap profiles can provide allocation call stacks for live allocations. They help locate allocation sites, but do not identify retained Rust object types. Enable them only when needed and keep output local.
- Keep whole alloc/free traces off by default. Use them only after a bounded profile points to a subsystem.

## Worktree and loop

1. Inspect `git status` and the current diff. Keep the user's original checkout untouched. Create an isolated worktree at the same HEAD, apply the tracked diff from `git diff HEAD` and copy any relevant untracked files into it. Do not drop staged or unstaged work from the source checkout.
2. Prepare the local target once, then capture three cold-build samples on the accepted baseline with the same app revision, environment, Node version, machine, and thread limits. Run a default-allocator build for process RSS and duration. Run the jemalloc profiling binding separately for Rust live bytes. Keep reports under local temporary directories.
3. Form one concrete hypothesis from the jemalloc live profile and source inspection. Record the expected byte reduction and the correctness behavior that must remain unchanged.
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

6. Write a round log with hypothesis, code change, correctness checks, three-run medians, profile evidence, decision, and any revert reason. Repeat from the latest accepted state, one change per round, for at most 10 rounds. Stop earlier when no credible candidate remains. Do not push or create a pull request.

## Local app commands

The build:dev app command, its private environment variables, and all reports remain local. Set `RSPACK_MEMORY_REPORT_DIR` for each app process. Build the profiling binding with the repository's `pnpm run build:binding:profiling` command; use the jemalloc profile settings documented in `.agents/DEVELOPMENT.md` when a sampled heap profile is needed. Use the default binding separately for product RSS. Do not collect every jemalloc allocation event for the large app.

For CSS-heavy cases, inspect process `external_bytes` and `array_buffers_bytes` as Node/native context. Those counters do not attribute Tailwind's native allocator to Rspack.

## Repository CI fixtures

Use only public fixtures in `xtask/benchmark/cases`. The memory benchmark target includes a high-cardinality module-graph build and CSS/Tailwind development build. CI uses CodSpeed's memory instrument on `ubuntu-24.04`. Run the fixture locally with:

```sh
pnpm run bench:prepare
pnpm run build:bench:memory
BENCH_MODE=memory \
RSPACK_BENCHCASES_DIR="$PWD/.bench/rspack-benchcases" \
cargo codspeed run --bench memory
```

CodSpeed allocation totals and peak-memory measurements describe the benchmark run. Use them to compare the same fixture and environment across baseline and candidate builds.
