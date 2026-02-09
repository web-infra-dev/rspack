# rspack_plugin_no_emit_on_errors

## Role
Suppress asset emission when compilation errors are present.

## Perf opportunities
- Keep checks lightweight; avoid scanning full diagnostics.
- Reuse cached error flags from earlier passes.
- Avoid allocations when no errors are present.

## Code pointers
- `crates/rspack_plugin_no_emit_on_errors/**`
