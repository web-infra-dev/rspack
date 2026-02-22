# rspack_binding_builder

## Role
Binding builder utilities for Node integration.

## Profiling relevance
- Build-time only; no runtime impact.
- Ensure builders are not executed in production builds.

## Perf opportunities
- Not runtime hot; ensure build helpers are not included in runtime paths.
- Keep generated bindings minimal to reduce JS↔Rust overhead.
- Single-file crate: concentrate profiling on `src/lib.rs` build helpers.

## Key functions/structs to inspect
- `register_custom_plugin` re-export (lib.rs) — see binding_api for implementation.
- `CustomPluginBuilder` re-export (lib.rs).

## Suggested experiments
- Validate binding builder execution frequency in CI pipelines.

## Code pointers
- `crates/rspack_binding_builder/Cargo.toml`
- `crates/rspack_binding_builder/src/lib.rs`
- `crates/rspack_binding_builder/**`
