# rspack_napi_macros

## Role
Procedural macros for NAPI bindings.

## Profiling relevance
- Not runtime hot; affects generated binding code.
- Indirectly influences JS↔Rust overhead through code shape.

## Perf opportunities
- Prefer zero-copy buffer types in generated bindings.
- Avoid redundant conversions in macro-generated code.
- Keep bindings minimal to reduce JS↔Rust overhead.

## Suggested experiments
- Inspect generated binding code for redundant conversions.
- Compare NAPI call overhead with simplified bindings.

## Code pointers
- `crates/rspack_napi_macros/**`
