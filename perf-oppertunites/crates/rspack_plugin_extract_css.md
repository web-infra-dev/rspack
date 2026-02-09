# rspack_plugin_extract_css

## Role
Extract CSS assets into separate files.

## Perf opportunities
- Cache extracted CSS results for unchanged modules.
- Avoid repeated string concatenation for CSS bundles.
- Batch CSS asset emission to reduce IO overhead.

## Code pointers
- `crates/rspack_plugin_extract_css/**`
