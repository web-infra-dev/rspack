# rspack_storage

## Role
Storage backend for persistent cache (pack storage, IO bridging).

## Perf opportunities
- Batch writes and flush asynchronously to avoid blocking compilation.
- Use scratch buffers to reduce serialization allocations.
- Limit scope enumeration to reduce IO in large caches.

## Code pointers
- `crates/rspack_storage/**`
