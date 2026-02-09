# rspack_plugin_hmr

## Role
Hot Module Replacement runtime and hooks.

## Profiling relevance
- Not visible in react-10k; active in dev/watch builds.
- Costs scale with number of updated modules.

## Perf opportunities
- Gate HMR logic strictly to dev/watch builds.
- Cache runtime template fragments by HMR mode.
- Avoid per-module string formatting for HMR metadata.

## Key functions/structs to inspect
- HMR runtime generation in `hot_module_replacement.rs`.
- HMR template in `runtime/hot_module_replacement.ejs`.

## Suggested experiments
- Measure HMR update time with different module counts.
- Compare template caching impact on rebuild latency.

## Code pointers
- `crates/rspack_plugin_hmr/Cargo.toml`
- `crates/rspack_plugin_hmr/src/lib.rs`
- `crates/rspack_plugin_hmr/src/hot_module_replacement.rs`
- `crates/rspack_plugin_hmr/src/runtime/hot_module_replacement.ejs`
- `crates/rspack_plugin_hmr/**`
