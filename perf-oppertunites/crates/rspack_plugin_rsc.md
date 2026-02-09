# rspack_plugin_rsc

## Role
React Server Components integration.

## Profiling relevance
- Not visible in react-10k; hot when RSC enabled.
- Costs scale with number of server/client component boundaries.

## Perf opportunities
- Cache RSC manifest generation for unchanged modules.
- Avoid repeated traversal of the same component graph.
- Gate RSC-specific work behind option checks to avoid overhead in non-RSC builds.

## Suggested experiments
- Profile RSC builds and measure manifest generation time.
- Validate cache hits for unchanged component graphs.

## Code pointers
- `crates/rspack_plugin_rsc/**`
