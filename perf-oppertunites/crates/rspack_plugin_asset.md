# rspack_plugin_asset

## Role
Asset module handling (resource/inline/source).

## Profiling relevance
- Not visible in react-10k; hot when many assets are emitted.
- Costs scale with asset size and transformation type.

## Perf opportunities
- Cache asset source transforms by content hash.
- Avoid re-encoding assets on incremental builds.
- Batch asset emission IO where possible.

## Suggested experiments
- Profile asset-heavy builds and measure transformation time.
- Compare cache hit rates for unchanged asset content.

## Code pointers
- `crates/rspack_plugin_asset/Cargo.toml`
- `crates/rspack_plugin_asset/src/lib.rs`
- `crates/rspack_plugin_asset/**`
