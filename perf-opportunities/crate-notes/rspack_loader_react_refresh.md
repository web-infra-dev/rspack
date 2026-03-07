# rspack_loader_react_refresh

## Role
React Refresh loader for HMR.

## Profiling relevance
- Not visible in react-10k; active in dev/watch mode only.
- Costs scale with number of refreshed modules.

## Perf opportunities
- Ensure loader is only active in dev/watch mode.
- Avoid re-running transforms when inputs are unchanged.
- Cache generated refresh code per module.

## Key functions/structs to inspect
- `ReactRefreshLoader::run` (lib.rs).
- `ReactRefreshLoader::with_identifier` (lib.rs).
- Plugin wiring in `plugin.rs`.

## Suggested experiments
- Measure refresh transform overhead in large dev builds.
- Compare cached vs recomputed refresh wrappers.

## Code pointers
- `crates/rspack_loader_react_refresh/Cargo.toml`
- `crates/rspack_loader_react_refresh/src/lib.rs`
- `crates/rspack_loader_react_refresh/src/plugin.rs`
- `crates/rspack_loader_react_refresh/**`
