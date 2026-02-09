# rspack_binding_builder_testing

## Role
Testing utilities for binding builder.

## Profiling relevance
- Test-only crate; no runtime impact.
- Ensure isolation from production builds.

## Perf opportunities
- Not runtime hot; keep tests isolated from production paths.
- Avoid heavy fixture loading unless required by tests.
- Single-file crate: concentrate profiling on `src/lib.rs` test helpers.

## Suggested experiments
- Verify testing utilities are gated in release builds.

## Code pointers
- `crates/rspack_binding_builder_testing/Cargo.toml`
- `crates/rspack_binding_builder_testing/src/lib.rs`
- `crates/rspack_binding_builder_testing/**`
