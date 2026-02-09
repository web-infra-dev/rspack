# rspack_plugin_progress

## Role
Progress reporting during compilation.

## Perf opportunities
- Throttle progress updates to reduce log overhead.
- Avoid formatting strings for every module in large builds.
- Disable progress hooks when not explicitly enabled.

## Code pointers
- `crates/rspack_plugin_progress/**`
