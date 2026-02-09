# rspack_plugin_module_replacement

## Role
Module replacement (aliasing/rewrites) support.

## Perf opportunities
- Cache replacement decisions by request + context.
- Avoid repeated resolver calls for identical replacements.
- Short-circuit when no replacements are configured.

## Code pointers
- `crates/rspack_plugin_module_replacement/**`
