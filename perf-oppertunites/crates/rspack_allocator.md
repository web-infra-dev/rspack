# rspack_allocator

## Role
mimalloc integration and allocator configuration.

## Perf opportunities
- Tune allocator settings for module graph + codegen workloads.
- Reduce large transient allocations to avoid page faults.
- Consider per-thread arenas for hot parallel paths.

## Code pointers
- `crates/rspack_allocator/**`
