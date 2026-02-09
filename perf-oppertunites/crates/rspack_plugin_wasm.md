# rspack_plugin_wasm

## Role
WASM module support and runtime integration.

## Perf opportunities
- Cache WASM module parsing and metadata extraction.
- Avoid repeated hashing of WASM binaries; reuse content hash.
- Parallelize WASM validation for large modules with bounded concurrency.

## Code pointers
- `crates/rspack_plugin_wasm/**`
