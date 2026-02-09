# rspack_core

## Role
Core compilation engine (module graph, chunking, codegen, hashing).

## Perf opportunities
- Reduce allocation pressure in module graph updates and codegen jobs.
- Optimize overlay map lookups and export prefetch caching.
- Re-enable incremental chunk graph with correctness guardrails.

## Code pointers
- `crates/rspack_core/src/compilation/**`
- `crates/rspack_core/src/module_graph/**`
