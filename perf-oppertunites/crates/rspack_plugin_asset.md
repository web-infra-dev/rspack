# rspack_plugin_asset

## Role
Asset module handling (resource/inline/source).

## Perf opportunities
- Cache asset source transforms by content hash.
- Avoid re-encoding assets on incremental builds.
- Batch asset emission IO where possible.

## Code pointers
- `crates/rspack_plugin_asset/**`
