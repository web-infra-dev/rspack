# rspack_plugin_html

## Role
HTML generation plugin (template rendering and asset injection).

## Perf opportunities
- Cache rendered templates by configuration.
- Avoid repeated asset list string building when unchanged.
- Use streaming template rendering for large HTML outputs.

## Code pointers
- `crates/rspack_plugin_html/**`
