# rspack_plugin_size_limits

## Role
Warnings for asset/entrypoint size limits.

## Perf opportunities
- Avoid computing sizes for assets that are already below thresholds.
- Reuse size calculations from previous passes.
- Gate warnings to only when size limit checks are enabled.

## Code pointers
- `crates/rspack_plugin_size_limits/**`
