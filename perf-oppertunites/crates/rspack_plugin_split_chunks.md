# rspack_plugin_split_chunks

## Role
SplitChunks optimization (chunk grouping and splitting).

## Profiling relevance
- Chunk graph operations are visible in extended perf samples (`CodeSplitter::prepare`).
- Hot for large module graphs with many chunk candidates.

## Perf opportunities
- Cache group computations by module/chunk signature.
- Avoid full graph scans when cache groups are unchanged.
- Parallelize heavy grouping passes with bounded concurrency.

## Suggested experiments
- Profile builds with large code splitting configs and track split pass time.
- Compare cached group results across incremental builds.

## Code pointers
- `crates/rspack_plugin_split_chunks/Cargo.toml`
- `crates/rspack_plugin_split_chunks/src/lib.rs`
- `crates/rspack_plugin_split_chunks/**`
