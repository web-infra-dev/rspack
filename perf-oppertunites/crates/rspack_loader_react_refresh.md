# rspack_loader_react_refresh

## Role
React Refresh loader for HMR.

## Perf opportunities
- Ensure loader is only active in dev/watch mode.
- Avoid re-running transforms when inputs are unchanged.
- Cache generated refresh code per module.

## Code pointers
- `crates/rspack_loader_react_refresh/**`
