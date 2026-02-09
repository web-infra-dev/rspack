# rspack_hook

## Role
Hook system definitions and macros.

## Perf opportunities
- Add fast paths when no taps are registered.
- Avoid building tap lists for empty hooks.
- Reduce allocations in hook argument preparation.

## Code pointers
- `crates/rspack_hook/**`
