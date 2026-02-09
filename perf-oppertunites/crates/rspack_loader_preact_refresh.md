# rspack_loader_preact_refresh

## Role
Preact Refresh loader for HMR.

## Perf opportunities
- Keep loader gated to dev builds only.
- Cache refresh wrapper code per module.
- Avoid repeated string allocations in transform output.

## Code pointers
- `crates/rspack_loader_preact_refresh/**`
