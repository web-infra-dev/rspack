# rspack_cacheable

## Role
Cacheable serialization/deserialization framework.

## Perf opportunities
- Reuse serialization scratch buffers.
- Avoid unnecessary allocations in rkyv serializers.
- Cache schema/metadata for repeated types.

## Code pointers
- `crates/rspack_cacheable/**`
