# Combined Realistic Profile: 531 Modules + 30 Async Chunks + Source Maps + Concatenation

This profile simulates a medium-size React SPA with route-based code splitting, shared component library, source maps, and scope hoisting.

**Setup**: 300 shared utils + 200 leaf components + 30 async pages + 1 entry = 531 modules, 30 dynamic imports, `devtool: 'source-map'`, `concatenateModules: true`, `splitChunks: { chunks: 'all', minSize: 0, minChunks: 2 }`

---

## Full Timing Breakdown

| Phase | Time (ms) | % of Total | Key Metric |
|-------|-----------|-----------|------------|
| **build chunk graph** | **1,210** | **48.8%** | 26,562 queue items, 9,051 blocks |
| build module graph | 453 | 18.3% | 531 modules |
| **optimize chunks (SplitChunks)** | **330** | **13.3%** | 315ms in process cache groups |
| optimize dependencies | 112 | 4.5% | SideEffects: 76ms update + 25ms find |
| process assets | 63 | 2.5% | Source maps + RealContentHash |
| code generation | 60 | 2.4% | 28 concatenated modules |
| optimize chunk modules (Concat) | 33 | 1.3% | 471 potential, 28 successful, 370 bailed |
| create chunk assets | 26 | 1.0% | |
| runtime requirements | 28 | 1.1% | |
| hashing | 19 | 0.8% | |
| create module hashes | 16 | 0.6% | |
| emit assets | 75 | 3.0% | |
| **Total** | **2,479** | | |

### Sub-Breakdowns

**BuildChunkGraph** (1,210ms):
```
process queue:                 1,205ms (99.6%)
  26,562 queue items
  9,051 blocks
  30 chunk groups connected
  30 chunk groups processed for merging
```

**SplitChunksPlugin** (330ms):
```
prepare module data:  3ms
prepare cache groups: 5ms
process cache groups: 315ms (95.5%) ← greedy loop
ensure max size fit:  4ms
```

**ModuleConcatenationPlugin** (33ms):
```
select relevant:         1ms (471 potential root, 470 potential inner)
find modules to concat:  26ms
  28 successful configs (avg 3 modules)
  370 bailed out
  654 incorrect chunks rejections
  552 incorrect importer chunks rejections
```

**SourceMapDevToolPlugin** (28ms):
```
collect source maps: 19ms
emit source map assets: 9ms
```

**RealContentHashPlugin** (30ms):
```
create hash regexp:   1ms
create ordered hashes: 10ms
old hash to new hash: 10ms
collect hash updates:  6ms
update assets:         3ms
```

---

## Key Insight: The Bottleneck Shifts with Workload

| Workload | #1 Bottleneck | % of Total |
|----------|--------------|-----------|
| 1000 linear modules, no async | optimize dependencies (SideEffects) | 26.3% |
| 151 modules, 50 async chunks | build chunk graph | 32.8% |
| **531 modules, 30 async chunks** | **build chunk graph** | **48.8%** |

**With realistic code splitting, build chunk graph dominates.** The BFS process queue processes 26,562 items for 531 modules — that's **50 queue items per module** because each module is visited by multiple chunk groups.

### Scaling Projection: 10K Modules + 200 Async Chunks

At 10K modules with 200 async chunks (realistic large React SPA):
- Queue items: ~500,000 (50 per module × 10K)
- Each queue item involves BigUint bit test: 1,250 bytes per test
- **Projected build chunk graph: 20-45 seconds (debug), 3-6.5 seconds (release)**

At this scale, build chunk graph alone could exceed the entire target build time.

---

## Module Concatenation Analysis

28 out of 471 potential modules were successfully concatenated (6.0% success rate). Main rejection reasons:

```
654 incorrect chunks          — Module not in same chunks as root
552 incorrect chunks of importer — Importer chunks don't match
370 bailed out completely     — Failed for other reasons
148 importer failed           — Parent module couldn't concatenate
```

The dominant failure reason is **chunk mismatch** — with code splitting, modules end up in different chunks, preventing concatenation. This is inherent to the architecture but means:
- The concatenation analysis effort (26ms for `find modules to concatenate`) is mostly wasted
- 94% of analysis work leads to rejection

**Opportunity**: Pre-filter modules by chunk membership before running the concatenation analysis. Build a chunk→module index and only attempt concatenation for modules sharing all chunks.

---

## Source Map Impact

With `devtool: 'source-map'`:
- SourceMapDevToolPlugin: 28ms (collect 19ms + emit 9ms)
- Additional overhead in code generation (generating source map mappings)
- Emit phase: 75ms (includes writing .map files)

**Total source map overhead: ~50-100ms** for 531 modules. At 10K modules: ~1-2 seconds.

**Opportunity**: Parallel source map generation. The current `collect source maps` step iterates assets sequentially. With rayon's `par_iter`, this could be parallelized.

---

## Code Generation with Concatenation

Code generation took 60ms — much higher than the 2ms seen without concatenation. This is because:
1. **28 ConcatenatedModule code gen** each re-parses their inner modules
2. Each concatenated module does scope analysis, name deduplication, and source assembly
3. With source maps, the ReplaceSource operations also track mappings

**Breakdown estimate**:
- Regular module codegen (503 modules): ~10ms (parallel)
- ConcatenatedModule codegen (28 modules, avg 3 inner each): ~50ms
  - Re-parsing 84 inner modules: ~30ms
  - Scope analysis + name dedup: ~10ms
  - Source assembly: ~10ms

**Opportunity**: Eliminating the re-parse in ConcatenatedModule would save ~30ms (50% of code gen time in this scenario).

---

## Updated Priority Ranking for Combined Workload

| # | Opportunity | Phase | This Scenario | 10K Projected |
|---|-----------|-------|---------------|---------------|
| **1** | **FixedBitSet for code splitter** | BuildChunkGraph | Save 400-600ms | Save 2-4s |
| **2** | **SideEffects O(n²) fix** | OptimizeDeps | Save 60-80ms | Save 2-4s |
| **3** | **SplitChunks priority queue** | OptimizeChunks | Save 100-200ms | Save 0.5-2s |
| **4** | **SWC pass merging** | BuildModuleGraph | Save 40-60ms | Save 0.5-1s |
| 5 | Module concat pre-filter by chunks | OptimizeChunkModules | Save 15-20ms | Save 50-200ms |
| 6 | ConcatenatedModule AST caching | CodeGeneration | Save 30ms | Save 200-500ms |
| 7 | Parallel source map generation | ProcessAssets | Save 10-15ms | Save 100-500ms |
| 8 | Parallel chunk group processing | BuildChunkGraph | Save 200-400ms | Save 1-3s |

The combined workload makes it clear that **code splitting performance is the dominant concern** for real-world React SPAs with route-based splitting.
