# rspack_futures

## Role
Async helpers and scoped task utilities.

## Profiling relevance
- Task spawning overhead shows up indirectly in codegen/hash passes.
- Costs scale with number of spawned jobs.

## Perf opportunities
- Reduce per-task allocations by reusing job structs.
- Avoid oversubscription by bounding concurrency.
- Batch small tasks to reduce scheduling overhead.

## Suggested experiments
- Measure task counts and overhead during module graph and codegen passes.
- Compare batched vs per-module task spawning.

## Code pointers
- `crates/rspack_futures/Cargo.toml`
- `crates/rspack_futures/src/lib.rs`
- `crates/rspack_futures/**`
