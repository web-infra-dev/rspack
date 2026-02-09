# swc_plugin_ts_collector

## Role
SWC plugin that collects TypeScript metadata used by Rspack.

## Perf opportunities
- Cache plugin configuration and derived state per compilation to avoid reparse.
- Minimize AST traversal work; short-circuit when TS metadata is unused.
- Reuse SWC allocator arenas if possible to reduce per-module allocations.

## Code pointers
- `crates/swc_plugin_ts_collector/**`
