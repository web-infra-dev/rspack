# rspack_storage — Performance Opportunities

**Size**: 6,414 lines of Rust across 28 files  
**Role**: Persistent cache storage — stores/loads compilation artifacts to/from disk for faster rebuilds  
**Impact**: High for rebuild/watch mode, Low for cold builds

---

## Table of Contents

1. [Pack Storage I/O Pattern](#1-pack-storage-io-pattern)
2. [Serialization Overhead](#2-serialization-overhead)
3. [Scope-Level Locking](#3-scope-level-locking)
4. [Cache Invalidation Granularity](#4-cache-invalidation-granularity)

---

## 1. Pack Storage I/O Pattern

**File**: `crates/rspack_storage/src/pack/mod.rs`

The storage uses a "pack" format with scoped data:

```rust
pub struct PackStorage {
    pub manager: ScopeManager,
    pub updates: Mutex<ScopeUpdates>,
}
```

Writes are batched via `set()` calls that accumulate in `ScopeUpdates`, then flushed via `trigger_save()`. Reads (`load()`) are per-scope.

**Opportunity**:
1. **Async I/O**: Ensure all disk operations use async I/O to avoid blocking the runtime
2. **Memory-mapped I/O**: Use `mmap` for reading large pack files — avoids copying data into user space
3. **Compression**: Compress pack data (zstd or lz4) to reduce I/O volume at the cost of CPU
4. **Parallel loads**: Load multiple scopes in parallel during startup

**Impact**: High for rebuild performance. The time to load/save persistent cache directly affects rebuild latency.

**Estimated Gain**: 20-50% of cache load/save time

---

## 2. Serialization Overhead

Data is serialized as `Vec<u8>` key-value pairs:

```rust
type ItemKey = Vec<u8>;
type ItemValue = Vec<u8>;
type ItemPairs = Vec<(Arc<ItemKey>, Arc<ItemValue>)>;
```

Each item is independently serialized, creating many small `Vec<u8>` allocations. The `Arc` wrapper adds reference counting overhead.

**Opportunity**:
1. **Batch serialization**: Serialize entire scopes as a single blob instead of individual items
2. **Use `rkyv` zero-copy deserialization**: The `rspack_cacheable` crate already supports `rkyv`. Ensure hot paths use zero-copy deserialization.
3. **Reduce Arc overhead**: For items that are loaded once and never shared, use `Box` instead of `Arc`

**Impact**: Medium. Serialization overhead scales with the number of cached items.

**Estimated Gain**: 10-30% of cache load time

---

## 3. Scope-Level Locking

```rust
pub updates: Mutex<ScopeUpdates>,
```

All cache updates go through a single `Mutex`. During save, the entire update map is locked.

**Opportunity**:
1. **Per-scope locks**: Use a `DashMap<&'static str, ScopeUpdate>` instead of a single Mutex
2. **Lock-free writes**: Use a concurrent append log for updates

**Impact**: Low in practice since saves are typically triggered once at the end of compilation.

**Estimated Gain**: <1%

---

## 4. Cache Invalidation Granularity

The cache tracks changes at the module level, but invalidation can be coarse — changes to one module may invalidate cache entries for dependent modules.

**Opportunity**:
1. **Content-addressed caching**: Key cache entries by content hash instead of module identity. Unchanged content gets automatic cache hits regardless of identity changes.
2. **Fine-grained invalidation**: Track which specific parts of a module changed (e.g., only exports changed) and only invalidate dependent caches that care about those parts.

**Impact**: Medium for watch mode with many dependent modules.

**Estimated Gain**: 10-30% of incremental rebuild time

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Memory-mapped I/O for pack files | 20-50% of cache I/O | Medium |
| 2 | Zero-copy deserialization with rkyv | 10-30% of cache load | Medium |
| 3 | Content-addressed caching | 10-30% of incremental rebuild | High |
