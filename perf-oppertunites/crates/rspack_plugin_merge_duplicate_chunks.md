# rspack_plugin_merge_duplicate_chunks

## Role
Merge duplicate chunks to reduce output size.

## Profiling relevance
- Not visible in react-10k; hot when chunk graphs are large and similar.
- Cost scales with chunk comparison strategy.

## Perf opportunities
- Use chunk fingerprints to avoid deep comparisons.
- Skip merge pass when chunk graph is unchanged.
- Batch graph mutations to reduce overhead.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Suggested experiments
- Profile large split-chunk builds and measure merge pass time.
- Compare fingerprint-based vs deep comparison strategies.

## Code pointers
- `crates/rspack_plugin_merge_duplicate_chunks/Cargo.toml`
- `crates/rspack_plugin_merge_duplicate_chunks/src/lib.rs`
- `crates/rspack_plugin_merge_duplicate_chunks/**`
