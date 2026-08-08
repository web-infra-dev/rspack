# Deep Dive: Memory Allocation Patterns & Struct Sizes

This analysis examines the memory footprint of key data structures in rspack at 10K module scale, identifying allocation hot spots and memory optimization opportunities.

---

## Key Struct Size Analysis

### Compilation (~1,864 bytes)
- 45+ fields, many of which are heap-allocated containers
- 15+ `StealCell` artifacts (~24 bytes each = 360 bytes just for wrappers)
- Multiple `HashMap`/`IdentifierMap` fields (48 bytes each for the container itself)
- This is a single instance, so its size doesn't scale with module count

### NormalModuleInner (~784 bytes)
- 28 fields
- `cached_source_sizes: DashMap<SourceType, f64>` — 48 bytes per module for rarely-used concurrent map
- `diagnostics: Vec<Diagnostic>` — 24 bytes per module, usually empty
- `code_generation_dependencies: Option<Vec<BoxModuleDependency>>` — 16 bytes per module, usually None
- `presentational_dependencies: Option<Vec<BoxDependencyTemplate>>` — 16 bytes per module, usually None
- `build_info: BuildInfo` — ~200 bytes with many `HashSet<PathBuf>` fields

### ExportInfoData (~232 bytes)
- `target: HashMap<Option<DependencyId>, ExportInfoTargetValue>` — 48 bytes, often empty (no re-export)
- `used_in_runtime: Option<ustr::UstrMap<UsageState>>` — 48 bytes, often None in single-runtime builds
- The `BTreeMap` in `ExportsInfoData` adds per-export overhead from tree nodes

### ModuleGraphModule (~200 bytes est.)
- `all_dependencies: Vec<DependencyId>` — 24 bytes + 4 bytes × num_deps
- Outgoing/incoming connection sets

---

## Memory Projections at 10K Modules

| Data Structure | Count | Per-Item | Total | Notes |
|---------------|-------|----------|-------|-------|
| NormalModule | 10,000 | ~784 bytes | **~7.5 MB** | Core module data |
| ExportInfoData | ~100,000 | ~232 bytes | **~22 MB** | 10 exports/module avg |
| ExportsInfoData | 10,000 | ~150 bytes | ~1.5 MB | One per module |
| ModuleGraphModule | 10,000 | ~200 bytes | ~2 MB | |
| ChunkGraphModule | 10,000 | ~100 bytes | ~1 MB | |
| Dependencies (BoxDependency) | ~500,000 | ~200 bytes | **~95 MB** | 50 deps/module avg |
| ModuleGraphConnections | ~500,000 | ~100 bytes | **~48 MB** | One per dependency |
| DependencyParents | ~500,000 | ~40 bytes | ~19 MB | |
| Source code (BoxSource) | 10,000 | ~5,000 bytes avg | ~48 MB | Module sources in memory |
| **Total estimated** | | | **~244 MB** | |

### Breakdown by Category

```
Dependency data:     ~162 MB (66%)  — Dependencies + Connections + Parents
Module data:          ~11 MB  (5%)  — NormalModule + MGM + CGM
Export info:          ~24 MB (10%)  — ExportInfoData + ExportsInfoData
Source code:          ~48 MB (20%)  — BoxSource kept in memory
```

---

## Hot Allocation Paths

### 1. Dependency Object Allocation (500K objects)

Each dependency is a `Box<dyn Dependency>` — a heap allocation. At 50 deps/module × 10K modules:

```rust
// In BuildResultTask::main_run — on main thread!
for dependency in dependencies.into_iter() {
    module_graph.add_dependency(dependency);  // HashMap insert of Box<dyn>
    module_graph.set_parents(dependency_id, DependencyParents { ... });
}
```

**500K HashMap inserts** of boxed trait objects on the main thread.

**Opportunity**: Use an arena allocator for dependencies. Since dependencies have the same lifetime as the module graph, they could share a bump allocator:

```rust
// Instead of:
let dep = Box::new(ESMImportDependency::new(...));

// Use arena:
let dep = arena.alloc(ESMImportDependency::new(...));
```

This would:
- Reduce allocation overhead (bump allocation is O(1) vs allocator overhead)
- Improve cache locality (dependencies allocated contiguously)
- Reduce fragmentation
- Enable bulk deallocation (drop the arena instead of 500K individual frees)

**Estimated savings**: 30-50% of dependency-related allocation time, ~10% of make phase

### 2. String Allocations During Parsing

The SWC parser allocates strings for:
- Identifier names (`Atom` — interned, relatively cheap)
- String literals
- Template literal quasis
- Regular expression patterns
- Source map mappings

At 10K modules with ~100 strings each: **1M string allocations**.

SWC uses `Atom` (string interning) for identifiers, which helps. But string literals and templates are heap-allocated.

### 3. Vec Resizing in Build Results

```rust
// BuildResultTask::main_run
let mut all_dependencies = vec![];  // Starts empty, grows
// ... loop adds deps one by one
```

Without pre-allocation, `Vec` resizes multiple times (1→2→4→8→16→32→... ). Each resize copies the existing data.

**Fix**: Pre-allocate based on hint from parser:
```rust
let mut all_dependencies = Vec::with_capacity(estimated_deps);
```

### 4. HashMap Growth in Module Graph

The main module graph HashMaps grow as modules are added:
```rust
modules: RollbackMap<ModuleIdentifier, BoxModule>,
dependencies: HashMap<DependencyId, BoxDependency>,
blocks: HashMap<AsyncDependenciesBlockIdentifier, Box<AsyncDependenciesBlock>>,
```

Without pre-allocation, these resize multiple times during the build.

**Fix**: Pre-allocate with `HashMap::with_capacity_and_hasher()` based on an estimate from entry analysis or previous build.

### 5. DashMap Per-Module Overhead

Each `NormalModule` has a `DashMap<SourceType, f64>` for cached source sizes:
```rust
cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
```

A `DashMap` with default sharding creates 16 shards, each with a `RwLock` and a `HashMap`. For a cache that typically holds 1-2 entries, this is massive overhead.

At 10K modules: **10K DashMaps × 16 shards × ~64 bytes per shard = ~10 MB** of overhead for a simple cache.

**Fix**: Use a simple `HashMap` (modules are accessed single-threaded during size computation) or even an inline `[Option<(SourceType, f64)>; 2]` for the common case.

---

## Memory Optimization Opportunities

| # | Opportunity | Memory Saved | Effort |
|---|-----------|-------------|--------|
| 1 | **Arena allocator for dependencies** | ~30% less fragmentation, faster alloc | High |
| 2 | **Replace DashMap in NormalModule** | ~10 MB at 10K modules | Low |
| 3 | **Pre-allocate module graph HashMaps** | Fewer resizing copies | Low |
| 4 | **Pre-allocate Vec in BuildResultTask** | Fewer Vec resizings | Low |
| 5 | **SmallVec for ExportInfoData.target** | Avoid HashMap for 0-1 targets | Medium |
| 6 | **Inline Option fields in ExportInfoData** | Reduce padding/alignment waste | Medium |
| 7 | **Lazy source retention** | Drop source after code gen in production | Medium |

### Reducing Peak Memory

The ~244 MB peak at 10K modules is significant. For CI/CD environments with limited memory:

1. **Drop module sources after code generation**: Sources are only needed for code gen. After all code is generated, sources can be dropped, freeing ~48 MB.

2. **Compact dependency storage**: Instead of individual `Box<dyn Dependency>` allocations, use a typed arena or `Vec<DependencyKind>` (enum dispatch instead of trait objects).

3. **Compress inactive ExportInfoData**: Most `ExportInfoData` instances have many None/default fields. A compact representation for the common case would save significant memory.

---

## Impact on Performance

Memory allocation patterns affect performance through:
1. **Allocation time**: `malloc`/`free` overhead for millions of small allocations
2. **Cache locality**: Scattered heap allocations cause cache misses
3. **GC pressure**: Large heap means more work for mimalloc's background compaction
4. **Page faults**: Large working set may exceed L2/L3 cache

**Estimated performance impact of memory optimizations**: 5-10% of total build time from better cache locality and reduced allocation overhead.
