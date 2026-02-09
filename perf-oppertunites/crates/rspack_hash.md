# rspack_hash

## Role
Hashing utilities used for modules/chunks/assets.

## Profiling relevance
- Hashing costs show up in chunk/module hash passes.
- Costs scale with module count and asset sizes.

## Perf opportunities
- Cache computed hashes and reuse across passes.
- Avoid hashing when content hash unchanged (use dirty flags).
- Prefer incremental hashing where possible.

## Suggested experiments
- Measure hash time with and without caching on large builds.
- Compare incremental vs full hash strategies.

## Code pointers
- `crates/rspack_hash/**`
