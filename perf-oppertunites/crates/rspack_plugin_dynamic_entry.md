# rspack_plugin_dynamic_entry

## Role
Dynamic entry handling for runtime-determined entrypoints.

## Profiling relevance
- Not visible in react-10k; hot when entries are computed at runtime.
- Costs scale with number of dynamic entries and resolution depth.

## Perf opportunities
- Cache resolved dynamic entries and avoid repeated resolution.
- Avoid rebuilding entry lists when only non-entry modules change.
- Batch dynamic entry evaluation to reduce hook overhead.

## Suggested experiments
- Profile dynamic-entry builds and measure resolution overhead.
- Compare cache hit rates for dynamic entry evaluation.

## Code pointers
- `crates/rspack_plugin_dynamic_entry/**`
