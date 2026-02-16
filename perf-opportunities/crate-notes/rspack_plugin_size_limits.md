# rspack_plugin_size_limits

## Role
Warnings for asset/entrypoint size limits.

## Profiling relevance
- Not visible in perf samples; only active when size limit checks are enabled.
- Costs scale with number of assets and entrypoints.

## Perf opportunities
- Avoid computing sizes for assets that are already below thresholds.
- Reuse size calculations from previous passes.
- Gate warnings to only when size limit checks are enabled.
- Single-file crate: concentrate profiling on `src/lib.rs` hook implementations.

## Key functions/structs to inspect
- Size limit checks in `src/lib.rs` (warning emission and thresholds).

## Suggested experiments
- Compare builds with size limit checks enabled vs disabled.
- Measure asset size calculation overhead in large outputs.

## Code pointers
- `crates/rspack_plugin_size_limits/Cargo.toml`
- `crates/rspack_plugin_size_limits/src/lib.rs`
- `crates/rspack_plugin_size_limits/**`
