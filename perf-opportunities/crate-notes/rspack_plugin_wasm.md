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

## Key functions/structs to inspect
- WASM parser/generator in `parser_and_generator.rs`.
- Loading pipeline in `loading_plugin.rs`.
- `WasmImportDependency` handling (dependency/wasm_import_dependency.rs).
- Runtime glue in `runtime.rs`.

## Suggested experiments
- Profile a WASM-heavy project and measure parse/validate time.
- Test cache effectiveness for unchanged WASM modules.

## Code pointers
- `crates/rspack_plugin_wasm/Cargo.toml`
- `crates/rspack_plugin_wasm/src/lib.rs`
- `crates/rspack_plugin_wasm/src/loading_plugin.rs`
- `crates/rspack_plugin_wasm/src/parser_and_generator.rs`
- `crates/rspack_plugin_wasm/src/dependency/wasm_import_dependency.rs`
- `crates/rspack_plugin_wasm/src/runtime.rs`
- `crates/rspack_plugin_wasm/**`
