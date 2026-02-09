# rspack_plugin_split_chunks

## Role
SplitChunks optimization (chunk grouping and splitting).

## Perf opportunities
- Cache group computations by module/chunk signature.
- Avoid full graph scans when cache groups are unchanged.
- Parallelize heavy grouping passes with bounded concurrency.

## Code pointers
- `crates/rspack_plugin_split_chunks/**`
