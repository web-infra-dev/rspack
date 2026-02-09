# rspack_browserslist

## Role
Browserslist configuration parsing and resolution.

## Profiling relevance
- Not visible in react-10k; hot when browserslist parsing is repeated.
- Costs scale with config size and frequency of parsing.

## Perf opportunities
- Cache parsed browserslist results per config.
- Avoid re-reading config files when unchanged.
- Reduce string allocations in version parsing.

## Suggested experiments
- Profile builds with frequent browserslist lookups.
- Compare cached vs uncached parsing on large configs.

## Code pointers
- `crates/rspack_browserslist/Cargo.toml`
- `crates/rspack_browserslist/src/lib.rs`
- `crates/rspack_browserslist/src/load_config.rs`
- `crates/rspack_browserslist/src/lightningcss.rs`
- `crates/rspack_browserslist/**`
