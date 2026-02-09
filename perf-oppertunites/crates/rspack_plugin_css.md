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

## Suggested experiments
- Profile CSS-heavy builds and measure parsing time.
- Compare cache hit rates for unchanged CSS modules.

## Code pointers
- `crates/rspack_plugin_css/**`
