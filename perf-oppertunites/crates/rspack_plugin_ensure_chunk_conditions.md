# rspack_plugin_ensure_chunk_conditions

## Role
Ensure chunk conditions and constraints are met.

## Profiling relevance
- Not visible in react-10k; hot when constraints are complex.
- Costs scale with chunk graph size.

## Perf opportunities
- Avoid re-evaluating conditions for unchanged chunks.
- Cache condition results per chunk/entry.
- Short-circuit when no constraints are configured.

## Suggested experiments
- Profile builds with many chunk conditions to measure evaluation overhead.
- Compare cached vs full condition evaluation.

## Code pointers
- `crates/rspack_plugin_ensure_chunk_conditions/**`
