# rspack_plugin_module_info_header

## Role
Inject module info headers into generated output.

## Perf opportunities
- Avoid string formatting per module when disabled.
- Cache header templates and reuse across modules.
- Skip header injection for small modules or when not needed.

## Code pointers
- `crates/rspack_plugin_module_info_header/**`
