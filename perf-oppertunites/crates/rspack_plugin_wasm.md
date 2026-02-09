# rspack_plugin_wasm

## Role
WASM module support and runtime integration.

## Profiling relevance
- Not visible in react-10k; hot when WASM modules are present.
- Heavy cost for large WASM binaries or many modules.

## Perf opportunities
- Cache WASM module parsing and metadata extraction.
- Avoid repeated hashing of WASM binaries; reuse content hash.
- Parallelize WASM validation for large modules with bounded concurrency.

## Suggested experiments
- Profile a WASM-heavy project and measure parse/validate time.
- Test cache effectiveness for unchanged WASM modules.

## Code pointers
- `crates/rspack_plugin_wasm/**`
