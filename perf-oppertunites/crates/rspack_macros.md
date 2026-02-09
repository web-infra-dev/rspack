# rspack_macros

## Role
Procedural macros used across Rspack.

## Profiling relevance
- Not runtime hot; affects generated code quality and binary size.
- Indirectly impacts runtime overhead through macro expansions.

## Perf opportunities
- Ensure generated code avoids unnecessary allocations in hot paths.
- Prefer compile-time constants over runtime initialization.
- Keep macro expansions minimal to reduce binary size.

## Suggested experiments
- Inspect macro expansions for hot path allocations.
- Compare binary size with optimized macro expansions.

## Code pointers
- `crates/rspack_macros/Cargo.toml`
- `crates/rspack_macros/src/lib.rs`
- `crates/rspack_macros/src/hook.rs`
- `crates/rspack_macros/src/plugin.rs`
- `crates/rspack_macros/src/runtime_module.rs`
- `crates/rspack_macros/**`
