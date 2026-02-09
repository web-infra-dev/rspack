# rspack_tasks

## Role
Task scheduling primitives for compilation work distribution.

## Profiling relevance
- Indirectly visible through task scheduling overhead.
- Critical when many small async jobs are spawned (codegen, hashing).

## Perf opportunities
- Reduce per-task allocations by reusing job structures.
- Bound concurrency to avoid oversubscription and context switching.
- Batch small tasks into larger units to reduce scheduling overhead.
- Single-file crate: concentrate profiling on `src/lib.rs` task primitives.

## Suggested experiments
- Measure task count and overhead during codegen/hash passes.
- Test batching of small tasks in a large module graph.

## Code pointers
- `crates/rspack_tasks/Cargo.toml`
- `crates/rspack_tasks/src/lib.rs`
- `crates/rspack_tasks/**`
