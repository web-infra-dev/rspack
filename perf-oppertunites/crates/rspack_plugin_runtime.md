# rspack_plugin_runtime

## Role
Runtime module generation and runtime requirements computation.

## Profiling relevance
- Not explicitly visible in flat samples; runtime codegen is part of chunk/asset passes.
- Costs scale with number of runtime variants and chunks.

## Perf opportunities
- Cache runtime module outputs keyed by runtime + feature flags.
- Avoid repeated template string concatenation; preallocate buffers.
- Skip runtime module regeneration when module hashes are unchanged.

## Suggested experiments
- Measure runtime module generation time on multi-runtime builds.
- Compare cached runtime outputs across incremental builds.

## Code pointers
- `crates/rspack_plugin_runtime/**`
