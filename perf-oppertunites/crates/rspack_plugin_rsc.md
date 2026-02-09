# rspack_plugin_rsc

## Role
React Server Components integration.

## Perf opportunities
- Cache RSC manifest generation for unchanged modules.
- Avoid repeated traversal of the same component graph.
- Gate RSC-specific work behind option checks to avoid overhead in non-RSC builds.

## Code pointers
- `crates/rspack_plugin_rsc/**`
