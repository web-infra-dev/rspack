# rspack_plugin_web_worker_template

## Role
Template generation for web worker runtime.

## Perf opportunities
- Cache rendered templates for identical options/runtime.
- Avoid repeated string concatenation by preallocating buffers.
- Short-circuit when worker templates are not used.

## Code pointers
- `crates/rspack_plugin_web_worker_template/**`
