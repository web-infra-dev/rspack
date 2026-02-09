# rspack_cacheable

## Role
Cacheable serialization/deserialization framework.

## Profiling relevance
- Hot when persistent caching is enabled.
- Costs scale with serialization size and frequency.

## Perf opportunities
- Reuse serialization scratch buffers.
- Avoid unnecessary allocations in rkyv serializers.
- Cache schema/metadata for repeated types.

## Suggested experiments
- Profile cache serialization time on large builds.
- Compare allocation counts with/without scratch buffer reuse.

## Code pointers
- `crates/rspack_cacheable/Cargo.toml`
- `crates/rspack_cacheable/src/lib.rs`
- `crates/rspack_cacheable/**`
