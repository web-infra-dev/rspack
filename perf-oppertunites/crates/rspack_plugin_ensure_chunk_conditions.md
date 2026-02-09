# rspack_plugin_ensure_chunk_conditions

## Role
Ensure chunk conditions and constraints are met.

## Perf opportunities
- Avoid re-evaluating conditions for unchanged chunks.
- Cache condition results per chunk/entry.
- Short-circuit when no constraints are configured.

## Code pointers
- `crates/rspack_plugin_ensure_chunk_conditions/**`
