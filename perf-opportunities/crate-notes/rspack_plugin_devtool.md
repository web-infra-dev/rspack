# rspack_plugin_devtool

## Role
Source map/devtool handling.

## Profiling relevance
- Not visible in react-10k; source map generation can be expensive when enabled.
- Costs scale with module count and map detail level.

## Perf opportunities
- Avoid generating source maps when not requested.
- Cache source map generation inputs and reuse for unchanged modules.
- Use incremental source map updates where possible.

## Key functions/structs to inspect
- `SourceMapDevToolPlugin` hooks (source_map_dev_tool_plugin.rs).
- Eval devtool variants (eval_dev_tool_module_plugin.rs, eval_source_map_dev_tool_plugin.rs).
- `MappedAssetsCache` (mapped_assets_cache.rs).

## Suggested experiments
- Profile builds with and without source maps enabled.
- Measure incremental source map update effectiveness.

## Code pointers
- `crates/rspack_plugin_devtool/Cargo.toml`
- `crates/rspack_plugin_devtool/src/lib.rs`
- `crates/rspack_plugin_devtool/src/source_map_dev_tool_plugin.rs`
- `crates/rspack_plugin_devtool/src/eval_source_map_dev_tool_plugin.rs`
- `crates/rspack_plugin_devtool/src/mapped_assets_cache.rs`
- `crates/rspack_plugin_devtool/**`
