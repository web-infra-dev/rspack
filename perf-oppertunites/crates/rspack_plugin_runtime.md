# rspack_plugin_runtime

## Role
Runtime module generation and runtime requirements computation.

## Perf opportunities
- Cache runtime module outputs keyed by runtime + feature flags.
- Avoid repeated template string concatenation; preallocate buffers.
- Skip runtime module regeneration when module hashes are unchanged.

## Code pointers
- `crates/rspack_plugin_runtime/**`
