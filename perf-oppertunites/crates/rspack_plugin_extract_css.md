# rspack_plugin_extract_css

## Role
Extract CSS assets into separate files.

## Profiling relevance
- Not visible in react-10k samples; hot in production builds with CSS extraction.
- Costs scale with number of CSS modules and asset size.

## Perf opportunities
- Cache extracted CSS results for unchanged modules.
- Avoid repeated string concatenation for CSS bundles.
- Batch CSS asset emission to reduce IO overhead.

## Suggested experiments
- Profile CSS-heavy builds with extraction enabled/disabled.
- Measure cache hit rates for unchanged CSS assets.

## Code pointers
- `crates/rspack_plugin_extract_css/Cargo.toml`
- `crates/rspack_plugin_extract_css/src/lib.rs`
- `crates/rspack_plugin_extract_css/src/css_dependency.rs`
- `crates/rspack_plugin_extract_css/src/plugin.rs`
- `crates/rspack_plugin_extract_css/src/runtime.rs`
- `crates/rspack_plugin_extract_css/**`
