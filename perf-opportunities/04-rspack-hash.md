# rspack_hash — Performance Opportunities

**Size**: ~250 lines of Rust in 1 file  
**Role**: Provides unified hashing (xxhash64, MD4, SHA256) and hex digest encoding  
**Impact**: High per-call volume — hashing is called for every module, chunk, and runtime module multiple times

---

## Table of Contents

1. [Hash Function Dispatch Overhead](#1-hash-function-dispatch-overhead)
2. [Digest Allocation (SmolStr)](#2-digest-allocation-smolstr)
3. [Hex Encoding Performance](#3-hex-encoding-performance)
4. [Box Indirection for Hash State](#4-box-indirection-for-hash-state)
5. [Hash Salt Overhead](#5-hash-salt-overhead)

---

## 1. Hash Function Dispatch Overhead

**File**: `crates/rspack_hash/src/lib.rs`

Every `write()` call goes through an enum match:

```rust
impl Hasher for RspackHash {
    fn write(&mut self, bytes: &[u8]) {
        match self {
            RspackHash::Xxhash64(hasher) => hasher.write(bytes),
            RspackHash::MD4(hasher) => hasher.update(bytes),
            RspackHash::SHA256(hasher) => hasher.update(bytes),
        }
    }
}
```

The hash function is configured once per compilation and never changes. But every `write()` call (millions per compilation — every module hash, chunk hash, content hash involves many writes) pays the enum dispatch cost.

**Opportunity**:
1. **Monomorphize at compilation level**: Use generics or type erasure at the compiler level so the hash function is resolved once, not on every write.
2. **Use `#[inline(always)]`**: Ensure the match is optimized away when the compiler can determine the variant at compile time.
3. **`xxhash64` fast path**: Since xxhash64 is the recommended/default hash, provide a specialized non-enum path when xxhash64 is selected.

**Impact**: Low-Medium. The match is branch-predicted well after warm-up, but at millions of writes, even small overhead compounds.

**Estimated Gain**: 1-3% of hashing time

---

## 2. Digest Allocation (SmolStr)

```rust
pub struct RspackHashDigest {
    encoded: SmolStr,
}
```

`SmolStr` is 24 bytes inline and heap-allocates for strings > 22 bytes. Hash digests:
- xxhash64: 16 hex chars (fits inline ✓)
- MD4: 32 hex chars (heap-allocated ✗)
- SHA256: 64 hex chars (heap-allocated ✗)

For MD4/SHA256, every `digest()` call allocates on the heap. At 10K modules × multiple hash computations, that's 50K+ allocations.

**Opportunity**:
1. **Use a fixed-size buffer**: Since the maximum digest is 64 chars (SHA256), use `[u8; 64]` + length instead of SmolStr.
2. **Lazy encoding**: Don't hex-encode immediately. Store the raw bytes and encode only when `.encoded()` is called.
3. **Consider `ArrayString<64>`**: From the `arrayvec` crate — stack-allocated, fixed-capacity string.

**Impact**: Medium for MD4/SHA256 users; Low for xxhash64 (default).

**Estimated Gain**: 1-5% of hashing time for MD4/SHA256

---

## 3. Hex Encoding Performance

```rust
#[inline]
fn hex<'a>(data: &[u8], output: &'a mut [u8]) -> &'a str {
    const HEX_TABLE: &[u8; 16] = b"0123456789abcdef";
    for byte in data {
        output[i] = HEX_TABLE[(byte >> 4) as usize];
        output[i + 1] = HEX_TABLE[(byte & 0x0f) as usize];
        i += 2;
    }
    unsafe { std::str::from_utf8_unchecked(&output[..i]) }
}
```

This implementation is already well-optimized with:
- Lookup table
- `#[inline]`
- No bounds checks (implicitly optimized via assertion)
- Unsafe UTF-8 conversion (hex is always ASCII)

**Opportunity**: The implementation is already good. Minor gain possible from SIMD hex encoding for SHA256 (64 byte output), but not worth the complexity.

**Estimated Gain**: Negligible

---

## 4. Box Indirection for Hash State

```rust
pub enum RspackHash {
    Xxhash64(Box<Xxh64>),
    MD4(Box<md4::Md4>),
    SHA256(Box<sha2::Sha256>),
}
```

Each hash state is boxed. `Xxh64` is small (only 48 bytes), so boxing it adds an unnecessary indirection. However, `Sha256` is 108 bytes, making boxing reasonable to keep the enum size manageable.

**Opportunity**: 
1. **Unbox Xxhash64**: Since xxhash64 is the default and most common, embed it directly in the enum and only box the larger variants.
2. **Use a union with manual tag**: Avoid the enum discriminant overhead for the hot path.

```rust
// Alternative: xxhash64 inline, others boxed
pub enum RspackHash {
    Xxhash64(Xxh64),  // 48 bytes, no indirection
    MD4(Box<md4::Md4>),
    SHA256(Box<sha2::Sha256>),
}
```

**Impact**: Low. The box indirection is a single pointer chase per write.

**Estimated Gain**: <1%

---

## 5. Hash Salt Overhead

```rust
pub fn with_salt(function: &HashFunction, salt: &HashSalt) -> Self {
    let mut this = Self::new(function);
    if !matches!(salt, HashSalt::None) {
        salt.hash(&mut this);
    }
    this
}
```

A new `RspackHash` is created with salt for every hash operation. The salt is typically the same string for the entire compilation. 

**Opportunity**: Pre-compute a "salted seed" state that can be cloned instead of hashing the salt every time.

```rust
// Pre-compute once
let salted_seed = RspackHash::with_salt(&function, &salt);
// Clone for each use (memcpy of hash state, no salt re-hashing)
let hasher = salted_seed.clone();
```

The `RspackHash` already derives `Clone`, so this is straightforward.

**Impact**: Low. Salt hashing is a single write per hash operation.

**Estimated Gain**: <1%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Monomorphize hash function at compilation level | 1-3% of hashing | Medium |
| 2 | Use fixed-size buffer for digest instead of SmolStr | 1-5% for MD4/SHA256 | Low |
| 3 | Unbox Xxhash64 variant | <1% | Low |
| 4 | Pre-compute salted seed state | <1% | Low |
