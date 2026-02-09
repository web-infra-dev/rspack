# rspack_plugin_esm_library

## Role
ESM library output formatting.

## Profiling relevance
- Not visible in react-10k; hot for library builds targeting ESM.
- Costs scale with export map generation and wrapper formatting.

## Perf opportunities
- Cache wrapper templates by output config.
- Avoid repeated string concatenation for export maps.
- Skip work when output target is not ESM.

## Suggested experiments
- Profile ESM library builds with large export surfaces.
- Compare cached template reuse across incremental builds.

## Code pointers
- `crates/rspack_plugin_esm_library/Cargo.toml`
- `crates/rspack_plugin_esm_library/src/lib.rs`
- `crates/rspack_plugin_esm_library/**`
