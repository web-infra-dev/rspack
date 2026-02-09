# rspack_binding_builder

## Role
Binding builder utilities for Node integration.

## Perf opportunities
- Not runtime hot; ensure build helpers are not included in runtime paths.
- Keep generated bindings minimal to reduce JS↔Rust overhead.

## Code pointers
- `crates/rspack_binding_builder/**`
