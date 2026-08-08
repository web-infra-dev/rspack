# rspack_plugin_split_chunks — Performance Opportunities

**Size**: 2,705 lines of Rust across 13 files  
**Role**: Implements `optimization.splitChunks` — groups modules into cache groups and creates optimized chunks  
**Impact**: Medium — runs once per compilation during OptimizeChunksPass, but with 10K modules the combinatorial analysis is expensive

---

## Table of Contents

1. [Combinatorial Chunk Set Explosion](#1-combinatorial-chunk-set-explosion)
2. [Module Group Map Greedy Loop](#2-module-group-map-greedy-loop)
3. [Used Exports Grouping Overhead](#3-used-exports-grouping-overhead)
4. [Chunk Index Map Computation](#4-chunk-index-map-computation)
5. [Module Size Computation Redundancy](#5-module-size-computation-redundancy)

---

## 1. Combinatorial Chunk Set Explosion

**File**: `crates/rspack_plugin_split_chunks/src/plugin/module_group.rs`

The `Combinator` computes all possible chunk combinations for each module:

```rust
fn get_combinations(
    chunk_sets_in_graph: FxHashMap<ChunksKey, UkeySet<ChunkUkey>>,
    chunk_sets_by_count: UkeyIndexMap<u32, Vec<UkeySet<ChunkUkey>>>,
) -> FxHashMap<ChunksKey, Vec<UkeySet<ChunkUkey>>> {
    chunk_sets_in_graph.into_par_iter()
        .map(|(chunks_key, chunks_set)| {
            let mut result = vec![];
            for (count, array_of_set) in chunk_sets_by_count.iter() {
                if *count < chunks_set.len() as u32 {
                    for set in array_of_set {
                        if set.is_subset(&chunks_set) {
                            result.push(set.clone());
                        }
                    }
                }
            }
            (chunks_key, result)
        }).collect()
}
```

This checks every possible subset of chunks for each unique chunk set. With many async chunks, the number of unique chunk sets can grow large, and the subset checking becomes O(n²) in the number of chunk sets.

**Opportunity**:
1. **Limit combination depth**: Most useful split chunks involve 2-3 chunks. Limit the subset enumeration to practical depths.
2. **Bitmap-based subset testing**: Use bitsets for chunks to make `is_subset` O(1) instead of O(n).
3. **Pre-filter**: Skip chunk sets that can't meet `minChunks` threshold before computing combinations.

**Impact**: Medium. At 10K modules, the number of unique chunk sets is bounded but the subset check cost grows.

**Estimated Gain**: 10-20% of SplitChunksPlugin time (varies significantly by config)

---

## 2. Module Group Map Greedy Loop

**File**: `crates/rspack_plugin_split_chunks/src/plugin/mod.rs`

The plugin uses a greedy loop to select and create chunks:

```rust
while !module_group_map.is_empty() {
    let (module_group_key, mut module_group) = self.find_best_module_group(&mut module_group_map);
    // Create/reuse chunk
    // Move modules to new chunk
    // Remove modules from other module groups
    self.remove_all_modules_from_other_module_groups(
        &module_group, &mut module_group_map, &used_chunks, compilation, &module_sizes
    );
}
```

`find_best_module_group` iterates all remaining module groups to find the best one. `remove_all_modules_from_other_module_groups` then updates all remaining groups. This is O(n²) in the number of module groups.

**Opportunity**:
1. **Priority queue**: Use a binary heap sorted by priority/size to find the best group in O(log n) instead of O(n).
2. **Lazy removal**: Instead of eagerly removing modules from all other groups, mark modules as claimed and skip them during chunk creation.
3. **Batch processing**: Groups with the same priority could potentially be processed in parallel.

**Impact**: Medium. With many cache groups and modules, the greedy loop dominates.

**Estimated Gain**: 15-30% of SplitChunksPlugin time for complex configurations

---

## 3. Used Exports Grouping Overhead

**File**: `crates/rspack_plugin_split_chunks/src/plugin/module_group.rs`

When `usedExports` optimization is enabled, each module's chunks are grouped by their export usage pattern:

```rust
fn group_chunks_by_exports(...) -> Vec<UkeySet<ChunkUkey>> {
    let exports_info = module_graph.get_prefetched_exports_info(...);
    let mut grouped_by_used_exports: FxHashMap<UsageKey, UkeySet<ChunkUkey>> = Default::default();
    for chunk_ukey in module_chunks {
        let chunk = chunk_by_ukey.expect_get(&chunk_ukey);
        let runtime = chunk.runtime();
        let usage_key = runtime_key_map.entry(...)
            .or_insert_with(|| exports_info.get_usage_key(Some(runtime)));
        grouped_by_used_exports.entry(usage_key).or_default().insert(chunk_ukey);
    }
    grouped_by_used_exports.into_values().collect()
}
```

`get_usage_key` computes a hash of all export usage states for a given runtime. At 10K modules × multiple runtimes, this is expensive.

**Opportunity**:
1. **Cache usage keys**: The same runtime configuration maps to the same usage key for most modules. Cache at the runtime level.
2. **Skip for simple cases**: If a module has no exports or all exports are used in all runtimes, the grouping is trivial.

**Estimated Gain**: 5-10% of SplitChunksPlugin time when usedExports is enabled

---

## 4. Chunk Index Map Computation

**File**: `crates/rspack_plugin_split_chunks/src/plugin/mod.rs`

```rust
let chunk_index_map: UkeyMap<ChunkUkey, u64> = {
    let mut ordered_chunks = compilation.build_chunk_graph_artifact.chunk_by_ukey
        .values().collect::<Vec<_>>();
    ordered_chunks.sort_by_cached_key(|chunk| {
        let group = chunk.groups().iter()
            .map(|group| compilation.build_chunk_graph_artifact.chunk_group_by_ukey.expect_get(group))
            .min_by(|group1, group2| group1.index.cmp(&group2.index))
            .expect("chunk should have at least one group");
        let chunk_index = group.chunks.iter()
            .position(|c| *c == chunk.ukey())
            .expect("chunk should be in its group");
        (group.index, chunk_index)
    });
    // ...
};
```

This sorts all chunks for deterministic ordering. The `min_by` lookup for each chunk iterates all its groups. With many chunks (hundreds in a 10K module app), this sorting is O(n log n) but with O(groups_per_chunk) inner work.

**Opportunity**: Pre-compute the primary group index during chunk graph construction.

**Estimated Gain**: <1% (minor)

---

## 5. Module Size Computation Redundancy

**File**: `crates/rspack_plugin_split_chunks/src/plugin/mod.rs`

```rust
let module_sizes = Self::get_module_sizes(&all_modules, compilation);
let module_chunks = Self::get_module_chunks(&all_modules, compilation);
```

Module sizes are computed by iterating all modules and calling `module.size()` for each source type. This data exists elsewhere (e.g., cached in the module itself via `cached_source_sizes`).

**Opportunity**: Reuse cached sizes from the module rather than recomputing. The `NormalModule::cached_source_sizes` DashMap already caches per-source-type sizes.

**Estimated Gain**: <1%

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Priority queue for module group selection | 15-30% of split chunks | Medium |
| 2 | Bitmap-based subset testing for combinations | 10-20% of split chunks | Medium |
| 3 | Cache usage keys for usedExports grouping | 5-10% | Low |
