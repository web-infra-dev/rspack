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

## Key functions/structs to inspect
- `to_bytes` / `from_bytes` entrypoints (lib.rs).
- `Serializer` / `Deserializer` (serialize.rs, deserialize.rs).
- `with/*` adapters for common types (with/mod.rs).

## Suggested experiments
- Profile cache serialization time on large builds.
- Compare allocation counts with/without scratch buffer reuse.

## Code pointers
- `crates/rspack_cacheable/Cargo.toml`
- `crates/rspack_cacheable/src/lib.rs`
- `crates/rspack_cacheable/src/serialize.rs`
- `crates/rspack_cacheable/src/deserialize.rs`
- `crates/rspack_cacheable/src/with/mod.rs`
- `crates/rspack_cacheable/**`
