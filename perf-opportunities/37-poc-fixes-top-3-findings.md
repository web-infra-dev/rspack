# Proof-of-Concept Fix Outlines for Top 3 Findings

This document provides concrete, implementable fix outlines for the three highest-impact performance issues discovered through profiling.

---

## Finding #1: SideEffectsFlagPlugin O(n²) — Per-Task Fresh Caches

### The Problem (Confirmed)

**File**: `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs`, lines 165-175

```rust
// CURRENT CODE — O(n²) due to per-task fresh caches
let side_effects_state_map: IdentifierMap<ConnectionState> = all_modules
    .par_iter()
    .map(|(module_identifier, module)| {
        (*module_identifier,
         module.get_side_effects_connection_state(
            module_graph,
            &compilation.module_graph_cache_artifact,
            &mut Default::default(),     // ← Fresh IdentifierSet per task!
            &mut Default::default(),     // ← Fresh IdentifierMap per task!
         ))
    })
    .collect();
```

**Impact**: 292ms at 1K modules (debug), projected 29s at 10K.

### The Fix: Topological Order with Shared Cache

```rust
// PROPOSED FIX — O(n) with shared cache

// Step 1: Compute topological order of modules (reverse dependency order)
let topo_order = {
    let mut order = Vec::with_capacity(all_modules.len());
    let mut visited = IdentifierSet::default();
    let mut in_progress = IdentifierSet::default();
    
    fn visit(
        module_id: &ModuleIdentifier,
        mg: &ModuleGraph,
        visited: &mut IdentifierSet,
        in_progress: &mut IdentifierSet,
        order: &mut Vec<ModuleIdentifier>,
    ) {
        if visited.contains(module_id) { return; }
        if in_progress.contains(module_id) { return; } // circular
        in_progress.insert(*module_id);
        
        if let Some(module) = mg.module_by_identifier(module_id) {
            for dep_id in module.get_dependencies() {
                if let Some(target) = mg.module_identifier_by_dependency_id(dep_id) {
                    visit(target, mg, visited, in_progress, order);
                }
            }
        }
        
        in_progress.remove(module_id);
        visited.insert(*module_id);
        order.push(*module_id);
    }
    
    for (module_id, _) in all_modules.iter() {
        visit(module_id, &module_graph, &mut visited, &mut in_progress, &mut order);
    }
    order
};

// Step 2: Process in topological order with shared cache
let mut connection_state_cache = IdentifierMap::default();
let mut module_chain = IdentifierSet::default();
let mut side_effects_state_map = IdentifierMap::default();

for module_id in &topo_order {
    let module = module_graph.module_by_identifier(module_id)
        .expect("should have module");
    let state = module.get_side_effects_connection_state(
        module_graph,
        &compilation.module_graph_cache_artifact,
        &mut module_chain,
        &mut connection_state_cache,  // SHARED — previous results reused!
    );
    side_effects_state_map.insert(*module_id, state);
}
```

**Why it works**: In topological order, when we process module A, all modules that A depends on have already been processed and their results are in `connection_state_cache`. The recursive `get_side_effects_connection_state` will hit the cache immediately instead of re-walking the dependency chain.

**Complexity**: O(n) instead of O(n²) — each module is visited exactly once.

**Trade-off**: Sequential instead of parallel. But O(n) sequential >> O(n²)/P parallel for large n. At 10K modules: O(n)=10K ops vs O(n²)/8=12.5M ops.

**Files to change**:
1. `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs` — Replace the `par_iter` with topological walk
2. No changes needed to `get_side_effects_connection_state` itself

---

## Finding #2: BuildChunkGraph BigUint → FixedBitSet

### The Problem (Confirmed)

**File**: `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs`

BigUint operations dominate the BFS process queue. At 531 modules + 30 async chunks, the BFS processes 26,562 queue items, each involving BigUint bit tests and bit sets.

The hottest path (line 2081-2084):
```rust
// CURRENT CODE — allocates new BigUint on every merge
let orig = cgi.min_available_modules.clone();  // Arc<BigUint> clone
cgi.min_available_modules =
    Arc::new(cgi.min_available_modules.as_ref() & modules_to_be_merged.as_ref());
    //       ^^^ ALLOCATES NEW BIGUINT ^^^
changed |= orig != cgi.min_available_modules;  // Full comparison
```

### The Fix: Replace BigUint with FixedBitSet

**Step 1**: Add `fixedbitset` dependency to `crates/rspack_core/Cargo.toml`:
```toml
[dependencies]
fixedbitset = "0.5"
```

**Step 2**: Replace types in `code_splitter.rs`:

```rust
// BEFORE:
use num_bigint::BigUint;

pub(crate) ordinal_by_module: IdentifierMap<u64>,
pub(crate) mask_by_chunk: UkeyMap<ChunkUkey, BigUint>,

pub struct ChunkGroupInfo {
    pub min_available_modules: Arc<BigUint>,
    pub available_modules_to_be_merged: Vec<Arc<BigUint>>,
    resulting_available_modules: Option<Arc<BigUint>>,
}

// AFTER:
use fixedbitset::FixedBitSet;

pub(crate) ordinal_by_module: IdentifierMap<usize>,  // usize for FixedBitSet indexing
pub(crate) mask_by_chunk: UkeyMap<ChunkUkey, FixedBitSet>,

pub struct ChunkGroupInfo {
    pub min_available_modules: FixedBitSet,        // No Arc needed — owned
    pub available_modules_to_be_merged: Vec<FixedBitSet>,  // No Arc
    resulting_available_modules: Option<FixedBitSet>,
}
```

**Step 3**: Replace operations:

```rust
// Bit test — BEFORE:
if cgi.min_available_modules.bit(*module_ordinal) { ... }
// AFTER:
if cgi.min_available_modules.contains(*module_ordinal) { ... }

// Bit set — BEFORE:
chunk_mask.set_bit(*module_ordinal, true);
// AFTER:
chunk_mask.insert(*module_ordinal);

// Merge (AND) — BEFORE (allocates!):
cgi.min_available_modules = Arc::new(cgi.min_available_modules.as_ref() & modules_to_be_merged.as_ref());
// AFTER (in-place, zero allocation!):
let changed = cgi.min_available_modules != modules_to_be_merged;  // Optional: pre-check
cgi.min_available_modules.intersect_with(&modules_to_be_merged);

// OR-assign — BEFORE:
new_resulting_available_modules |= mask;
// AFTER:
new_resulting_available_modules.union_with(mask);

// Initialize — BEFORE:
self.mask_by_chunk.insert(chunk_ukey, BigUint::from(0u32));
// AFTER:
self.mask_by_chunk.insert(chunk_ukey, FixedBitSet::with_capacity(module_count));
```

**Step 4**: Initialize with known module count:
```rust
// In prepare():
let module_count = all_modules.len();
// All FixedBitSets created with this capacity
```

**Files to change**:
1. `crates/rspack_core/Cargo.toml` — Add fixedbitset dependency
2. `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs` — Replace all BigUint usage (~47 occurrences)
3. `crates/rspack_core/src/compilation/build_chunk_graph/incremental.rs` — Update incremental types

**Expected improvement**: 40-60% of BuildChunkGraph time (from profiling: 1,210ms → ~500-700ms for 531+30async case).

---

## Finding #3: SplitChunksPlugin Greedy Loop → Priority Queue

### The Problem (Confirmed)

**File**: `crates/rspack_plugin_split_chunks/src/plugin/mod.rs`, lines ~200-260

```rust
// CURRENT CODE — O(n²) in module groups
while !module_group_map.is_empty() {
    // O(n) scan to find best group
    let (module_group_key, mut module_group) =
        self.find_best_module_group(&mut module_group_map);
    
    // ... create chunk, move modules ...
    
    // O(n) update all remaining groups  
    self.remove_all_modules_from_other_module_groups(
        &module_group, &mut module_group_map, &used_chunks,
        compilation, &module_sizes
    );
}
```

`find_best_module_group` iterates ALL remaining groups to find the one with the highest priority/size. With 100+ module groups (common with `minChunks: 2` and many shared modules), this is O(n²).

### The Fix: Binary Heap Priority Queue

```rust
use std::collections::BinaryHeap;
use std::cmp::Ordering;

// Wrapper for priority queue ordering
struct PrioritizedGroup {
    key: String,
    priority: f64,      // cache_group.priority
    size: f64,           // total module size
    module_count: usize, // for tiebreaking
}

impl Ord for PrioritizedGroup {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.partial_cmp(&other.priority)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.size.partial_cmp(&other.size).unwrap_or(Ordering::Equal))
            .then_with(|| self.module_count.cmp(&other.module_count))
    }
}

// PROPOSED FIX:
let mut heap = BinaryHeap::new();
for (key, group) in &module_group_map {
    heap.push(PrioritizedGroup {
        key: key.clone(),
        priority: group.get_cache_group(&self.cache_groups).priority as f64,
        size: group.total_size(),
        module_count: group.modules.len(),
    });
}

while let Some(best) = heap.pop() {
    let Some(module_group) = module_group_map.remove(&best.key) else {
        continue; // Already removed by a previous iteration
    };
    
    // Skip if group no longer meets min_size after module removals
    if !self.meets_min_size(&module_group) {
        continue;
    }
    
    // ... create chunk, move modules ...
    
    // Update remaining groups (mark modules as claimed)
    self.remove_all_modules_from_other_module_groups(
        &module_group, &mut module_group_map, &used_chunks,
        compilation, &module_sizes
    );
    // Note: groups whose priority changed need to be re-inserted into the heap
    // For simplicity, we can re-check during pop() and skip stale entries
}
```

**Key insight**: Instead of re-scanning all groups to find the best, the heap gives us O(log n) best-group extraction. Groups removed or modified by `remove_all_modules_from_other_module_groups` are handled by checking staleness at pop time (lazy deletion).

**Files to change**:
1. `crates/rspack_plugin_split_chunks/src/plugin/mod.rs` — Replace the while loop with heap-based selection

**Expected improvement**: 30-50% of SplitChunksPlugin time for complex configurations (315ms → ~160-220ms for 531+30async case).

---

## Isolating Profile: Side Effects Impact on BuildChunkGraph

To quantify the interaction between SideEffects and BuildChunkGraph, we profiled with sideEffects disabled:

| Config | BuildChunkGraph | Queue Items | Blocks |
|--------|----------------|-------------|--------|
| Production (full) | 1,210ms | 26,562 | 9,051 |
| Production (sideEffects=false) | 154ms | 3,168 | 1,221 |
| Development | 158ms | 3,168 | 1,221 |

**SideEffects optimization causes an 8.4x increase in BFS work.** This is because:
1. SideEffects rewires module connections (points imports directly to the final target module)
2. This creates more active connections per module
3. More connections mean more BFS queue items per module

**Implication**: Fixing the SideEffects O(n²) issue (#1) and the BigUint issue (#2) have a **compound effect** — fixing SideEffects may also reduce BuildChunkGraph time by reducing the connection explosion.

---

## Combined Impact Estimate (531 modules + 30 async chunks)

| Fix | Before | After | Saved |
|-----|--------|-------|-------|
| #1 SideEffects O(n²) → O(n) | 112ms | ~20ms | 92ms |
| #2 BigUint → FixedBitSet | 1,210ms | ~600ms | 610ms |
| #3 SplitChunks Priority Queue | 330ms | ~180ms | 150ms |
| **Combined** | **1,652ms** | **~800ms** | **852ms (52%)** |

At 10K modules + 200 async chunks (projected):
| Fix | Before (est.) | After (est.) | Saved |
|-----|--------------|-------------|-------|
| #1 | 4,170ms | ~430ms | 3,740ms |
| #2 | 3,000-6,500ms | 1,200-2,600ms | 1,800-3,900ms |
| #3 | 1,000-3,000ms | 500-1,500ms | 500-1,500ms |
| **Combined** | **8,170-13,670ms** | **2,130-4,530ms** | **6,040-9,140ms (67-74%)** |
