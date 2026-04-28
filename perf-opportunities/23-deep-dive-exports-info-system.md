# Deep Dive: ExportsInfo System & Side Effects Scaling Root Cause

**Files**:
- `crates/rspack_core/src/exports/exports_info.rs` — ExportsInfoData with BTreeMap of exports
- `crates/rspack_core/src/exports/export_info.rs` — ExportInfoData with target resolution
- `crates/rspack_core/src/normal_module.rs:808` — `get_side_effects_connection_state`
- `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs` — SideEffectsFlagPlugin

---

## The ExportsInfoData Structure

Every module has an `ExportsInfoData`:

```rust
pub struct ExportsInfoData {
    exports: BTreeMap<Atom, ExportInfoData>,  // Named exports
    other_exports_info: ExportInfoData,        // Template for unknown exports
    side_effects_only_info: ExportInfoData,    // Side effects tracking
    id: ExportsInfo,                           // Unique key
}
```

Each `ExportInfoData` contains:

```rust
pub struct ExportInfoData {
    belongs_to: ExportsInfo,
    name: Option<Atom>,
    used_name: Option<UsedNameItem>,
    target: HashMap<Option<DependencyId>, ExportInfoTargetValue>,  // Re-export targets
    target_is_set: bool,
    provided: Option<ExportProvided>,
    can_mangle_provide: Option<bool>,
    can_mangle_use: Option<bool>,
    can_inline_provide: Option<EvaluatedInlinableValue>,
    can_inline_use: Option<CanInlineUse>,
    terminal_binding: bool,
    exports_info: Option<ExportsInfo>,          // Nested exports (for objects)
    exports_info_owned: bool,
    has_use_in_runtime_info: bool,
    global_used: Option<UsageState>,
    used_in_runtime: Option<ustr::UstrMap<UsageState>>,  // Per-runtime usage
}
```

At 10K modules with avg 10 exports each:
- **100K ExportInfoData** instances
- Each with a `HashMap<Option<DependencyId>, ExportInfoTargetValue>` for target tracking
- Each with `Option<ustr::UstrMap<UsageState>>` for per-runtime usage

**Memory**: ~100K × ~200 bytes = **~20MB** just for export info data.

---

## Root Cause of O(n²) in SideEffectsFlagPlugin

### The smoking gun: line ~162 in side_effects_flag_plugin.rs

```rust
let side_effects_state_map: IdentifierMap<ConnectionState> = all_modules
    .par_iter()
    .map(|(module_identifier, module)| {
        (*module_identifier,
         module.get_side_effects_connection_state(
            module_graph,
            &compilation.module_graph_cache_artifact,
            &mut Default::default(),     // ← NEW IdentifierSet per module!
            &mut Default::default(),     // ← NEW IdentifierMap per module!
         ))
    })
    .collect();
```

**Each parallel task creates fresh caches.** When module A calls `get_side_effects_connection_state`, it may recursively check modules B, C, D (via `get_module_evaluation_side_effects_state`). Module E doing the same thing independently rediscovers the same B→C→D chain.

### The recursive chain in NormalModule

```rust
// crates/rspack_core/src/normal_module.rs:808
fn get_side_effects_connection_state(&self, ..., 
    module_chain: &mut IdentifierSet, 
    connection_state_cache: &mut IdentifierMap<ConnectionState>
) -> ConnectionState {
    // Check cache first
    if let Some(state) = connection_state_cache.get(&self.inner().id) {
        return *state;  // HIT — but cache is empty because it's fresh!
    }
    
    if Some(true) == self.build_meta().side_effect_free {
        module_chain.insert(self.identifier());
        
        // FOR EACH DEPENDENCY: recursively check side effects
        for dependency_id in self.get_dependencies().iter() {
            let state = dependency.get_module_evaluation_side_effects_state(
                module_graph, module_graph_cache,
                module_chain,           // Passed through — tracks circular refs
                connection_state_cache, // Passed through — but still per-task empty
            );
            if matches!(state, ConnectionState::Active(true)) {
                connection_state_cache.insert(self.inner().id, ConnectionState::Active(true));
                return ConnectionState::Active(true);
            }
        }
        connection_state_cache.insert(self.inner().id, current);
        return current;
    }
    ConnectionState::Active(true)
}
```

### Concrete scaling example

Consider a linear chain: A imports B imports C imports D imports ... imports Z (26 modules).

When computing `side_effects_state_map` in parallel:
- Thread 1 processes module A: walks A→B→C→D→...→Z (26 checks)
- Thread 2 processes module B: walks B→C→D→...→Z (25 checks)
- Thread 3 processes module C: walks C→D→...→Z (24 checks)
- ...
- Thread 26 processes module Z: walks Z (1 check)

Total work: 26 + 25 + 24 + ... + 1 = **n(n+1)/2 = O(n²)**

At 1000 modules in a chain: 500,500 module checks. At 10K: 50,005,000 module checks.

### Why `module_graph_cache` doesn't help

There IS a cache: `module_graph_cache.cached_get_side_effects_connection_state`. But this cache is read-only during this phase (it was computed in a previous compilation). For a cold build, this cache is empty.

Even in warm builds, the cache wraps the ENTIRE computation including the recursive part, so it only helps if the exact same module was computed in a previous compilation.

---

## Confirmed by Profiling Data

| Modules | SideEffects `update connections` | Growth Factor | Expected O(n²) |
|---------|--------------------------------|---------------|-----------------|
| 200 | 12ms | — | — |
| 500 | 81ms | 6.75x | 6.25x (500²/200²) |
| 1000 | 292ms | 3.60x | 4.0x (1000²/500²) |

The growth factors closely match O(n²) prediction (some deviation due to cache effects and not all modules being in chains).

**Projection at 10K modules**: 292ms × (10000²/1000²) = **29,200ms ≈ 29 seconds**

---

## Fix: Shared Cache Across Parallel Tasks

### Option A: Pre-compute in topological order (single-threaded)

```rust
let mut connection_state_cache = IdentifierMap::default();
let mut module_chain = IdentifierSet::default();

// Process in reverse topological order (leaves first)
for module_id in reverse_topo_order(&module_graph) {
    let module = module_graph.module_by_identifier(&module_id).unwrap();
    let state = module.get_side_effects_connection_state(
        module_graph, module_graph_cache,
        &mut module_chain,
        &mut connection_state_cache,  // SHARED — results accumulate!
    );
    side_effects_state_map.insert(module_id, state);
}
```

**Complexity**: O(n) — each module is visited exactly once because cache hits prevent re-walks.

**Trade-off**: Single-threaded, but O(n) >> O(n²)/P for large n.

### Option B: Two-phase parallel approach

```rust
// Phase 1: Compute leaf modules in parallel (no recursion needed)
let leaf_states: IdentifierMap<ConnectionState> = all_modules.par_iter()
    .filter(|(_, m)| m.get_dependencies().is_empty() || has_direct_side_effect(m))
    .map(|(id, m)| (*id, compute_direct_state(m)))
    .collect();

// Phase 2: Propagate up the dependency graph using cached leaf results
let all_states = propagate_from_leaves(&module_graph, &leaf_states);
```

### Option C: DashMap shared cache with parallel processing

```rust
let shared_cache: DashMap<ModuleIdentifier, ConnectionState> = DashMap::default();

all_modules.par_iter().map(|(module_identifier, module)| {
    if let Some(cached) = shared_cache.get(module_identifier) {
        return (*module_identifier, *cached);
    }
    let state = module.get_side_effects_connection_state(
        module_graph, module_graph_cache,
        &mut Default::default(),
        &shared_cache,  // Thread-safe shared cache
    );
    shared_cache.insert(*module_identifier, state);
    (*module_identifier, state)
}).collect()
```

This would require changing the `connection_state_cache` parameter from `&mut IdentifierMap` to a thread-safe cache.

---

## ExportsInfoData Clone Cost (for FlagDependencyExports/Usage)

Both `FlagDependencyExportsPlugin` and `FlagDependencyUsagePlugin` clone `ExportsInfoData` for parallel processing:

```rust
// FlagDependencyExportsPlugin:
let exports_info = self.mg.get_exports_info_data(&module_id).clone();

// FlagDependencyUsagePlugin:
let mut exports_info = mg.get_exports_info_data(&module_id).clone();
```

Each clone involves:
- `BTreeMap<Atom, ExportInfoData>` clone — O(exports) tree nodes
- Each `ExportInfoData` clone includes `HashMap<Option<DependencyId>, ExportInfoTargetValue>` clone
- `Option<ustr::UstrMap<UsageState>>` clone

At 10K modules with 10 exports each:
- 10K BTreeMap clones × 10 entries = 100K ExportInfoData clones
- Each ExportInfoData is ~200 bytes = **~20MB of cloned data**

**Fix**: Use per-module `RwLock` or atomic operations instead of cloning entire ExportsInfoData.

---

## BTreeMap vs HashMap for Exports

`ExportsInfoData` uses `BTreeMap<Atom, ExportInfoData>` for named exports:

```rust
exports: BTreeMap<Atom, ExportInfoData>,
```

BTreeMap has O(log n) lookup vs HashMap's O(1). For modules with many exports (e.g., barrel files with 100+ re-exports), this matters.

**Fix**: Use `HashMap<Atom, ExportInfoData>` or `IndexMap<Atom, ExportInfoData>` for O(1) lookup. BTreeMap is only needed if ordered iteration is required.

---

## Summary

| Issue | Impact | Fix |
|-------|--------|-----|
| **Per-task fresh caches in side effects computation** | O(n²) → projected 29s at 10K | Shared cache or topological ordering |
| ExportsInfoData cloning for parallel deps plugins | ~20MB cloned at 10K | Per-module locks or atomic fields |
| BTreeMap for exports (O(log n) lookup) | Minor | Switch to HashMap |
| ExportInfoData target HashMap allocation | Per-export overhead | Use SmallVec for typical 0-1 targets |
