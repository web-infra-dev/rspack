# rspack_plugin_sri

## Role
Subresource Integrity (SRI) generation for emitted assets.

## Profiling relevance
- Not directly visible in react-10k perf samples; only active when SRI enabled.
- Can become significant for large asset graphs.

## Perf opportunities
- Hash assets in parallel using bounded concurrency.
- Avoid re-hashing unchanged assets by caching content hashes.
- Stream hash computation for large assets to avoid full-buffer loads.

## Key functions/structs to inspect
- `compute_integrity` and `create_hash` (integrity.rs).
- `SubresourceIntegrityHashFunction::try_from` (integrity.rs).
- Asset/HTML integration helpers in `asset.rs` and `html.rs`.

## Suggested experiments
- Enable SRI on a large asset workload and measure hash time vs parallel settings.
- Verify cache hit rates for unchanged assets across rebuilds.

## Code pointers
- `crates/rspack_plugin_sri/Cargo.toml`
- `crates/rspack_plugin_sri/src/lib.rs`
- `crates/rspack_plugin_sri/src/integrity.rs`
- `crates/rspack_plugin_sri/src/html.rs`
- `crates/rspack_plugin_sri/src/runtime.rs`
- `crates/rspack_plugin_sri/**`
