# rspack_cacheable_macros

## Role
Procedural macros for cacheable serialization.

## Profiling relevance
- Not runtime hot; impacts generated serialization code.
- Indirectly affects runtime allocation patterns.

## Perf opportunities
- Ensure generated code avoids intermediate allocations.
- Prefer compile-time constants for cache keys.
- Keep macro expansions minimal to reduce binary size.

## Suggested experiments
- Inspect generated serializers for unnecessary allocations.
- Compare serialized payload size with optimized macros.

## Code pointers
- `crates/rspack_cacheable_macros/**`
