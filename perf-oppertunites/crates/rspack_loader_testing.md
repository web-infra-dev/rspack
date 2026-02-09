# rspack_loader_testing

## Role
Loader testing utilities.

## Perf opportunities
- Not runtime hot; ensure utilities are not pulled into production builds.
- Avoid heavy fixture loading unless explicitly used.

## Code pointers
- `crates/rspack_loader_testing/**`
