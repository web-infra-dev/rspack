# rspack_plugin_library

## Role
Library output configuration and wrapper generation.

## Perf opportunities
- Cache library wrapper templates by target/library type.
- Avoid string concatenation in per-module wrappers.
- Skip library formatting when output is not a library target.

## Code pointers
- `crates/rspack_plugin_library/**`
