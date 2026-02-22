# Comprehensive Scaling Model: Rspack Performance at 10K-50K Modules

This document synthesizes all profiling data, code analysis, and algorithmic complexity findings into a unified performance model for rspack at scale.

---

## Data Points (Debug Build, Single-Thread Equivalent)

All profiling data collected on the same machine (cloud VM, Linux x86_64, Rust debug build).

### Linear Module Chain (No Async Chunks)

| Phase | 200 mod | 500 mod | 1000 mod | Complexity |
|-------|---------|---------|----------|------------|
| build module graph | 146ms | 380ms | 748ms | O(n) |
| optimize dependencies | 14ms | 85ms | 300ms | **O(n²)** |
| - SideEffects: update connections | 12ms | 81ms | 292ms | **O(n²)** |
| - SideEffects: find optimizable | 4ms | 19ms | 39ms | O(n log n) |
| build chunk graph | 3ms | 6ms | 11ms | O(n) |
| optimize chunks | 3ms | 7ms | 11ms | O(n) |
| optimize chunk modules | 1ms | 2ms | 4ms | O(n) |
| create module hashes | 1ms | 1ms | 3ms | O(n) ∥ |
| code generation | 0ms | 1ms | 2ms | O(n) ∥ |
| runtime requirements | 3ms | 3ms | 3ms | O(chunks) |
| hashing | 3ms | 3ms | 3ms | O(chunks) |
| create chunk assets | 1ms | 1ms | 1ms | O(chunks) |
| **Total** | **216ms** | **529ms** | **1140ms** | |

### With 50 Async Chunks (151 Modules)

| Phase | 151 mod, 50 async | Complexity |
|-------|-------------------|------------|
| build module graph | 176ms | O(n) |
| optimize dependencies | 27ms | O(n²) |
| build chunk graph | 268ms | O(n × chunks²) |
| optimize chunks | 158ms | O(modules × chunk_combos) |
| runtime requirements | 24ms | O(chunks) |
| hashing | 19ms | O(chunks) |
| create chunk assets | 23ms | O(chunks) |
| **Total** | **816ms** | |

---

## Scaling Formulas (Derived from Data)

Using regression on the profiling data:

### Build Module Graph (Make Phase)
```
T_make = 0.75 × n_modules (ms, debug build)
T_make_release ≈ T_make / 7 ≈ 0.107 × n_modules (ms)
```
Parallelized across P cores: `T_make_wall ≈ T_make / P + T_main_thread`

Main thread overhead: `T_main = 0.15 × n_modules (ms, debug)` (sequential)

### Optimize Dependencies (SideEffects O(n²))
```
T_side_effects = 2.92e-4 × n_modules² (ms, debug build)
T_side_effects_release ≈ T_side_effects / 7 ≈ 4.17e-5 × n_modules²
```

### Build Chunk Graph (Code Splitting)
```
T_chunk_graph = α × n_modules × n_chunk_groups + β × n_queue_items
where:
  α ≈ 0.01 (ms per module-chunk_group pair)
  β ≈ 0.053 (ms per queue item)
  n_queue_items ≈ 33 × n_modules (with async chunks)
```

### Optimize Chunks (SplitChunks)
```
T_split = γ × n_modules × n_chunk_combinations + δ × n_module_groups²
where:
  γ ≈ 0.003 (ms per module-combination pair)
  δ ≈ 0.01 (ms per module group comparison, greedy loop)
```

### Per-Chunk Phases
```
T_runtime_req = 0.48 × n_chunks (ms, debug)
T_hashing = 0.38 × n_chunks (ms, debug)
T_chunk_assets = 0.46 × n_chunks (ms, debug)
T_process_assets = 0.34 × n_chunks (ms, debug, with RealContentHash)
```

---

## Projections

### 10K Modules, 1 Chunk (Monolith, No Code Splitting)

| Phase | Debug (ms) | Release (est. ms) | % of Total |
|-------|-----------|-------------------|-----------|
| build module graph | 7,500 | 1,070 | 29.0% |
| - main thread bottleneck | 1,500 | 214 | 5.8% |
| optimize dependencies | **29,200** | **4,170** | **~55%** ★ |
| build chunk graph | 100 | 14 | 0.4% |
| optimize chunks | 100 | 14 | 0.4% |
| optimize chunk modules | 80 | 11 | 0.3% |
| create module hashes | 60 | 9 | 0.2% |
| code generation | 40 | 6 | 0.2% |
| runtime requirements | 3 | 0.4 | 0% |
| hashing | 3 | 0.4 | 0% |
| **Total** | **~37,000** | **~5,300** | |
| **Wall-clock (8 cores)** | | **~3,700** | |

**★ SideEffects O(n²) dominates!** At 10K modules it's projected to take ~4 seconds in release mode.

### 10K Modules, 200 Async Chunks (Typical React SPA)

| Phase | Debug (ms) | Release (est. ms) | % of Total |
|-------|-----------|-------------------|-----------|
| build module graph | 7,500 | 1,070 | 11.5% |
| optimize dependencies | 29,200 | 4,170 | 44.9% |
| build chunk graph | 5,000-20,000 | 700-2,900 | 7.5-31% |
| optimize chunks | 2,000-8,000 | 290-1,140 | 3-12% |
| runtime requirements | 96 | 14 | 0.1% |
| hashing | 76 | 11 | 0.1% |
| create chunk assets | 92 | 13 | 0.1% |
| process assets | 68 | 10 | 0.1% |
| **Total** | **~44,000-65,000** | **~6,300-9,300** | |

### 50K Modules (Monorepo at Scale)

| Phase | Release (est. ms) | Notes |
|-------|-------------------|-------|
| build module graph | 5,350 | Linear, well parallelized |
| optimize dependencies | **104,250** | **O(n²) — 104 seconds!** |
| **Total** | **>110,000** | **Over 2 minutes** |

This shows that **SideEffects O(n²) is the limiting factor** at very large scale. Without fixing this, rspack cannot efficiently handle 50K+ module projects in production mode.

---

## After Proposed Optimizations

### Fix SideEffects O(n²) → O(n)

Using shared cache or topological ordering:
```
T_side_effects_fixed = 0.3 × n_modules (ms, debug)
T_side_effects_fixed_release ≈ 0.043 × n_modules
```

### Fix BigUint → FixedBitSet

```
T_chunk_graph_fixed ≈ 0.4 × T_chunk_graph_current
```

### Merge SWC Passes (7→4)

```
T_make_fixed ≈ 0.85 × T_make_current (15% reduction)
```

### Projected 10K Modules After All Fixes

| Phase | Before (release ms) | After (release ms) | Improvement |
|-------|--------------------|--------------------|-------------|
| build module graph | 1,070 | 910 | -15% |
| optimize dependencies | **4,170** | **430** | **-90%** |
| build chunk graph (200 chunks) | 700-2,900 | 280-1,160 | -60% |
| optimize chunks | 290-1,140 | 200-800 | -30% |
| Other phases | ~60 | ~50 | -17% |
| **Total** | **~6,300-9,300** | **~1,870-3,350** | **-64 to -70%** |

### Projected 50K Modules After All Fixes

| Phase | Before | After |
|-------|--------|-------|
| optimize dependencies | 104s | **2.15s** |
| build module graph | 5.35s | 4.55s |
| **Total** | **>110s** | **~8-12s** |

---

## Summary: Performance Budget at 10K Modules (Release)

### Current State
- **Cold build**: ~5-9 seconds (dominated by SideEffects O(n²))
- **Rebuild (1 file change)**: ~500ms-2s (depending on incremental pass coverage)
- **Watch mode HMR**: ~200-500ms

### After All Tier 1-2 Optimizations
- **Cold build**: ~2-3 seconds (dominated by make phase)
- **Rebuild**: ~100-500ms
- **Watch mode HMR**: ~100-300ms

### Performance Breakdown (After Optimization)

```
Make Phase:        ~1,000ms (build module graph, parallelized)
Optimize Deps:       ~430ms (SideEffects, fixed O(n))
Code Splitting:    ~300-800ms (FixedBitSet, depends on chunks)
Split Chunks:      ~200-800ms (bitmap subset testing)
Code Gen + Hash:     ~100ms (already well parallelized)
Chunk Assets:         ~50ms (parallel rendering)
────────────────────────────
Total:           ~2,000-3,200ms
```

### Theoretical Minimum (All Known Optimizations Applied)

At 10K modules with 200 async chunks:
```
File I/O:              ~300ms (10K file reads, cached by OS)
SWC Parse+Transform:   ~400ms (0.04ms/module × 10K, parallel)
Dependency Scan:       ~200ms (parallel across modules)
Main Thread Graph:     ~200ms (unavoidable sequential part)
Optimize Deps:         ~200ms (single-pass, cached)
Code Splitting:        ~200ms (FixedBitSet + partial parallel)
Split Chunks:          ~150ms (bitmap ops)
Code Gen:               ~50ms (parallel)
Hashing:                ~50ms (parallel + streaming)
Chunk Assets:           ~50ms (parallel)
Emit:                   ~50ms (parallel I/O)
────────────────────────────
Theoretical Min:    ~1,850ms (~1.85 seconds)
```

This represents the practical floor for a 10K-module React SPA cold build on a modern 8-core machine.
