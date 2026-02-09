# rspack_ids

## Role
ID generation and management for modules/chunks.

## Perf opportunities
- Cache derived IDs to avoid repeated hashing.
- Use incremental ID assignment where possible.
- Avoid string allocations when IDs are numeric.

## Code pointers
- `crates/rspack_ids/**`
