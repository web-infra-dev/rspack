# rspack_cacheable_test

## Role
Tests for cacheable serialization utilities.

## Perf opportunities
- Not runtime hot; keep tests isolated.
- Ensure test-only helpers are not included in production builds.

## Code pointers
- `crates/rspack_cacheable_test/**`
