# rspack_futures

## Role
Async helpers and scoped task utilities.

## Perf opportunities
- Reduce per-task allocations by reusing job structs.
- Avoid oversubscription by bounding concurrency.
- Batch small tasks to reduce scheduling overhead.

## Code pointers
- `crates/rspack_futures/**`
