# rspack_plugin_limit_chunk_count

## Role
Limit chunk count by merging or adjusting chunk graph.

## Perf opportunities
- Avoid full graph scans when chunk count is already below threshold.
- Use heuristics to reduce repeated merge attempts.
- Cache merge results for incremental builds.

## Code pointers
- `crates/rspack_plugin_limit_chunk_count/**`
