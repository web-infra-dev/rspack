# rspack_plugin_circular_dependencies

## Role
Detect circular dependencies in module graph.

## Profiling relevance
- Not visible in react-10k; hot when detection is enabled.
- Cost scales with module graph size and traversal depth.

## Perf opportunities
- Only run when enabled; skip in production builds.
- Cache traversal results between incremental builds.
- Limit depth or scope for large graphs to reduce cost.

## Key functions/structs to inspect
- Circular dependency traversal in `src/lib.rs`.

## Suggested experiments
- Profile circular dependency detection on large graphs.
- Compare full traversal vs incremental caching.

## Code pointers
- `crates/rspack_plugin_circular_dependencies/Cargo.toml`
- `crates/rspack_plugin_circular_dependencies/src/lib.rs`
- `crates/rspack_plugin_circular_dependencies/**`
