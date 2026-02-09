# rspack_plugin_css

## Role
CSS parsing, dependency extraction, and runtime handling.

## Perf opportunities
- Cache parsed CSS ASTs for unchanged modules.
- Avoid repeated string conversions during CSS transformations.
- Batch CSS dependency processing to reduce graph churn.

## Code pointers
- `crates/rspack_plugin_css/**`
