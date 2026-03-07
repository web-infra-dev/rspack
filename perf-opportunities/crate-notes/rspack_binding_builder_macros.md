# rspack_binding_builder_macros

## Role
Procedural macros for binding builder utilities.

## Profiling relevance
- Build-time only; affects generated binding code.
- Indirectly impacts runtime overhead through code shape.

## Perf opportunities
- Ensure generated code avoids redundant conversions.
- Keep macro expansions minimal to reduce compile-time and binary size.

## Key functions/structs to inspect
- `register_plugin` proc-macro (lib.rs).
- Parsing and expansion in `register_plugin.rs`.

## Suggested experiments
- Inspect generated bindings for redundant conversions.

## Code pointers
- `crates/rspack_binding_builder_macros/Cargo.toml`
- `crates/rspack_binding_builder_macros/src/lib.rs`
- `crates/rspack_binding_builder_macros/src/register_plugin.rs`
- `crates/rspack_binding_builder_macros/**`
