# rspack_plugin_sri

## Role
Subresource Integrity (SRI) generation for emitted assets.

## Perf opportunities
- Hash assets in parallel using bounded concurrency.
- Avoid re-hashing unchanged assets by caching content hashes.
- Stream hash computation for large assets to avoid full-buffer loads.

## Code pointers
- `crates/rspack_plugin_sri/**`
