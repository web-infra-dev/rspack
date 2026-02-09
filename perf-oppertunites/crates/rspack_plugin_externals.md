# rspack_plugin_externals

## Role
Externalization of modules based on configuration.

## Perf opportunities
- Cache external resolution decisions by request/context.
- Avoid resolver work when externals match early.
- Batch external checks for repeated specifiers.

## Code pointers
- `crates/rspack_plugin_externals/**`
