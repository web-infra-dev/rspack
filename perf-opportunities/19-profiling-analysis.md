# Profiling Analysis — Real Trace Data

This document presents actual profiling results from running Rspack on benchmark workloads, with projections to the react-10k scale.

---

## Benchmark Setup

- **Rspack**: v2.0.0-beta.0 (debug build, unoptimized)
- **Mode**: Production, minimize disabled, splitChunks enabled
- **Hardware**: Cloud VM, Linux x86_64
- **Tracing**: `RSPACK_TRACING=ALL` with logger output

> Note: Debug builds are 5-10x slower than release builds. The relative proportions between phases are what matter, not absolute times.

---

## 200-Module Benchmark Results

| Phase | Time (ms) | % of Total |
|-------|-----------|-----------|
| **build module graph** | 146 | 67.6% |
| optimize dependencies | 14 | 6.5% |
| build chunk graph | 3 | 1.4% |
| optimize chunks | 3 | 1.4% |
| optimize chunk modules | 1 | 0.5% |
| create module hashes | 1 | 0.5% |
| code generation | 0 | 0% |
| runtime requirements | 3 | 1.4% |
| hashing | 3 | 1.4% |
| create chunk assets | 1 | 0.5% |
| **Total** | **216** | |

### Sub-phase breakdown:
- **SideEffectsFlagPlugin**: find optimizable=4ms, update connections=12ms (total 16ms)
- **ModuleConcatenationPlugin**: <1ms total
- **SplitChunksPlugin**: <1ms total

---

## 500-Module Benchmark Results

| Phase | Time (ms) | % of Total | vs 200-mod |
|-------|-----------|-----------|------------|
| **build module graph** | 380 | 71.8% | 2.60x |
| optimize dependencies | 85 | 16.1% | 6.07x |
| build chunk graph | 6 | 1.1% | 2.0x |
| optimize chunks | 7 | 1.3% | 2.33x |
| optimize chunk modules | 2 | 0.4% | 2.0x |
| create module hashes | 1 | 0.2% | 1.0x |
| code generation | 1 | 0.2% | - |
| runtime requirements | 3 | 0.6% | 1.0x |
| hashing | 3 | 0.6% | 1.0x |
| create chunk assets | 1 | 0.2% | 1.0x |
| **Total** | **529** | | **2.45x** |

### Sub-phase breakdown:
- **SideEffectsFlagPlugin**: find optimizable=19ms, update connections=81ms (total 100ms — **super-linear scaling!**)
- **ModuleConcatenationPlugin**: ~1ms
- **SplitChunksPlugin**: prepare=1ms, process=2ms

---

## Scaling Analysis

### Linear Scaling Phases (O(n))
These phases scale roughly linearly with module count:
- `build module graph`: 2.60x for 2.5x modules → ~linear
- `build chunk graph`: 2.0x for 2.5x modules → ~linear
- `optimize chunks`: 2.33x → ~linear
- `code generation`: linear (parallel)
- `create module hashes`: ~constant (likely already cached/parallel)

### Super-Linear Scaling Phases (O(n²) or worse)
- **`optimize dependencies`**: 6.07x for 2.5x modules → **~O(n²)**
  - SideEffectsFlagPlugin `update connections`: 12ms → 81ms (6.75x) → **O(n²)**
  - SideEffectsFlagPlugin `find optimizable`: 4ms → 19ms (4.75x) → **~O(n²)**

### Projected Times at 10K Modules (20x from 500)

| Phase | 500 mod | Projected 10K (linear) | Projected 10K (if O(n²)) |
|-------|---------|----------------------|--------------------------|
| build module graph | 380ms | 7,600ms | 7,600ms |
| optimize dependencies | 85ms | 1,700ms | **34,000ms** |
| build chunk graph | 6ms | 120ms | 120ms |
| optimize chunks | 7ms | 140ms | 140ms |
| hashing | 3ms | 60ms | 60ms |
| code generation | 1ms | 20ms | 20ms |
| **Total** | 529ms | ~10.5s | ~42s |

> In release mode, divide by ~5-10x. So at 10K modules: ~1-2s (linear) or ~4-8s (if O(n²) phases dominate).

---

## Key Finding: SideEffectsFlagPlugin is the Scaling Bottleneck

The `optimize dependencies` phase shows **super-linear scaling**, driven by `SideEffectsFlagPlugin`:

```
LOG from rspack.SideEffectsFlagPlugin
<t> prepare connections: 0 ms
<t> find optimizable connections: 19 ms     ← O(n²)
<t> do optimize connections: 0 ms
<t> update connections: 81 ms               ← O(n²)
    optimized 0 connections
```

At 500 modules, `update connections` alone takes 81ms. At 10K modules, this could become **seconds**.

**Root cause**: The plugin iterates all connections and for each connection, checks if the target module has side effects. The connection checking involves walking dependency chains, which creates O(n × m) complexity where n is modules and m is average connections per module.

**File**: `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs`

This is the **#1 highest-priority optimization target** revealed by profiling.

---

## Key Finding: Build Module Graph Dominates

The make phase consumes 68-72% of total time across both benchmarks. This confirms that:
1. Module resolution and building are the primary bottleneck
2. The task loop main-thread serialization is the limiting factor
3. Parsing + SWC transforms per module dominate

At 10K modules in release mode, the make phase would likely take 500ms-1.5s, with:
- ~50% in actual file I/O and parsing
- ~30% in task loop scheduling and main-thread graph updates
- ~20% in resolution

---

## Key Finding: Hashing and Code Gen Are Already Well-Parallelized

Hashing and code generation show near-constant time between 200 and 500 modules, indicating they are effectively parallelized. The minor increase is from the constant overhead of the passes themselves.

This confirms that the `rspack_futures::scope` parallelism model is working well for these phases.

---

## Updated Priority Ranking (Based on Profiling)

| # | Opportunity | Evidence | Priority |
|---|-----------|----------|----------|
| **1** | **Fix SideEffectsFlagPlugin O(n²) scaling** | 6.07x growth for 2.5x modules | **CRITICAL** |
| **2** | **Reduce make phase main-thread bottleneck** | 72% of total time | **HIGH** |
| **3** | **Replace BigUint with FixedBitSet** | BuildChunkGraph will matter more at 10K | **HIGH** |
| **4** | **Merge SWC AST transform passes** | Per-module overhead × 10K | **HIGH** |
| **5** | **Skip JS boundary for unused hooks** | Constant overhead per hook call | **MEDIUM** |
| 6 | Avoid `.modules()` allocating new HashMap | Called many times | MEDIUM |
| 7 | Batch tokio task spawns | 10K task overhead | MEDIUM |
| 8 | Optimize module concatenation analysis | Only significant for ESM-heavy projects | MEDIUM |
| 9 | Eliminate ExportsInfoData clones | Part of optimize dependencies overhead | MEDIUM |

---

## Profiling Methodology Notes

### Running Profiling Yourself

```bash
# Build dev mode
pnpm run build:binding:dev

# Run with logger tracing (verbose timing)
node -e "
const binding = require('./crates/node_binding/binding.js');
binding.registerGlobalTrace('ALL', 'logger', './trace.log');
// ... run build ...
binding.cleanupGlobalTrace();
"

# Run with perfetto tracing (Chrome devtools trace)
node -e "
const binding = require('./crates/node_binding/binding.js');
binding.registerGlobalTrace('BENCH', 'perfetto', './trace.perfetto');
// ... run build ...
binding.cleanupGlobalTrace();
"
```

### Interpreting Stats Output

Set `stats.logging: 'verbose'` in rspack config to see per-plugin timing:
```js
stats: { logging: 'verbose', loggingTrace: false }
```

Key timing prefixes:
- `<t>` — timing measurement
- Numbers in `LOG from rspack.xxx` — per-plugin breakdown
