# rspack_plugin_module_replacement

## Role
Module replacement (aliasing/rewrites) support.

## Profiling relevance
- Not visible in react-10k; hot when many replacements are configured.
- Costs scale with resolver invocation count.

## Perf opportunities
- Cache replacement decisions by request + context.
- Avoid repeated resolver calls for identical replacements.
- Short-circuit when no replacements are configured.

## Suggested experiments
- Profile builds with heavy replacement rules and measure resolver calls.
- Compare cache hit rates for repeated replacement requests.

## Code pointers
- `crates/rspack_plugin_module_replacement/**`
