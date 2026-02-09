# rspack_cacheable_macros

## Role
Procedural macros for cacheable serialization.

## Perf opportunities
- Ensure generated code avoids intermediate allocations.
- Prefer compile-time constants for cache keys.
- Keep macro expansions minimal to reduce binary size.

## Code pointers
- `crates/rspack_cacheable_macros/**`
