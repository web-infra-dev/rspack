# rspack_cacheable_test

## Role
Tests for cacheable serialization utilities.

## Profiling relevance
- Test-only crate; no runtime impact.
- Ensure isolation from production builds.

## Perf opportunities
- Not runtime hot; keep tests isolated.
- Ensure test-only helpers are not included in production builds.

## Suggested experiments
- Verify test features are gated in release builds.

## Code pointers
- `crates/rspack_cacheable_test/**`
