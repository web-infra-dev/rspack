# swc_plugin_ts_collector

## Role
SWC plugin that collects TypeScript metadata used by Rspack.

## Profiling relevance
- Not directly visible in react-10k perf samples (depends on TS usage).
- Hot when TS projects enable metadata collection for analysis.

## Perf opportunities
- Cache plugin configuration and derived state per compilation to avoid reparse.
- Minimize AST traversal work; short-circuit when TS metadata is unused.
- Reuse SWC allocator arenas if possible to reduce per-module allocations.

## Key functions/structs to inspect
- `ExportedEnumCollector::new` and `ExportedEnumCollector::collect` (enums.rs).
- `ExportedEnumCollector::evaluate_expr` / `evaluate_bin` (enums.rs).
- `TypeExportsCollector::new` and `TypeExportsCollector::visit_program` (type_exports.rs).

## Suggested experiments
- Profile TS-heavy cases with and without metadata collection enabled.
- Measure AST traversal time vs. cached results per module.

## Code pointers
- `crates/swc_plugin_ts_collector/Cargo.toml`
- `crates/swc_plugin_ts_collector/src/lib.rs`
- `crates/swc_plugin_ts_collector/src/enums.rs`
- `crates/swc_plugin_ts_collector/src/type_exports.rs`
- `crates/swc_plugin_ts_collector/**`
