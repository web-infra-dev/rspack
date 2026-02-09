# rspack_plugin_swc_js_minimizer

## Role
SWC-based JavaScript minification plugin.

## Perf opportunities
- Cache normalized SWC minifier configs across modules.
- Reuse AST arenas between modules where possible.
- Parallelize minification with bounded concurrency to avoid oversubscription.

## Code pointers
- `crates/rspack_plugin_swc_js_minimizer/**`
