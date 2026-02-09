# rspack_macros

## Role
Procedural macros used across Rspack.

## Perf opportunities
- Ensure generated code avoids unnecessary allocations in hot paths.
- Prefer compile-time constants over runtime initialization.
- Keep macro expansions minimal to reduce binary size.

## Code pointers
- `crates/rspack_macros/**`
