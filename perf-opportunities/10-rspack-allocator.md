# rspack_allocator — Performance Opportunities

**Size**: ~15 lines of Rust in 1 file  
**Role**: Sets the global memory allocator — uses mimalloc by default, with optional tracy or sftrace profiling allocators  
**Impact**: Foundational — every allocation in the entire process goes through this allocator

---

## Table of Contents

1. [mimalloc Configuration](#1-mimalloc-configuration)
2. [Alternative Allocators](#2-alternative-allocators)
3. [Allocation-Heavy Hot Paths](#3-allocation-heavy-hot-paths)

---

## 1. mimalloc Configuration

**File**: `crates/rspack_allocator/src/lib.rs`

```rust
#[global_allocator]
#[cfg(not(any(miri, target_family = "wasm")))]
#[cfg(not(any(feature = "sftrace-setup", feature = "tracy-client")))]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;
```

mimalloc is used with default configuration. mimalloc provides:
- Excellent multi-threaded allocation performance
- Low fragmentation for mixed allocation sizes
- Good locality for per-thread allocations

However, mimalloc's default configuration may not be optimal for Rspack's allocation pattern, which involves:
- Many small allocations (AST nodes, dependency objects, string interning)
- Large temporary allocations (source buffers, source maps)
- Frequent cross-thread transfers (task loop sends results between threads)

**Opportunity**:
1. **Tune mimalloc options**: Enable `mimalloc-sys` with `extended` feature and configure:
   - `mi_option_eager_commit`: Enable eager commit for large allocations
   - `mi_option_segment_cache`: Increase segment cache for better reuse
   - `mi_option_purge_delay`: Tune purge delay for better memory return to OS
2. **Thread-local caches**: Ensure mimalloc's thread-local free lists are properly sized for Rspack's allocation patterns
3. **Page size tuning**: Match mimalloc page sizes to common allocation sizes (dependency objects, AST nodes)

**Impact**: Low-Medium. mimalloc defaults are already good, but tuning can help with Rspack's specific workload.

**Estimated Gain**: 1-5% overall (hard to predict without profiling)

---

## 2. Alternative Allocators

**Opportunity**: Benchmark alternative allocators for Rspack's workload:
- **jemalloc**: Better for long-running processes (watch mode), better fragmentation control
- **snmalloc**: Good multi-threaded performance, message-passing friendly (suits task loop pattern)
- **tcmalloc**: Google's allocator, good for large applications

Each allocator has different trade-offs for:
- Small allocation throughput
- Large allocation throughput
- Cross-thread deallocation
- Memory fragmentation over time
- Memory return to OS

**Impact**: Unknown without benchmarking. Could be significant (5-10%) or negligible.

**Estimated Gain**: 0-10% (requires benchmarking)

---

## 3. Allocation-Heavy Hot Paths

The allocator itself can't fix allocation-heavy code, but identifying where allocations dominate can guide optimization efforts:

1. **Module graph construction**: Each module creates ~50-100 small objects (dependencies, connections, blocks)
2. **AST construction**: SWC's AST allocates many small nodes on the heap
3. **HashMap resizing**: Growing hash maps from default size causes multiple re-allocations
4. **String operations**: Module identifiers, dependency requests, file paths involve many string allocations
5. **Source map processing**: JSON serialization/deserialization creates temporary allocations

**Opportunity**: Use arena allocators for per-module data. The `bumpalo` crate provides fast arena allocation that can be freed in bulk:

```rust
// Hypothetical: arena-allocated per-module
let arena = bumpalo::Bump::new();
let ast = parse_with_arena(&source, &arena);
let deps = scan_dependencies_with_arena(&ast, &arena);
// ... use deps ...
drop(arena); // Free everything at once
```

This would eliminate thousands of individual allocations per module, replacing them with a few large arena allocations.

**Impact**: Medium-High if applied to the make phase.

**Estimated Gain**: 5-15% of make phase (if arena allocation is adopted for AST and dependency objects)

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Arena allocation for per-module data | 5-15% of make phase | High |
| 2 | Tune mimalloc options | 1-5% overall | Low |
| 3 | Benchmark alternative allocators | 0-10% | Medium |
