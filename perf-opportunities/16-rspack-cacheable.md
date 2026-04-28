# rspack_cacheable — Performance Opportunities

**Size**: 2,736 lines of Rust across 36 files  
**Role**: Serialization framework for persistent cache — provides derive macros and converters for serializing/deserializing compilation artifacts  
**Impact**: High for persistent cache performance; determines how fast artifacts can be saved/loaded

---

## Table of Contents

1. [Serialization Format Choice](#1-serialization-format-choice)
2. [Derive Macro Generated Code](#2-derive-macro-generated-code)
3. [Custom Converters Overhead](#3-custom-converters-overhead)

---

## 1. Serialization Format Choice

The cacheable system uses `rkyv` (zero-copy deserialization) under the hood:

```toml
# Cargo.toml dependencies
rkyv = { workspace = true, optional = true }
```

`rkyv` provides:
- Zero-copy deserialization (data is usable directly from the serialized buffer)
- Very fast serialization
- Aligned memory layout requirements

**Opportunity**:
1. **Ensure zero-copy is actually used**: Verify that deserialized data is accessed through `rkyv` archived types rather than being copied into owned types
2. **Alignment optimization**: Use `rkyv`'s alignment features to reduce padding overhead
3. **Skip serialization for unchanged artifacts**: Track which artifacts changed and only serialize those

**Impact**: High for persistent cache load/save performance.

**Estimated Gain**: 10-30% of cache I/O (if zero-copy is fully utilized)

---

## 2. Derive Macro Generated Code

The `#[cacheable]` derive macro generates serialization/deserialization code for every annotated struct. This includes:
- `rkyv::Archive`, `Serialize`, `Deserialize` implementations
- Custom field converters (e.g., `AsPreset`, `AsOption`, `AsVec`)
- Size calculations for pre-allocation

**Opportunity**:
1. **Reduce generated code size**: Large derive impls increase compile time and binary size
2. **Inline critical serialization paths**: Ensure the generated code is `#[inline]` for small types
3. **Skip unnecessary fields**: Mark fields with `#[cacheable(skip)]` for runtime-only data

**Impact**: Low for runtime performance (more relevant for compile time).

**Estimated Gain**: <1% runtime, faster compile times

---

## 3. Custom Converters Overhead

The `with` module provides converters like `AsPreset`, `AsOption`, `AsMap`:

```rust
#[cacheable(with=AsPreset)]
encoded: SmolStr,
```

Each converter adds a layer of indirection during serialization/deserialization.

**Opportunity**: For high-frequency types (like hash digests, identifiers), ensure converters are zero-cost abstractions that compile down to direct memory operations.

**Impact**: Low.

**Estimated Gain**: <1%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Ensure zero-copy deserialization is fully utilized | 10-30% of cache I/O | Medium |
| 2 | Skip serialization for unchanged artifacts | 5-15% of cache save | Medium |
