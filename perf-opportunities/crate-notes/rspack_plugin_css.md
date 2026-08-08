# rspack_plugin_css

## Role
CSS parsing, dependency extraction, and runtime handling.

## Profiling relevance
- Not visible in react-10k perf samples; hot for CSS-heavy apps.
- Costs scale with CSS module count and parsing complexity.

## Perf opportunities
- Cache parsed CSS ASTs for unchanged modules.
- Avoid repeated string conversions during CSS transformations.
- Batch CSS dependency processing to reduce graph churn.

## Key functions/structs to inspect
- CSS parser/generator in `parser_and_generator/mod.rs`.
- Plugin entrypoints in `plugin/mod.rs` (hook registration).
- Dependency implementations in `dependency/*` (compose/import/url).
- Runtime template helpers in `runtime/mod.rs`.

## Suggested experiments
- Profile CSS-heavy builds and measure parsing time.
- Compare cache hit rates for unchanged CSS modules.

## Code pointers
- `crates/rspack_plugin_css/Cargo.toml`
- `crates/rspack_plugin_css/src/lib.rs`
- `crates/rspack_plugin_css/src/plugin/mod.rs`
- `crates/rspack_plugin_css/src/dependency/mod.rs`
- `crates/rspack_plugin_css/src/parser_and_generator/mod.rs`
- `crates/rspack_plugin_css/src/runtime/mod.rs`
- `crates/rspack_plugin_css/**`
