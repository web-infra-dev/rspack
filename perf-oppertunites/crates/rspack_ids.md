# rspack_ids

## Role
ID generation and management for modules/chunks.

## Profiling relevance
- Not visible in react-10k; hot when many modules/chunks are created.
- Costs scale with hashing and ID assignment strategy.

## Perf opportunities
- Cache derived IDs to avoid repeated hashing.
- Use incremental ID assignment where possible.
- Avoid string allocations when IDs are numeric.

## Suggested experiments
- Measure ID assignment time in large module graphs.
- Compare hashed vs incremental ID strategies.

## Code pointers
- `crates/rspack_ids/**`
