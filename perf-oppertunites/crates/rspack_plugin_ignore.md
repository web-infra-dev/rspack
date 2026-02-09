# rspack_plugin_ignore

## Role
Ignore plugin for excluding modules based on patterns.

## Perf opportunities
- Precompile ignore patterns and reuse across builds.
- Short-circuit resolution when ignore hits early.
- Avoid allocating new regex objects per module.

## Code pointers
- `crates/rspack_plugin_ignore/**`
