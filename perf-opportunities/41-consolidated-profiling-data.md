# Consolidated Profiling Data — All 8 Runs

All profiling runs on the same machine (cloud VM, Linux x86_64), rspack v2.0.0-beta.0 (debug build, unoptimized + debuginfo). Release builds are estimated at ~7x faster.

---

## Run Configurations

| Run | Modules | Async Chunks | Mode | SideEffects | Concat | Minimize | Devtool |
|-----|---------|-------------|------|-------------|--------|----------|---------|
| A | 200 | 0 | prod | true | false | false | none |
| B | 500 | 0 | prod | true | false | false | none |
| C | 1000 | 0 | prod | true | false | false | none |
| D | 151 | 50 | prod | true | false | false | none |
| E | 531 | 30 | prod | true | true | false | source-map |
| F | 531 | 30 | dev | false | false | false | eval-cheap |
| G | 531 | 30 | prod | **false** | **false** | false | none |
| H | 531 | 30 | prod | true | true | **true** | none |

---

## Complete Phase Timing Table (milliseconds)

| Phase | A (200) | B (500) | C (1K) | D (151+50a) | E (531+30a) | F (dev) | G (no-SE) | H (minify) |
|-------|---------|---------|--------|-------------|-------------|---------|-----------|------------|
| **build module graph** | 146 | 380 | 748 | 176 | 453 | 408 | 408 | 418 |
| finish modules | 3 | 6 | 12 | 4 | 7 | 6 | 5 | 7 |
| **optimize dependencies** | 14 | 85 | 300 | 27 | 112 | **0** | 48 | 115 |
| **build chunk graph** | 3 | 6 | 11 | 268 | **1,210** | 158 | 154 | **1,165** |
| optimize modules | 0 | 1 | 1 | 1 | 1 | 1 | 0 | 1 |
| **optimize chunks** | 3 | 7 | 11 | 158 | **330** | 67 | 66 | 322 |
| optimize tree | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| optimize chunk modules | 1 | 2 | 4 | 9 | 33 | 0 | 0 | 32 |
| module ids | 1 | 1 | 2 | 3 | 7 | 4 | 4 | 5 |
| chunk ids | 0 | 0 | 0 | 1 | 3 | 5 | 1 | 3 |
| optimize code gen | 2 | 3 | 6 | 1 | 2 | 0 | 2 | 2 |
| create module hashes | 1 | 1 | 3 | 3 | 16 | 6 | 6 | 15 |
| **code generation** | 0 | 1 | 2 | 14 | 60 | 35 | 36 | 59 |
| runtime requirements | 3 | 3 | 3 | 24 | 28 | 27 | 26 | 25 |
| **hashing** | 3 | 3 | 3 | 19 | 19 | 21 | 19 | 21 |
| create chunk assets | 1 | 1 | 1 | 23 | 26 | 43 | 28 | 25 |
| **process assets** | 0 | 0 | 0 | 17 | 63 | 0 | 0 | **405** |
| **emit** | 4 | 4 | 3 | — | 75 | 51 | 75 | 81 |
| **TOTAL** | **216** | **529** | **1,140** | **816** | **2,479** | **868** | **906** | **2,730** |

---

## Scaling Patterns Extracted

### Linear with Module Count (O(n))

| Phase | Formula (debug ms) | R² | Notes |
|-------|-------------------|----|-------|
| build module graph | T ≈ 0.75 × n | 0.99 | Parallelized; main-thread bottleneck |
| finish modules | T ≈ 0.012 × n | 0.98 | |
| module ids | T ≈ 0.002 × n | 0.95 | |
| optimize code gen | T ≈ 0.006 × n | 0.97 | |
| create module hashes | T ≈ 0.003 × n | 0.90 | Parallelized |

### Super-Linear with Module Count (O(n²))

| Phase | Formula (debug ms) | Evidence |
|-------|-------------------|----------|
| **optimize dependencies** | T ≈ 2.92e-4 × n² | 6.07x growth for 2.5x modules |
| - SideEffects: update connections | T ≈ 2.92e-4 × n² | Confirmed root cause: per-task fresh caches |

### Linear with Chunk Count (O(chunks))

| Phase | Formula (debug ms) | Notes |
|-------|-------------------|-------|
| runtime requirements | T ≈ 0.48 × chunks | |
| hashing | T ≈ 0.38 × chunks | Sequential for runtime chunks |
| create chunk assets | T ≈ 0.46 × chunks | Parallelized |
| process assets (no minify) | T ≈ 0.34 × chunks | RealContentHash |
| process assets (minify) | T ≈ 8.1 × chunks | SWC minification dominates |

### Super-Linear with Chunk Count + Modules (O(n × chunks²))

| Phase | Evidence | Notes |
|-------|----------|-------|
| **build chunk graph** | 3ms@1K/0async → 1,210ms@531/30async | BFS queue items scale as n × chunks |
| **optimize chunks** | 3ms@200/0async → 330ms@531/30async | Combinatorial cache group matching |

---

## Feature Impact Matrix

Shows the **marginal cost** of each production feature:

| Feature | Cost (531+30async) | What it Adds | Can it be Optimized? |
|---------|-------------------|-------------|---------------------|
| **SideEffects optimization** | +1,056ms (chunk graph) + 112ms (opt deps) | Tree shaking quality | YES — O(n²) → O(n) |
| **Module concatenation** | +33ms (analysis) + ~25ms (codegen) | Bundle size reduction | YES — AST caching |
| **Minification** | +342ms | Bundle size compression | YES — result caching |
| **Source maps** | +63ms (process) + ~25ms (emit) | Debugging support | Moderate |
| **RealContentHash** | +30ms | Long-term caching | Minor |
| **SplitChunks** | +330ms | Code splitting optimization | YES — priority queue |

**Total production overhead vs development**: 2,479ms - 868ms = **1,611ms (1.86x slower)**

---

## Bottleneck Ranking by Configuration

### For Pure Module-Heavy Builds (10K modules, few chunks)
1. **SideEffects O(n²)** — projected 29s at 10K
2. Make phase main-thread — projected 1.5s
3. Minification — projected 1s

### For Chunk-Heavy Builds (many async imports)
1. **BuildChunkGraph BFS** — 48.8% of total at 531+30async
2. **SplitChunks greedy loop** — 13.3% of total
3. SideEffects — 4.5% (lower % because chunk graph dominates)

### For Development Mode
1. **Make phase** — 47% of total (no optimization passes to compete with)
2. Build chunk graph — 18%
3. Optimize chunks — 7.7%

### For Watch Mode Rebuild (1 file change)
1. Make phase (re-build 1-5 modules) — ~50-200ms
2. Seal phase (if incremental disabled) — ~200-500ms
3. Minification (if enabled) — ~50-200ms per changed chunk

---

## Key Cross-Run Insights

1. **SideEffects causes a 7.9x increase in chunk graph work** (Run G vs E: 154ms → 1,210ms) — because it rewires connections, creating more active edges for the BFS.

2. **Development mode is 2.85x faster** (Run F vs E) — but only because 3 expensive plugins are disabled, not because of any algorithmic optimization.

3. **Minification adds 13% to total** (Run H vs E: +251ms net) — already well-parallelized with rayon.

4. **The bottleneck shifts with workload**: linear modules → SideEffects dominates; async chunks → BuildChunkGraph dominates. Both need fixing for the react-10k case which has both.

5. **Code generation scales with chunk count, not module count**: 0ms at 1K modules/1 chunk → 60ms at 531 modules/30 chunks. This is because ConcatenatedModule codegen is expensive.
