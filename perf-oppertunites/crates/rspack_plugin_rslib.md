# rspack_plugin_rslib

## Role
Rslib integration for library builds.

## Perf opportunities
- Avoid extra manifest generation when library mode is not active.
- Cache library output templates by configuration.
- Reduce per-module string formatting in library wrappers.

## Code pointers
- `crates/rspack_plugin_rslib/**`
