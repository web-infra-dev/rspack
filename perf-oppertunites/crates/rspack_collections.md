# rspack_collections

## Role
Custom collection types used in hot paths (sets, maps, identifiers).

## Perf opportunities
- Use cache‑friendly layouts for hot lookups.
- Avoid repeated hashing by caching keys where possible.
- Prefer specialized maps for identifier-heavy workloads.

## Code pointers
- `crates/rspack_collections/**`
