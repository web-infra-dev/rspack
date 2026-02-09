# rspack_loader_preact_refresh

## Role
Preact Refresh loader for HMR.

## Profiling relevance
- Not visible in react-10k; active in dev/watch mode only.
- Costs scale with number of refreshed modules.

## Perf opportunities
- Keep loader gated to dev builds only.
- Cache refresh wrapper code per module.
- Avoid repeated string allocations in transform output.

## Suggested experiments
- Measure refresh transform overhead in large dev builds.
- Compare cached vs recomputed refresh wrappers.

## Code pointers
- `crates/rspack_loader_preact_refresh/Cargo.toml`
- `crates/rspack_loader_preact_refresh/src/lib.rs`
- `crates/rspack_loader_preact_refresh/src/plugin.rs`
- `crates/rspack_loader_preact_refresh/src/runtime.js`
- `crates/rspack_loader_preact_refresh/**`
