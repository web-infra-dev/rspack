# rspack_collections — Performance Opportunities

**Size**: ~400 lines of Rust across 3 files  
**Role**: Provides specialized collection types — `Identifier` (interned string via `Ustr`), `IdentifierMap/Set` (identity-hashed), `Ukey` (u32 key), `UkeyMap/Set` (identity-hashed), `Database` (typed key-value store)  
**Impact**: Foundational — these types are used throughout the entire codebase for every module, dependency, chunk, and chunk group lookup

---

## Table of Contents

1. [Identifier (Ustr) Interning Overhead](#1-identifier-ustr-interning-overhead)
2. [UkeyHasher Identity Hash](#2-ukeyhasherd-identity-hash)
3. [Database (HashMap wrapper) Overhead](#3-database-hashmap-wrapper-overhead)
4. [IdentifierMap vs FxHashMap Choice](#4-identifiermap-vs-fxhashmap-choice)

---

## 1. Identifier (Ustr) Interning Overhead

**File**: `crates/rspack_collections/src/identifier.rs`

`Identifier` wraps `Ustr`, which is an interned string:

```rust
pub struct Identifier(Ustr);
```

`Ustr` provides:
- O(1) equality comparison (pointer comparison)
- O(1) hashing (precomputed hash)
- Global interning table with atomic operations

The identity hasher is used:
```rust
pub type IdentifierHasher = ustr::IdentityHasher;
pub type IdentifierMap<V> = HashMap<Identifier, V, BuildHasherDefault<IdentifierHasher>>;
```

This is already well-optimized for lookup-heavy workloads. However:

**Issues**:
1. **Ustr global table never shrinks**: All identifiers are interned for the process lifetime. In long-running watch mode, this leaks memory for removed modules.
2. **Ustr creation involves global lock**: Creating a new `Ustr` from `&str` requires checking the global interning table (read lock) and potentially inserting (write lock). At 10K modules with unique identifiers, the initial interning creates lock contention during parallel module building.
3. **`Identifier::from(String)` allocates**: Converting a `String` to `Identifier` requires interning, which copies the string into the global table. The original `String` is dropped.

**Opportunity**:
1. **Pre-intern common identifiers**: Common module type identifiers, dependency types, etc. could be pre-interned at startup.
2. **Batch interning**: During module graph construction, batch intern strings to reduce lock contention.
3. **Consider `lasso` crate**: A more modern string interning crate with better concurrent performance and optional arena-based storage.

**Impact**: Low. Ustr's performance is generally excellent, but the global lock can become visible under high concurrency.

**Estimated Gain**: 1-2% of make phase (interning during parallel factorize/build)

---

## 2. UkeyHasher Identity Hash

**File**: `crates/rspack_collections/src/ukey.rs`

```rust
pub struct UkeyHasher(u32);

impl std::hash::Hasher for UkeyHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!("UkeyHasher should only used for UKey")
    }
    fn write_u32(&mut self, i: u32) {
        self.0 = i;
    }
    fn finish(&self) -> u64 {
        self.0 as u64
    }
}
```

This is an optimal identity hasher for `u32` keys. It's used for `ChunkUkey`, `ChunkGroupUkey`, and `CgiUkey`. Zero hash computation overhead.

However, since Ukey values are sequential (`AtomicU32::fetch_add`), using them as direct HashMap indices could lead to poor bucket distribution. The identity hash maps sequential keys to sequential buckets, which is fine for `HashMap` (it applies its own mixing) but could be suboptimal.

**Opportunity**: The current design is reasonable. If HashMap performance becomes an issue, consider `Vec`-based indexed storage for dense sequential Ukeys (O(1) lookup, better cache locality).

**Impact**: Low.

**Estimated Gain**: <1%

---

## 3. Database (HashMap wrapper) Overhead

```rust
pub struct Database<Item: DatabaseItem> {
    inner: HashMap<<Item as DatabaseItem>::ItemUkey, Item, BuildHasherDefault<UkeyHasher>>,
}
```

`Database` is a typed wrapper around `UkeyMap`. It provides `expect_get()` methods that panic on missing keys. The overhead is near-zero (the wrapper methods should be inlined).

**Opportunity**: Consider a `Vec`-based database for `ChunkGroupInfo` and similar types with dense sequential keys. This would give O(1) access without hashing.

**Impact**: Low.

**Estimated Gain**: <1%

---

## 4. IdentifierMap vs FxHashMap Choice

Throughout the codebase, both `IdentifierMap<V>` and `FxHashMap<Identifier, V>` are used for module-keyed maps. The former uses the identity hasher (O(1) hash via precomputed hash), while the latter uses FxHash.

Some code uses `FxHashMap` with `Identifier` keys, missing the benefit of precomputed hashes:
- `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs` uses `IdentifierMap` correctly
- But some utility code may inadvertently use `HashMap` with default hasher

**Opportunity**: Audit all `HashMap<ModuleIdentifier, _>` usages to ensure they use `IdentifierMap` (identity hasher) rather than `FxHashMap` or default `HashMap`.

**Impact**: Low. Most code already uses the correct types.

**Estimated Gain**: <1%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Reduce Ustr interning lock contention | 1-2% of make phase | Medium |
| 2 | Vec-based storage for dense sequential Ukeys | <1% | Medium |
| 3 | Audit HashMap type usage for module keys | <1% | Low |
