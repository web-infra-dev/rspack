# rspack_cacheable_test

## Role
Tests for cacheable serialization utilities.

## Profiling relevance
- Test-only crate; no runtime impact.
- Ensure isolation from production builds.

## Perf opportunities
- Not runtime hot; keep tests isolated.
- Ensure test-only helpers are not included in production builds.

## Key functions/structs to inspect
- Macro test cases in `tests/macro/*`.
- Utility tests in `tests/utils/*`.

## Suggested experiments
- Verify test features are gated in release builds.

## Code pointers
- `crates/rspack_cacheable_test/Cargo.toml`
- `crates/rspack_cacheable_test/tests/context.rs`
- `crates/rspack_cacheable_test/tests/macro/mod.rs`
- `crates/rspack_cacheable_test/tests/utils/mod.rs`
- `crates/rspack_cacheable_test/**`
