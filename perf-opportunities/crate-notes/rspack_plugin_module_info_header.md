# rspack_plugin_module_info_header

## Role
Inject module info headers into generated output.

## Profiling relevance
- Not visible in react-10k; overhead when enabled on large module graphs.
- Costs scale with number of modules and header complexity.

## Perf opportunities
- Avoid string formatting per module when disabled.
- Cache header templates and reuse across modules.
- Skip header injection for small modules or when not needed.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- Header injection logic in `src/lib.rs`.

## Suggested experiments
- Measure header injection cost on large builds.
- Compare cached template vs per-module formatting overhead.

## Code pointers
- `crates/rspack_plugin_module_info_header/Cargo.toml`
- `crates/rspack_plugin_module_info_header/src/lib.rs`
- `crates/rspack_plugin_module_info_header/**`
