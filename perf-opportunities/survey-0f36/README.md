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
- [hotspot-map.md](./hotspot-map.md) — perf symbols mapped to code.
- [crate-survey.md](./crate-survey.md) — crate-by-crate inspection notes.
- [../crate-notes/](../crate-notes/) — per-crate performance opportunity write-ups.
- [../crate-notes/004-all-crates-detailed-opportunities.md](../crate-notes/004-all-crates-detailed-opportunities.md) — consolidated detailed opportunity ledger for all crates.
- [../crate-notes/000-deep-audit-and-profiling-status.md](../crate-notes/000-deep-audit-and-profiling-status.md) — latest deep audit + profiling artifact status.
- [../crate-notes/001-all-crates-remaining-opportunities.md](../crate-notes/001-all-crates-remaining-opportunities.md) — one-by-one remaining opportunities for all crates.
- [../crate-notes/003-file-level-manual-review-coverage-index.md](../crate-notes/003-file-level-manual-review-coverage-index.md) — file-count coverage index across all crates.
- [macos-profiling-deep-research.md](./macos-profiling-deep-research.md) — macOS profiling workflows, constraints, and run evidence.
- [module-graph-and-resolution.md](./module-graph-and-resolution.md)
- [parsing-loaders-and-transforms.md](./parsing-loaders-and-transforms.md)
- [chunking-codegen-and-runtime.md](./chunking-codegen-and-runtime.md)
- [caching-incremental-and-io.md](./caching-incremental-and-io.md)
- [plugin-hooks-and-bindings.md](./plugin-hooks-and-bindings.md)
- [cross-cutting-optimizations.md](./cross-cutting-optimizations.md)

## How to Use This Folder

1. Start with `profiling-results.md` to see the perf evidence.
2. Review `crate-survey.md` to map crates to pipeline stages.
3. Review `../crate-notes/000-deep-audit-and-profiling-status.md` for current profiling feasibility and captured trace artifacts.
4. Use `../crate-notes/001-all-crates-remaining-opportunities.md` as the crate-by-crate optimization backlog.
5. Use `../crate-notes/003-file-level-manual-review-coverage-index.md` to verify crate/file coverage.
6. Dive into the per-stage docs for optimization opportunities, each including
   code pointers and suggested experiments.
