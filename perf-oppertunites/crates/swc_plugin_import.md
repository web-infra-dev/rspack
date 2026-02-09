# swc_plugin_import

## Role
SWC plugin that implements `babel-plugin-import` behavior in Rust.

## Perf opportunities
- Cache resolved import mappings to avoid recomputing per module.
- Avoid repeated string allocations when generating new import paths.
- Short-circuit when module has no matching import patterns.

## Code pointers
- `crates/swc_plugin_import/**`
