# rspack_storage

## Role
Storage backend for persistent cache (pack storage, IO bridging).

## Profiling relevance
- Not directly visible in single build profiles; hot for incremental builds with persistent cache.
- IO and serialization costs dominate when cache size grows.

## Perf opportunities
- Batch writes and flush asynchronously to avoid blocking compilation.
- Use scratch buffers to reduce serialization allocations.
- Limit scope enumeration to reduce IO in large caches.

## Suggested experiments
- Compare build times with persistent cache enabled/disabled on large projects.
- Measure write amplification and IO time for different cache sizes.

## Code pointers
- `crates/rspack_storage/**`
