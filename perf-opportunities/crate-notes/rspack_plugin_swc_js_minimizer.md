# rspack_plugin_swc_js_minimizer

## Role
SWC-based JavaScript minification plugin.

## Profiling relevance
- Appears in extended perf samples via SWC minifier calls.
- Becomes a major cost in production builds with minification enabled.

## Perf opportunities
- Cache normalized SWC minifier configs across modules.
- Reuse AST arenas between modules where possible.
- Parallelize minification with bounded concurrency to avoid oversubscription.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- `SwcJsMinimizerRspackPlugin::new` (lib.rs) and plugin initialization.
- `process_assets` hook (lib.rs) — main minification loop.
- `js_chunk_hash` hook (lib.rs) — hash integration.
- `MinimizerOptions::hash` (lib.rs) — config hashing + caching.

## Suggested experiments
- Profile large builds with/without minification and measure per-file cost.
- Test cache hits on repeated builds with unchanged JS bundles.

## Code pointers
- `crates/rspack_plugin_swc_js_minimizer/Cargo.toml`
- `crates/rspack_plugin_swc_js_minimizer/src/lib.rs`
- `crates/rspack_plugin_swc_js_minimizer/**`
