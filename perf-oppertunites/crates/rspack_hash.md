# rspack_hash

## Role
Hashing utilities used for modules/chunks/assets.

## Perf opportunities
- Cache computed hashes and reuse across passes.
- Avoid hashing when content hash unchanged (use dirty flags).
- Prefer incremental hashing where possible.

## Code pointers
- `crates/rspack_hash/**`
