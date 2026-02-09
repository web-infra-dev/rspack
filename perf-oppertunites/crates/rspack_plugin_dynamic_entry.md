# rspack_plugin_dynamic_entry

## Role
Dynamic entry handling for runtime-determined entrypoints.

## Perf opportunities
- Cache resolved dynamic entries and avoid repeated resolution.
- Avoid rebuilding entry lists when only non-entry modules change.
- Batch dynamic entry evaluation to reduce hook overhead.

## Code pointers
- `crates/rspack_plugin_dynamic_entry/**`
