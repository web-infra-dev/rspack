# rspack_plugin_real_content_hash

## Role
Compute content hashes based on emitted asset content.

## Perf opportunities
- Cache content hashes for unchanged assets.
- Hash assets in parallel with bounded concurrency.
- Avoid re-reading asset sources when already in memory.

## Code pointers
- `crates/rspack_plugin_real_content_hash/**`
