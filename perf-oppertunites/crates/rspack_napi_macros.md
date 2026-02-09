# rspack_napi_macros

## Role
Procedural macros for NAPI bindings.

## Perf opportunities
- Prefer zero-copy buffer types in generated bindings.
- Avoid redundant conversions in macro-generated code.
- Keep bindings minimal to reduce JS↔Rust overhead.

## Code pointers
- `crates/rspack_napi_macros/**`
