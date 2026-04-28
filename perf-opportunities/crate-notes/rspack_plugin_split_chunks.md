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

## Key functions/structs to inspect
- Cache group definitions (`options/cache_group.rs`).
- Grouping logic in `module_group.rs`.
- Split passes in `plugin/mod.rs` and `plugin/min_size.rs` / `max_size.rs`.

## Suggested experiments
- Profile builds with large code splitting configs and track split pass time.
- Compare cached group results across incremental builds.

## Code pointers
- `crates/rspack_plugin_split_chunks/Cargo.toml`
- `crates/rspack_plugin_split_chunks/src/lib.rs`
- `crates/rspack_plugin_split_chunks/src/common.rs`
- `crates/rspack_plugin_split_chunks/src/module_group.rs`
- `crates/rspack_plugin_split_chunks/src/options/cache_group.rs`
- `crates/rspack_plugin_split_chunks/src/plugin/mod.rs`
- `crates/rspack_plugin_split_chunks/**`
