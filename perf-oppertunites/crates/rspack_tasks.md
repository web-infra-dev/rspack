# rspack_tasks

## Role
Task scheduling primitives for compilation work distribution.

## Perf opportunities
- Reduce per-task allocations by reusing job structures.
- Bound concurrency to avoid oversubscription and context switching.
- Batch small tasks into larger units to reduce scheduling overhead.

## Code pointers
- `crates/rspack_tasks/**`
