# rspack_plugin_javascript

## Role
JavaScript module parsing, dependency extraction, and runtime hooks.

## Profiling relevance
- Inner-graph processing appears in extended perf samples.
- Costs scale with module count, export analysis, and transform complexity.

## Perf opportunities
- Cache parser options and reuse AST arenas across modules.
- Avoid repeated inner-graph analysis when exports are unchanged.
- Reduce string conversions when generating runtime code.

## Key functions/structs to inspect
- Parser plugin traversal in `parser_plugin/*`.
- `flag_dependency_usage_plugin::collect_active_dependencies` (plugin/flag_dependency_usage_plugin.rs).
- `module_concatenation_plugin::optimize_chunk_modules_impl` (plugin/module_concatenation_plugin.rs).
- URL plugin ReplaceSource handling (`plugin/url_plugin.rs`).

## Suggested experiments
- Profile large JS graphs with inner-graph enabled/disabled.
- Track AST arena reuse impact on allocation pressure.

## Code pointers
- `crates/rspack_plugin_javascript/Cargo.toml`
- `crates/rspack_plugin_javascript/src/lib.rs`
- `crates/rspack_plugin_javascript/src/plugin/mod.rs`
- `crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`
- `crates/rspack_plugin_javascript/src/parser_plugin/mod.rs`
- `crates/rspack_plugin_javascript/src/dependency/esm/mod.rs`
- `crates/rspack_plugin_javascript/src/utils/mangle_exports.rs`
- `crates/rspack_plugin_javascript/**`
