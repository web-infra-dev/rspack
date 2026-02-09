# rspack_browserslist

## Role
Browserslist configuration parsing and resolution.

## Perf opportunities
- Cache parsed browserslist results per config.
- Avoid re-reading config files when unchanged.
- Reduce string allocations in version parsing.

## Code pointers
- `crates/rspack_browserslist/**`
