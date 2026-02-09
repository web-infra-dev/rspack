# Deep Dive: Incremental Compilation System & Its Limitations

**Files**:
- `crates/rspack_core/src/incremental/mod.rs` — Incremental state management
- `crates/rspack_core/src/incremental/mutations.rs` — Mutation tracking and affected module computation
- `crates/rspack_core/src/cache/` — Cache implementations (disable, memory, mixed, persistent)
- Each compilation pass's `before_pass`/`after_pass` for cache integration

---

## Architecture

The incremental compilation system tracks changes via `Mutations`:

```rust
pub enum Mutation {
    ModuleAdd { module },
    ModuleUpdate { module },
    ModuleRemove { module },
    DependencyUpdate { dependency },
    ModuleSetAsync { module },
    ModuleSetId { module },
    ModuleSetHashes { module },
    ChunkSetId { chunk },
    ChunkAdd { chunk },
    ChunkSplit { from, to },
    ChunksIntegrate { to },
    ChunkRemove { chunk },
    ChunkSetHashes { chunk },
}
```

Mutations are recorded during compilation and used by subsequent passes to determine what needs recomputation.

### Incremental Passes (13 stages)

```rust
const BUILD_MODULE_GRAPH = 1 << 0;
const FINISH_MODULES = 1 << 1;
const OPTIMIZE_DEPENDENCIES = 1 << 2;
const BUILD_CHUNK_GRAPH = 1 << 3;
const MODULE_IDS = 1 << 4;
const CHUNK_IDS = 1 << 5;
const MODULES_HASHES = 1 << 6;
const MODULES_CODEGEN = 1 << 7;
const MODULES_RUNTIME_REQUIREMENTS = 1 << 8;
const CHUNKS_RUNTIME_REQUIREMENTS = 1 << 9;
const CHUNKS_HASHES = 1 << 10;
const CHUNK_ASSET = 1 << 11;
const EMIT_ASSETS = 1 << 12;
```

---

## Current Limitations

### 1. BUILD_CHUNK_GRAPH Is Disabled

```rust
// crates/rspack_core/src/compilation/build_chunk_graph/mod.rs
pub fn build_chunk_graph(compilation: &mut Compilation) -> rspack_error::Result<()> {
    // TODO: heuristic incremental update is temporarily disabled
    let enable_incremental = false;
    // ...
}
```

The chunk graph is rebuilt from scratch every time. This means:
- All code splitting runs fully on every rebuild
- At 10K modules, BuildChunkGraph takes ~10ms (small) but ALL downstream passes (hashing, codegen, chunk assets) must also fully rerun

### 2. Global-Effect Plugins Disable Incremental Passes

Several plugins force full recomputation:

```rust
// MangleExportsPlugin disables MODULES_HASHES
compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "MangleExportsPlugin (optimization.mangleExports = true)",
    ...
);

// Hash-dependent filenames disable CHUNK_ASSET
if filename.has_hash_placeholder() {
    compilation.incremental.disable_passes(
        IncrementalPasses::CHUNK_ASSET,
        "Chunk filename that dependent on full hash",
        ...
    );
}

// Full-hash chunks disable CHUNKS_HASHES
if !full_hash_chunks.is_empty() {
    compilation.incremental.disable_passes(
        IncrementalPasses::CHUNKS_HASHES,
        ...
    );
}
```

In a typical production config with `optimization.mangleExports: true` and hash-based filenames, **three incremental passes are disabled**, forcing full recomputation of:
- All module hashes
- All chunk hashes
- All chunk assets

### 3. Affected Module Computation Is Expensive

```rust
pub fn get_affected_modules_with_module_graph(&self, module_graph: &ModuleGraph) -> IdentifierSet {
    self.affected_modules_with_module_graph.get_or_init(|| {
        compute_affected_modules_with_module_graph(module_graph, built_modules, built_dependencies)
    }).clone()
}
```

The `compute_affected_modules_with_module_graph` function walks the dependency graph from changed modules to find all affected modules. This is O(affected × avg_connections) which can be large for deeply connected graphs.

The result is cached in `OnceCell` but **cloned** every time it's accessed (returns `IdentifierSet` by value). At 10K modules, if 100 modules changed, the affected set might contain 500+ modules, and this set is cloned for each pass that reads it.

### 4. Mutex for Mutations

```rust
pub struct Incremental {
    state: IncrementalState,
}

enum IncrementalState {
    Cold,
    Hot { mutations: Mutex<Mutations> },
}
```

All mutation recording goes through a `Mutex`. During parallel operations (code generation, hashing), threads contend on this mutex.

### 5. Cold Build Never Uses Incremental

```rust
// crates/rspack_core/src/compiler/mod.rs
async fn build_inner(&mut self) -> Result<()> {
    // ...
    let _is_hot = self.cache.before_compile(&mut self.compilation).await;
    // TODO: disable it for now, enable it once persistent cache is added to all artifacts
    // if is_hot {
    //   self.compilation.incremental = Incremental::new_hot(self.options.incremental);
    // }
    // ...
}
```

Even with persistent cache, **incremental is not enabled for cold builds!** The TODO comment suggests this is a known issue.

---

## Impact on Watch Mode Performance

In watch mode (rebuild after file change):

### Best Case (no global-effect plugins)
A single file change at 10K modules:
1. BUILD_MODULE_GRAPH: Only rebuild the changed module + affected deps (~2-10 modules)
2. FINISH_MODULES: Only re-analyze affected modules
3. OPTIMIZE_DEPENDENCIES: Only re-optimize affected modules  
4. BUILD_CHUNK_GRAPH: **Full rebuild** (disabled)
5. MODULE_IDS: Incremental (only changed)
6. CHUNK_IDS: Incremental
7. MODULES_HASHES: Incremental (only affected modules)
8. MODULES_CODEGEN: Incremental (only affected modules)
9. RUNTIME_REQUIREMENTS: Incremental
10. CHUNKS_HASHES: Incremental
11. CHUNK_ASSET: Incremental
12. EMIT_ASSETS: Incremental (version-based diff)

### Worst Case (production config)
With `mangleExports: true` and `[contenthash]` filenames:
1-4: Same as above
5-12: **ALL FULL REBUILD** due to disabled passes

This means a production rebuild at 10K modules may still take **seconds** even for a one-line change.

---

## Optimization Opportunities

### 1. Enable Incremental BUILD_CHUNK_GRAPH

The code has the infrastructure for incremental chunk graph updates but it's disabled. The `CodeSplitter` already tracks `chunk_caches` and `edges`:

```rust
pub(crate) edges: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,
pub(crate) chunk_caches: HashMap<AsyncDependenciesBlockIdentifier, ChunkCreateData>,
```

Enabling this would allow most rebuilds to skip the full BFS traversal.

**Impact**: Saves 10-50ms at 10K modules per rebuild.

### 2. Stable Mangle Exports

Make MangleExportsPlugin use content-hash-based naming:
```rust
// Instead of: deterministic IDs that change with module order
assign_deterministic_ids(...)

// Use: hash-based names that only change when exports change
fn hash_based_mangle(export_name: &str, module_hash: &str) -> String {
    let hash = xxhash(&format!("{module_hash}:{export_name}"));
    encode_as_identifier(hash)
}
```

This eliminates the global effect, keeping MODULES_HASHES incremental.

### 3. Avoid Affected Set Cloning

```rust
// Before: clone IdentifierSet every time
pub fn get_affected_modules_with_module_graph(...) -> IdentifierSet {
    self.affected_modules_with_module_graph.get_or_init(|| ...).clone()
}

// After: return reference
pub fn get_affected_modules_with_module_graph(...) -> &IdentifierSet {
    self.affected_modules_with_module_graph.get_or_init(|| ...)
}
```

### 4. Per-Thread Mutation Buffers

Replace the single `Mutex<Mutations>` with thread-local buffers:

```rust
thread_local! {
    static LOCAL_MUTATIONS: RefCell<Vec<Mutation>> = RefCell::new(Vec::new());
}

// At sync points, merge into global mutations
fn flush_local_mutations(incremental: &Incremental) {
    LOCAL_MUTATIONS.with(|local| {
        let mutations = local.borrow_mut().drain(..).collect::<Vec<_>>();
        if let Some(mut global) = incremental.mutations_write() {
            global.extend(mutations);
        }
    });
}
```

### 5. Enable Persistent Cache for Cold Builds

Uncomment and implement:
```rust
if is_hot {
    self.compilation.incremental = Incremental::new_hot(self.options.incremental);
}
```

This would allow cold builds with persistent cache to benefit from incremental compilation.

---

## Impact Projection

| Fix | Rebuild Improvement | Effort |
|-----|-------------------|--------|
| Enable incremental BUILD_CHUNK_GRAPH | 10-50ms/rebuild | Medium |
| Stable mangle exports | 30-50% of seal phase | High |
| Avoid affected set cloning | 1-5ms/rebuild | Low |
| Per-thread mutation buffers | 2-10ms/rebuild | Medium |
| Enable persistent cache for cold builds | 50-80% of cold rebuild | Medium |
