# rspack_plugin_limit_chunk_count

## Role
Limit chunk count by merging or adjusting chunk graph.

## Profiling relevance
- Not visible in react-10k; hot when aggressive chunk limits are enabled.
- Costs scale with chunk graph size and merge operations.

## Perf opportunities
- Avoid full graph scans when chunk count is already below threshold.
- Use heuristics to reduce repeated merge attempts.
- Cache merge results for incremental builds.

## Key functions/structs to inspect
- `ChunkCombinationBucket::sort_combinations` (chunk_combination.rs).
- Chunk merge decision loop in `lib.rs`.

## Suggested experiments
- Profile builds with tight chunk limits to measure merge overhead.
- Compare cached vs non-cached merge behavior across rebuilds.

## Code pointers
- `crates/rspack_plugin_limit_chunk_count/Cargo.toml`
- `crates/rspack_plugin_limit_chunk_count/src/lib.rs`
- `crates/rspack_plugin_limit_chunk_count/src/chunk_combination.rs`
- `crates/rspack_plugin_limit_chunk_count/**`
