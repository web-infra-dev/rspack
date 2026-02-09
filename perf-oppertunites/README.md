# Rspack Performance Opportunities (react-10k)

This folder contains a crate-by-crate survey, profiling evidence, and a set of
performance opportunity write-ups based on the `build-tools-performance`
`cases/react-10k` workload.

## Scope

- **Workload:** `build-tools-performance/cases/react-10k`
- **Profiling:** perf (DWARF call stacks) + flat hotspot summaries
- **Codebase:** Rust crates under `crates/` with emphasis on the compilation
  pipeline (module graph, resolver, loaders, codegen, chunking, caching)

## Key Observations (high level)

- **Allocation pressure dominates** in the sampled run (mimalloc + kernel page
  zeroing). This signals a need to reduce transient allocations in hot paths.
- **Module graph overlay lookups** (`OverlayMap::get`) show up in top samples.
  This suggests work in incremental/rollback paths may be heavier than expected.
- **Lossy UTF-8 conversions** surface in samples, pointing at buffer → string
  conversions during loader/content handling.
- **Incremental chunk graph** is currently disabled; re-enabling should be a
  high‑impact opportunity once correctness is validated.

## Documents

- [workload-react-10k.md](./workload-react-10k.md) — setup/runbook for the case.
- [profiling-results.md](./profiling-results.md) — perf output and notes.
- [prioritized-opportunities.md](./prioritized-opportunities.md) — top opportunities, ordered by impact.
- [crate-survey.md](./crate-survey.md) — crate-by-crate inspection notes.
- [module-graph-and-resolution.md](./module-graph-and-resolution.md)
- [parsing-loaders-and-transforms.md](./parsing-loaders-and-transforms.md)
- [chunking-codegen-and-runtime.md](./chunking-codegen-and-runtime.md)
- [caching-incremental-and-io.md](./caching-incremental-and-io.md)
- [plugin-hooks-and-bindings.md](./plugin-hooks-and-bindings.md)
- [cross-cutting-optimizations.md](./cross-cutting-optimizations.md)

## How to Use This Folder

1. Start with `profiling-results.md` to see the perf evidence.
2. Review `crate-survey.md` to map crates to pipeline stages.
3. Dive into the per-stage docs for optimization opportunities, each including
   code pointers and suggested experiments.
