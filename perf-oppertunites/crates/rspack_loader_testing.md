# rspack_loader_testing

## Role
Loader testing utilities.

## Profiling relevance
- Not runtime hot; test-only utilities.
- Ensure no production build impact.

## Perf opportunities
- Not runtime hot; ensure utilities are not pulled into production builds.
- Avoid heavy fixture loading unless explicitly used.
- Single-file crate: concentrate profiling on `src/lib.rs` test helpers.

## Key functions/structs to inspect
- Test harness helpers in `src/lib.rs`.

## Suggested experiments
- Verify test helpers are not compiled in release builds.

## Code pointers
- `crates/rspack_loader_testing/Cargo.toml`
- `crates/rspack_loader_testing/src/lib.rs`
- `crates/rspack_loader_testing/**`
