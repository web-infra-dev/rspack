# rspack_plugin_esm_library

## Role
ESM library output formatting.

## Perf opportunities
- Cache wrapper templates by output config.
- Avoid repeated string concatenation for export maps.
- Skip work when output target is not ESM.

## Code pointers
- `crates/rspack_plugin_esm_library/**`
