# rspack_plugin_circular_dependencies

## Role
Detect circular dependencies in module graph.

## Perf opportunities
- Only run when enabled; skip in production builds.
- Cache traversal results between incremental builds.
- Limit depth or scope for large graphs to reduce cost.

## Code pointers
- `crates/rspack_plugin_circular_dependencies/**`
