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

## Key functions/structs to inspect
- `cacheable` macro expansion in `cacheable/impl.rs`.
- `cacheable_dyn` and `impl_with` helpers (cacheable_dyn.rs, cacheable/impl_with.rs).

## Suggested experiments
- Inspect generated serializers for unnecessary allocations.
- Compare serialized payload size with optimized macros.

## Code pointers
- `crates/rspack_cacheable_macros/Cargo.toml`
- `crates/rspack_cacheable_macros/src/lib.rs`
- `crates/rspack_cacheable_macros/src/cacheable/mod.rs`
- `crates/rspack_cacheable_macros/src/cacheable/impl.rs`
- `crates/rspack_cacheable_macros/src/cacheable/impl_with.rs`
- `crates/rspack_cacheable_macros/**`
