# rspack_binding_api — Performance Opportunities

**Size**: 23,251 lines of Rust across 125 files  
**Role**: NAPI (Node-API) bindings — bridges Rust core with JavaScript/Node.js for configuration, plugins, hooks, and compilation access  
**Impact**: Medium-High — every JS plugin hook call crosses this boundary, and hot paths like `buildModule`/`succeedModule` fire for every module

---

## Table of Contents

1. [JS↔Rust Boundary Crossing Cost](#1-jsrust-boundary-crossing-cost)
2. [Hook System Overhead](#2-hook-system-overhead)
3. [Data Marshalling](#3-data-marshalling)
4. [Thread Safety Overhead](#4-thread-safety-overhead)
5. [Compilation Object Exposure](#5-compilation-object-exposure)

---

## 1. JS↔Rust Boundary Crossing Cost

Every time a Rust compilation hook calls into JavaScript (e.g., for webpack-compatible plugins), it involves:
1. **Context switch**: From Rust async runtime to Node.js event loop
2. **Data serialization**: Converting Rust types to NAPI values
3. **GC pressure**: Creating JS objects that need garbage collection
4. **Await overhead**: Waiting for JS promise resolution

For hooks like `buildModule` and `succeedModule` that fire for every module (10K times), this cost is significant:

```rust
define_hook!(CompilationBuildModule: Series(...), tracing=false);
define_hook!(CompilationSucceedModule: Series(...), tracing=false);
```

Note: `tracing=false` on these hooks — they're already identified as hot paths.

**Opportunity**:
1. **Batch hook calls**: Instead of calling JS once per module, batch multiple modules and call once per batch
2. **Filter hooks**: Track which hooks have JS listeners. Skip the boundary crossing entirely for hooks with no JS taps.
3. **Move common plugin logic to Rust**: Popular webpack plugins (DefinePlugin, ProvidePlugin, etc.) should be reimplemented in Rust to avoid the boundary entirely.

**Impact**: High for projects with JS plugins that tap hot hooks. Low for projects using only builtin plugins.

**Estimated Gain**: 10-40% of make phase for JS-plugin-heavy configurations

---

## 2. Hook System Overhead

The hook system supports multiple tap types (sync, async, bail, parallel). Each hook invocation involves:
- Checking if any JavaScript taps exist
- Dynamic dispatch through the plugin driver
- NAPI callback invocation for JS taps
- Result conversion back to Rust

**Opportunity**:
1. **Compile-time hook optimization**: When no JS taps are registered, compile out the JS check entirely
2. **Hook fusion**: Combine related hooks (e.g., `buildModule` + `succeedModule` → `moduleBuilt` with before/after data)

**Estimated Gain**: 2-5% of compilation time

---

## 3. Data Marshalling

Converting between Rust and JS types is expensive for complex objects:

```rust
// Examples of data crossing the boundary:
// - Module objects (identifier, type, loaders, dependencies)
// - Chunk objects (id, name, files, groups)
// - Compilation stats (modules, chunks, assets)
// - Source maps
```

Large objects like source maps or module sources involve string/buffer copies across the boundary.

**Opportunity**:
1. **Lazy property access**: Instead of eagerly converting all fields, expose properties as getters that only convert when accessed
2. **SharedArrayBuffer**: For large binary data (source maps, assets), use shared memory instead of copying
3. **Cached conversions**: Cache converted JS objects for repeatedly-accessed data

**Impact**: Medium. The marshalling cost scales with object complexity and access frequency.

**Estimated Gain**: 5-15% of hook overhead

---

## 4. Thread Safety Overhead

The `BindingCell` type wraps compilation data for safe access from JS:

```rust
pub code_generation_results: BindingCell<CodeGenerationResults>,
```

This involves reference counting (`Arc`) and potentially locking for mutable access.

**Opportunity**: Use unsafe escape hatches when JS access patterns are known to be single-threaded (e.g., within a single hook callback).

**Impact**: Low. The overhead per access is small.

**Estimated Gain**: <1%

---

## 5. Compilation Object Exposure

The entire compilation object is exposed to JS through NAPI. This means:
- Every field must be convertible to JS
- Changes to internal data structures must maintain NAPI compatibility
- Performance optimizations in Rust must not break JS API contracts

**Opportunity**: Reduce the surface area of exposed data. Many internal fields are never accessed from JS but are still made accessible.

**Impact**: Low for performance, significant for API stability and refactoring freedom.

**Estimated Gain**: Enables other optimizations (e.g., changing internal data structures)

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Filter/skip hooks with no JS taps | 10-40% of make phase | Medium |
| 2 | Batch hook calls for per-module hooks | 10-30% of hook overhead | High |
| 3 | Lazy property access for marshalled objects | 5-15% of hook overhead | Medium |
