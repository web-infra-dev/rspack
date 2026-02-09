# rspack_plugin_runtime_chunk

## Role
Runtime chunk extraction and configuration.

## Perf opportunities
- Avoid repeated runtime chunk decisions across modules.
- Cache runtime chunk names and template outputs.
- Skip runtime chunk work when output uses a single chunk.

## Code pointers
- `crates/rspack_plugin_runtime_chunk/**`
