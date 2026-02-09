# rspack_macros_test

## Role
Tests for procedural macro expansion behavior.

## Profiling relevance
- Not runtime hot; test-only crate.
- Ensure tests remain isolated from production builds.

## Perf opportunities
- Not runtime hot; keep tests isolated to avoid impacting build performance.
- Ensure test helpers are not included in production builds.

## Suggested experiments
- Verify test-only features are gated and not compiled in release builds.

## Code pointers
- `crates/rspack_macros_test/Cargo.toml`
- `crates/rspack_macros_test/tests/compiletest.rs`
- `crates/rspack_macros_test/**`
