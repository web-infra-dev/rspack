# rspack_collections

## Role
Custom collection types used in hot paths (sets, maps, identifiers).

## Profiling relevance
- Indirectly visible via identifier maps and hash insertions.
- Hot when module graph and resolver perform many lookups.

## Perf opportunities
- Use cache‑friendly layouts for hot lookups.
- Avoid repeated hashing by caching keys where possible.
- Prefer specialized maps for identifier-heavy workloads.

## Suggested experiments
- Benchmark custom collections with realistic module graph sizes.
- Compare `FxHash`/custom hashers for identifier-heavy workloads.

## Code pointers
- `crates/rspack_collections/**`
