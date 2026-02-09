# rspack_plugin_progress

## Role
Progress reporting during compilation.

## Profiling relevance
- Not visible in perf samples; can add overhead in large builds if verbose.
- Costs scale with module count and update frequency.

## Perf opportunities
- Throttle progress updates to reduce log overhead.
- Avoid formatting strings for every module in large builds.
- Disable progress hooks when not explicitly enabled.

## Suggested experiments
- Measure build time difference with progress enabled vs disabled.
- Adjust update interval and compare overhead.

## Code pointers
- `crates/rspack_plugin_progress/**`
