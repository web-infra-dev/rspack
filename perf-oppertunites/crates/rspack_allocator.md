# rspack_allocator

## Role
mimalloc integration and allocator configuration.

## Profiling relevance
- Allocation pressure is a top hotspot in perf samples.
- Allocation strategy directly affects page faults and throughput.

## Perf opportunities
- Tune allocator settings for module graph + codegen workloads.
- Reduce large transient allocations to avoid page faults.
- Consider per-thread arenas for hot parallel paths.

## Suggested experiments
- Compare allocation stats with different mimalloc settings.
- Measure build time impact of allocator tuning.

## Code pointers
- `crates/rspack_allocator/Cargo.toml`
- `crates/rspack_allocator/src/lib.rs`
- `crates/rspack_allocator/**`
