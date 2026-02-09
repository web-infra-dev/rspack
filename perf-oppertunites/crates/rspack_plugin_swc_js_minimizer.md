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

## Suggested experiments
- Profile large builds with/without minification and measure per-file cost.
- Test cache hits on repeated builds with unchanged JS bundles.

## Code pointers
- `crates/rspack_plugin_swc_js_minimizer/**`
