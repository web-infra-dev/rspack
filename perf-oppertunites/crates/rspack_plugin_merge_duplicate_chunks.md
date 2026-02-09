# rspack_plugin_merge_duplicate_chunks

## Role
Merge duplicate chunks to reduce output size.

## Perf opportunities
- Use chunk fingerprints to avoid deep comparisons.
- Skip merge pass when chunk graph is unchanged.
- Batch graph mutations to reduce overhead.

## Code pointers
- `crates/rspack_plugin_merge_duplicate_chunks/**`
