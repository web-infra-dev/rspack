# Deep Dive: Async Chunks Profiling — Code Splitting Under Stress

**Benchmark**: 151 modules (100 shared + 50 async pages + 1 entry), 50 dynamic imports creating 50 async chunk groups  
**Key finding**: BuildChunkGraph becomes the **#1 bottleneck** with async chunks, taking 268ms (33% of total) vs 11ms without.

---

## Profiling Results

| Phase | No Async (1K modules) | With 50 Async (151 modules) | Notes |
|-------|----------------------|----------------------------|-------|
| build module graph | 748ms (65.6%) | 176ms (21.6%) | Far fewer modules |
| **build chunk graph** | **11ms (1.0%)** | **268ms (32.8%)** | **24x slower!** |
| optimize dependencies | 300ms (26.3%) | 27ms (3.3%) | Fewer modules |
| **optimize chunks** | **11ms (1.0%)** | **158ms (19.4%)** | **14x slower!** |
| code generation | 2ms | 14ms | Per-chunk overhead |
| runtime requirements | 3ms | 24ms | Per-chunk overhead |
| hashing | 3ms | 19ms | Per-chunk overhead |
| create chunk assets | 1ms | 23ms | Per-chunk overhead |
| process assets | 0ms | 17ms | RealContentHashPlugin |
| **Total** | **1140ms** | **816ms** | |

### BuildChunkGraph Sub-Breakdown

```
process queue: 266ms              ← BFS traversal with 4,995 queue items
  2,518 blocks processed
  50 chunk groups connected
  50 chunk groups processed for merging
  50 module sets merged
```

### SplitChunksPlugin Sub-Breakdown

```
prepare module data: 0ms
prepare cache groups: 2ms
process cache groups: 149ms       ← The greedy module group loop
ensure max size fit: 3ms
```

---

## Why Code Splitting Is So Expensive Here

### 1. BigUint Available Module Merging (268ms)

With 50 async chunk groups, the `process_chunk_groups_for_merging` method runs frequently. Each merge involves:

```rust
// For each chunk group to merge:
cgi.min_available_modules = Arc::new(
    cgi.min_available_modules.as_ref() & modules_to_be_merged.as_ref()
);
```

With 151 modules, each BigUint is `⌈151/64⌉ = 3` u64 words = 24 bytes (small). But with 50 chunk groups × multiple merge iterations, the number of operations is high.

The `process queue` processes 4,995 queue items — that's 33 items per module on average. Each item involves BigUint bit tests and bit sets.

At 10K modules with 500 async chunks (realistic for a large SPA):
- Each BigUint: 1,250 bytes
- 500 chunk groups × ~10 merges each = 5,000 merge operations
- Each merge: 1,250 byte AND + comparison
- Queue items: ~50,000+
- **Projected time: 5-20 seconds**

### 2. SplitChunksPlugin Combinatorial (149ms)

The `process cache groups` time comes from the combinatorial chunk set analysis:

```rust
fn get_combinations(
    chunk_sets_in_graph, chunk_sets_by_count
) -> FxHashMap<ChunksKey, Vec<UkeySet<ChunkUkey>>> {
    chunk_sets_in_graph.into_par_iter()
        .map(|(chunks_key, chunks_set)| {
            for (count, array_of_set) in chunk_sets_by_count.iter() {
                for set in array_of_set {
                    if set.is_subset(&chunks_set) { result.push(set.clone()); }
                }
            }
        }).collect()
}
```

With 50 async chunks creating many unique chunk sets, the subset checking becomes expensive. The `minChunks: 2` + `minSize: 0` config creates many cache group candidates.

### 3. Per-Chunk Overhead Accumulates

Every phase that processes chunks scales with chunk count:
- Runtime requirements: 24ms (for 50+ chunks)
- Hashing: 19ms (for 50+ chunks, including sequential runtime chunk hashing)
- Create chunk assets: 23ms (render each chunk)
- Process assets: 17ms (RealContentHashPlugin processes each chunk)

At 500 chunks: these would be ~10x larger.

---

## Key Finding: Chunk Count Is the Real Scaling Dimension

For rspack_core, the two independent scaling dimensions are:
1. **Module count** — affects make phase, tree shaking, module hashing
2. **Chunk count** — affects code splitting, chunk hashing, asset rendering

Most analysis has focused on module count. But **chunk count scales independently** and can be equally problematic:

| Chunk Count | BuildChunkGraph | SplitChunks | ChunkHashing | ChunkAssets |
|-------------|----------------|-------------|-------------|-------------|
| 1 | 3ms | 1ms | 2ms | 1ms |
| 50 | 268ms | 149ms | 19ms | 23ms |
| 500 (est.) | 2-5s | 1-3s | 200ms | 250ms |

---

## Optimization Opportunities Specific to Async Chunks

### 1. FixedBitSet for Available Modules (Critical at Scale)

Even at 151 modules, BigUint operations take 266ms. FixedBitSet would:
- Eliminate heap allocation per merge (in-place AND)
- Enable SIMD for bitwise operations
- Reduce Arc overhead

**Projected savings**: 40-60% of BuildChunkGraph = 100-160ms at 50 async chunks

### 2. Parallel Chunk Group Processing

Currently, `process_queue` is single-threaded. Chunk groups that share no modules could be processed in parallel:

```rust
// Current: sequential BFS
while let Some(action) = self.queue.pop() {
    match action { ... }
}

// Proposed: parallel when independent
let (independent, dependent) = partition_by_dependencies(&self.queue);
independent.par_iter().for_each(|action| process(action));
// Then process dependent sequentially
```

**Projected savings**: 20-40% of process_queue time (depends on graph structure)

### 3. SplitChunksPlugin: Bitmap-Based Subset Testing

Replace `UkeySet::is_subset` with bitmap operations:

```rust
// Current: HashSet subset check — O(|subset|) per check
if set.is_subset(&chunks_set) { ... }

// Proposed: bitmap subset — O(1) per check
if (set_bitmap & chunks_bitmap) == set_bitmap { ... }
```

With 50 chunks, a u64 bitmap is sufficient. With 500 chunks, 8 u64s suffice.

**Projected savings**: 50-70% of `process cache groups` time

### 4. Reduce Per-Chunk Runtime Requirements Iterations

Each chunk's runtime requirements involve iterating all modules in the chunk and merging their requirements. The iterative loop (described in `01-rspack-core.md` §7) runs per-chunk.

**Opportunity**: Pre-compute a global map of `RuntimeGlobals → additional RuntimeGlobals` and apply the transitive closure in one pass instead of iterating.

### 5. Parallel Chunk Asset Rendering

Chunk asset rendering already uses `rspack_futures::scope`:
```rust
let results = rspack_futures::scope(|token| {
    chunks.iter().for_each(|chunk| { s.spawn(render_chunk) })
}).await;
```

But the overhead of spawning 50+ tokio tasks adds up. Batching (as discussed in `05-rspack-futures.md`) would help.

---

## Projected Impact at Scale (10K modules, 500 async chunks)

| Optimization | Current (est.) | After Fix | Savings |
|-------------|---------------|-----------|---------|
| FixedBitSet for BuildChunkGraph | 5-20s | 2-8s | 3-12s |
| Bitmap SplitChunks | 1-3s | 0.3-1s | 0.7-2s |
| Parallel chunk group processing | 5-20s | 3-12s | 2-8s |
| Batch task spawning | 0.5-1s | 0.2-0.5s | 0.3-0.5s |
| **Total seal phase** | **8-30s** | **3-12s** | **5-18s** |

This could be the **largest single improvement** for applications with many async routes (which is the react-10k profile — React apps with route-based code splitting).
