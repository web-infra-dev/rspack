# rspack_plugin_remove_empty_chunks

## Role
Remove empty chunks from the chunk graph after optimization.

## Perf opportunities
- Track emptiness incrementally to avoid full graph scans.
- Batch removals to reduce chunk graph mutation overhead.
- Avoid recomputation when chunk graph is unchanged.

## Code pointers
- `crates/rspack_plugin_remove_empty_chunks/**`
