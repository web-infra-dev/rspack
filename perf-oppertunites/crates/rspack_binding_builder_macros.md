# rspack_binding_builder_macros

## Role
Procedural macros for binding builder utilities.

## Profiling relevance
- Build-time only; affects generated binding code.
- Indirectly impacts runtime overhead through code shape.

## Perf opportunities
- Ensure generated code avoids redundant conversions.
- Keep macro expansions minimal to reduce compile-time and binary size.

## Suggested experiments
- Inspect generated bindings for redundant conversions.

## Code pointers
- `crates/rspack_binding_builder_macros/**`
