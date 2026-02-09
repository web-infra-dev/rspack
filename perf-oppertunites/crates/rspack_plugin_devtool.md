# rspack_plugin_devtool

## Role
Source map/devtool handling.

## Perf opportunities
- Avoid generating source maps when not requested.
- Cache source map generation inputs and reuse for unchanged modules.
- Use incremental source map updates where possible.

## Code pointers
- `crates/rspack_plugin_devtool/**`
