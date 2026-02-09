# rspack_plugin_javascript

## Role
JavaScript module parsing, dependency extraction, and runtime hooks.

## Perf opportunities
- Cache parser options and reuse AST arenas across modules.
- Avoid repeated inner-graph analysis when exports are unchanged.
- Reduce string conversions when generating runtime code.

## Code pointers
- `crates/rspack_plugin_javascript/**`
