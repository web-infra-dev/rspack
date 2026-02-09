# rspack_macros_test

## Role
Tests for procedural macro expansion behavior.

## Perf opportunities
- Not runtime hot; keep tests isolated to avoid impacting build performance.
- Ensure test helpers are not included in production builds.

## Code pointers
- `crates/rspack_macros_test/**`
