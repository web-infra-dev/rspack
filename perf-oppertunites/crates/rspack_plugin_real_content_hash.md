# rspack_plugin_real_content_hash

## Role
Compute content hashes based on emitted asset content.

## Profiling relevance
- Not visible in react-10k samples; hot when real content hash enabled.
- Costs scale with asset count and size.

## Perf opportunities
- Cache content hashes for unchanged assets.
- Hash assets in parallel with bounded concurrency.
- Avoid re-reading asset sources when already in memory.

## Suggested experiments
- Compare hashing time with and without asset hash caching.
- Profile large asset outputs to measure parallel hash scaling.

## Code pointers
- `crates/rspack_plugin_real_content_hash/Cargo.toml`
- `crates/rspack_plugin_real_content_hash/src/lib.rs`
- `crates/rspack_plugin_real_content_hash/**`
