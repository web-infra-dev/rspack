# rspack_core — Performance Opportunities

**Size**: 55,113 lines of Rust across 236 files  
**Role**: Central compilation engine — module graph, chunk graph, task loop, hashing, code generation, all compilation passes  
**Impact**: Critical — this crate contains the hottest paths in the entire bundler

---

## Table of Contents

1. [Compilation Pipeline Sequential Bottleneck](#1-compilation-pipeline-sequential-bottleneck)
2. [Module Graph Allocation Patterns](#2-module-graph-allocation-patterns)
3. [Code Splitting (CodeSplitter) BigUint Scaling](#3-code-splitting-codesplitter-biguint-scaling)
4. [Hashing Pipeline Redundancy](#4-hashing-pipeline-redundancy)
5. [Code Generation Parallelism Gaps](#5-code-generation-parallelism-gaps)
6. [Task Loop (Build Module Graph) Overhead](#6-task-loop-build-module-graph-overhead)
7. [Runtime Requirements Iterative Loop](#7-runtime-requirements-iterative-loop)
8. [OverlayMap Materialization Cost](#8-overlaymap-materialization-cost)
9. [Incremental Compilation Mutex Contention](#9-incremental-compilation-mutex-contention)
10. [StealCell Pattern & Memory](#10-stealcell-pattern--memory)
11. [Compilation Struct Size](#11-compilation-struct-size)
12. [Module Graph `.modules()` Allocation on Every Call](#12-module-graph-modules-allocation-on-every-call)
13. [DashMap in Hot Paths](#13-dashmap-in-hot-paths)
14. [Pass Framework Overhead](#14-pass-framework-overhead)
15. [Chunk Graph Lookups](#15-chunk-graph-lookups)
16. [Memory GC Strategy](#16-memory-gc-strategy)

---

## 1. Compilation Pipeline Sequential Bottleneck

**File**: `crates/rspack_core/src/compilation/run_passes.rs`

The compilation executes **21 sequential passes** in a fixed order:

```rust
let passes: Vec<Box<dyn PassExt>> = vec![
    Box::new(BuildModuleGraphPhasePass),     // Phase 1
    Box::new(SealPass),                       // Phase 2 start
    Box::new(OptimizeDependenciesPass),
    Box::new(BuildChunkGraphPass),
    Box::new(OptimizeModulesPass),
    Box::new(OptimizeChunksPass),
    Box::new(OptimizeTreePass),
    Box::new(OptimizeChunkModulesPass),
    Box::new(ModuleIdsPass),
    Box::new(ChunkIdsPass),
    Box::new(AssignRuntimeIdsPass),
    Box::new(OptimizeCodeGenerationPass),
    Box::new(CreateModuleHashesPass),        // Phase 3 start
    Box::new(CodeGenerationPass),
    Box::new(RuntimeRequirementsPass),
    Box::new(CreateHashPass),
    Box::new(CreateModuleAssetsPass),
    Box::new(CreateChunkAssetsPass),
    Box::new(ProcessAssetsPass),
    Box::new(AfterProcessAssetsPass),
    Box::new(AfterSealPass),
];
```

**Opportunity**: Several passes could potentially overlap:
- `ModuleIdsPass` and `ChunkIdsPass` are independent and could run concurrently
- `AssignRuntimeIdsPass` is lightweight and could be folded into `ChunkIdsPass`
- `OptimizeModulesPass` and `OptimizeChunksPass` could potentially overlap if data dependencies allow
- `CreateModuleAssetsPass` could start while `CreateHashPass` is still processing non-asset chunks

**Impact**: Medium. At 10K modules, each pass adds measurable latency even if individually fast. Pipelining independent passes could shave 5-15% off the seal phase.

**Estimated Gain**: 5-15% of seal phase time

---

## 2. Module Graph Allocation Patterns

**File**: `crates/rspack_core/src/module_graph/mod.rs`

### `.modules()` creates a new HashMap every call

```rust
pub fn modules(&self) -> IdentifierMap<&BoxModule> {
    self.inner.modules.iter().map(|(k, v)| (*k, v)).collect()
}
```

This method is called extensively throughout the compilation. Every call allocates a new `IdentifierMap` and copies all 10,000+ entries. This is called in:

- `code_generation_pass_impl` — to get all module identifiers
- `create_module_hashes_pass_impl` — to iterate modules
- `finish_modules_inner` — for diagnostics collection
- `runtime_requirements_pass_impl` — to get module count
- Multiple other passes

**Opportunity**: Return an iterator or reference instead of collecting into a new map. The `RollbackMap` backing store could expose `iter()` directly.

**Impact**: High. At 10K modules, each call allocates ~80KB (8 bytes key + 8 bytes value pointer × 10K) and does 10K HashMap insertions. This happens dozens of times per compilation.

Similarly, `module_graph_modules()` has the same pattern:
```rust
pub fn module_graph_modules(&self) -> IdentifierMap<&ModuleGraphModule> {
    self.inner.module_graph_modules.iter().map(|(k, v)| (*k, v)).collect()
}
```

**Estimated Gain**: 1-3% of total build time (dozens of allocations of 80KB+ maps eliminated)

---

## 3. Code Splitting (CodeSplitter) BigUint Scaling

**File**: `crates/rspack_core/src/compilation/build_chunk_graph/code_splitter.rs`

The code splitter uses `BigUint` (arbitrary-precision integers) as bitmasks to track available modules:

```rust
pub(crate) ordinal_by_module: IdentifierMap<u64>,
pub(crate) mask_by_chunk: UkeyMap<ChunkUkey, BigUint>,
// ...
pub min_available_modules: Arc<BigUint>,
```

Each module gets an ordinal, and available modules are tracked as bits in a `BigUint`. At 10,000 modules:
- Each `BigUint` is 10,000 / 8 = **1,250 bytes**
- Operations like `|=`, `&`, bit testing require scanning the entire bitmask
- Every `ChunkGroupInfo` stores an `Arc<BigUint>` for `min_available_modules`
- `available_modules_to_be_merged` stores `Vec<Arc<BigUint>>` — multiple 1.25KB bitmasks per chunk group
- `calculate_resulting_available_modules` does BigUint clone + OR operation

The `process_chunk_groups_for_merging` method is particularly expensive:
```rust
fn process_chunk_groups_for_merging(&mut self, compilation: &mut Compilation) {
    // For each chunk group, merges available module sets using BigUint operations
    // This is O(num_chunk_groups × bitmask_size) per iteration
}
```

**Opportunity**:
1. Use a `BitVec` or `FixedBitSet` instead of `BigUint` — these are optimized for fixed-size bitsets and avoid arbitrary-precision overhead
2. Pre-allocate bitmasks with known capacity (number of modules is known after build phase)
3. Consider a hierarchical bitset for sparse sets (most chunks contain a small fraction of total modules)
4. Use SIMD-accelerated bitwise operations (the `bitvec` crate supports this)

**Impact**: High. Code splitting is single-threaded and processes every module through the BFS. At 10K modules, BigUint operations become a significant bottleneck due to:
- Heap allocation for each BigUint (vs stack-allocated FixedBitSet for <8192 bits)
- No SIMD optimization in BigUint's bitwise ops
- Frequent Arc cloning of 1.25KB buffers

**Estimated Gain**: 10-30% of BuildChunkGraphPass time

---

## 4. Hashing Pipeline Redundancy

**Files**:
- `crates/rspack_core/src/compilation/create_module_hashes/mod.rs`
- `crates/rspack_core/src/compilation/create_hash/mod.rs`
- `crates/rspack_core/src/compilation/code_generation/mod.rs`

The hashing pipeline involves multiple passes:

1. **CreateModuleHashesPass**: Hashes each module × each runtime → stores in `cgm_hash_artifact`
2. **CodeGenerationPass**: Uses module hashes as cache keys, then hashes each `CodeGenerationResult`
3. **CreateHashPass**: Hashes each chunk (including runtime module hashes), then creates full compilation hash

Within `CreateHashPass`, the hashing is done in stages:
```rust
// Stage 1: Hash runtime modules in non-runtime chunks (parallel)
let other_chunk_runtime_module_hashes = rspack_futures::scope(...)

// Stage 2: Hash non-runtime chunks (parallel)  
let other_chunks_hash_results = rspack_futures::scope(...)

// Stage 3: Hash runtime chunks SEQUENTIALLY (dependencies between them)
for runtime_chunk_ukey in runtime_chunks {
    // Hash runtime modules for this chunk
    // Hash the chunk itself
    // Must be sequential because later chunks depend on earlier hashes
}

// Stage 4: Create full hash from all chunk hashes
// Stage 5: Re-hash full-hash-dependent chunks
```

**Opportunity**:
1. **Merge module hashing into code generation**: The module hash is computed, then immediately used as a code generation cache key. These could be a single operation.
2. **Reduce runtime chunk sequential bottleneck**: The topological sort of runtime chunks could be optimized to maximize parallelism (process independent chains in parallel).
3. **Avoid re-hashing full-hash chunks**: The "full hash" mechanism requires rehashing some chunks after the compilation hash is known. Consider a two-phase hash where the full hash is a placeholder that gets replaced, avoiding the rehash pass.
4. **Hash streaming**: Instead of building complete hash inputs then hashing, stream data through the hasher to reduce memory pressure.

**Impact**: Medium-High. Hashing at 10K modules involves:
- 10K module hash computations (parallelized but still significant)
- Hundreds of chunk hash computations
- Sequential runtime chunk hashing (the real bottleneck)

**Estimated Gain**: 5-15% of seal phase

---

## 5. Code Generation Parallelism Gaps

**File**: `crates/rspack_core/src/compilation/code_generation/mod.rs`

Code generation splits modules into two groups:
```rust
for module_identifier in modules {
    let module = module_graph.module_by_identifier(&module_identifier)...;
    if module.get_code_generation_dependencies().is_none() {
        no_codegen_dependencies_modules.insert(module_identifier);
    } else {
        has_codegen_dependencies_modules.insert(module_identifier);
    }
}

// First batch: no dependencies (fully parallel)
self.code_generation_modules(&mut codegen_cache_counter, no_codegen_dependencies_modules).await?;
// Second batch: has dependencies (also parallel, but must wait for first batch)
self.code_generation_modules(&mut codegen_cache_counter, has_codegen_dependencies_modules).await?;
```

**Opportunity**:
1. The two-phase split is conservative. Modules with code generation dependencies only depend on *specific* other modules, not on *all* modules in the first batch. A dependency-aware scheduling could increase parallelism.
2. The `CodeGenerationJob` groups runtimes by hash to avoid redundant work, but this grouping itself involves HashMap operations per module.

Within `code_generation_modules`, jobs are created per-module-per-unique-hash:
```rust
for runtime in chunk_graph.get_module_runtimes_iter(module, ...) {
    let hash = ChunkGraph::get_module_hash(self, module, runtime)...;
    if let Some(job) = map.get_mut(hash) {
        job.runtimes.push(runtime.clone());
    } else {
        map.insert(hash.clone(), CodeGenerationJob { ... });
    }
}
```

This creates a HashMap per module to deduplicate runtimes. At 10K modules, that's 10K HashMap allocations.

**Opportunity**: Pre-allocate or use a simpler dedup strategy (e.g., sort + dedup).

**Impact**: Medium. Code generation is already parallelized, but the job creation overhead and two-phase constraint limit throughput.

**Estimated Gain**: 5-10% of CodeGenerationPass time

---

## 6. Task Loop (Build Module Graph) Overhead

**File**: `crates/rspack_core/src/utils/task_loop.rs`

The task loop uses `tokio::sync::mpsc::unbounded_channel` for background task communication:

```rust
struct TaskLoop<Ctx> {
    main_task_queue: VecDeque<Box<dyn Task<Ctx>>>,
    background_task_count: u32,
    task_result_sender: UnboundedSender<TaskResult<Ctx>>,
    task_result_receiver: UnboundedReceiver<TaskResult<Ctx>>,
}
```

Background tasks (factorize, build) are spawned via `rspack_tasks::spawn_in_compiler_context`:
```rust
fn spawn_background(&mut self, task: Box<dyn Task<Ctx>>) {
    let tx = self.task_result_sender.clone();
    self.background_task_count += 1;
    rspack_tasks::spawn_in_compiler_context(task::unconstrained(async move {
        let r = task.background_run().await;
        tx.send(r).expect("failed to send task result");
    }.in_current_span()));
}
```

**Opportunity**:
1. **Main thread becomes a bottleneck**: While background tasks (factorize/build) run in parallel, their results must be processed sequentially on the main thread (ProcessDependenciesTask, AddTask, BuildResultTask are all `TaskType::Main`). At 10K modules, the main thread processes 10K `BuildResultTask`s and 10K `ProcessDependenciesTask`s sequentially.
2. **ProcessDependenciesTask groups dependencies**: It creates one `FactorizeTask` per unique resource identifier, but the grouping itself uses HashMap operations on the main thread.
3. **BuildResultTask does significant work on main thread**: Adding modules to the graph, setting up dependencies, updating file dependencies — all sequential.

**Opportunity**: 
- Move more work from `TaskType::Main` to `TaskType::Background` where possible
- Batch main-thread operations to reduce per-task overhead
- Consider lock-free data structures for the module graph to allow concurrent writes from background tasks

**Impact**: High. The make phase (BuildModuleGraphPhasePass) is typically 40-60% of total build time for a react-10k project. Even small improvements to main-thread throughput compound across 10K modules.

**Estimated Gain**: 5-20% of make phase time

---

## 7. Runtime Requirements Iterative Loop

**File**: `crates/rspack_core/src/compilation/runtime_requirements/mod.rs`

The runtime requirements computation uses an iterative loop that calls hooks until convergence:

```rust
loop {
    runtime_requirements = runtime_requirements_mut;
    runtime_requirements_mut = RuntimeGlobals::default();
    call_hook(self, requirements, &runtime_requirements, &mut runtime_requirements_mut).await?;
    runtime_requirements_mut = runtime_requirements_mut.difference(
        requirements.intersection(runtime_requirements_mut)
    );
    if runtime_requirements_mut.is_empty() {
        break;
    } else {
        requirements.insert(runtime_requirements_mut);
    }
}
```

This loop runs for **each module × each runtime**, then for **each chunk**, then for **each entry chunk**. The loop typically converges in 2-3 iterations, but each iteration invokes plugin hooks.

**Opportunity**:
1. Pre-compute which runtime requirements trigger additional requirements (build a dependency graph of runtime globals)
2. Use the dependency graph to compute the transitive closure in one pass instead of iterating
3. Cache convergence results — many modules will have identical runtime requirements patterns

**Impact**: Low-Medium. RuntimeGlobals is a bitflag set, so operations are fast, but the hook overhead per iteration is not trivial.

**Estimated Gain**: 2-5% of RuntimeRequirementsPass time

---

## 8. OverlayMap Materialization Cost

**File**: `crates/rspack_core/src/module_graph/rollback/overlay_map.rs`

When `par_iter_mut()` is called on an `OverlayMap` with an active overlay, it must materialize ALL base entries into the overlay:

```rust
fn materialize_all(&mut self) where V: Clone {
    let cloned_missing = {
        let overlay_keys = self.overlay.as_ref()...;
        self.base.iter()
            .filter_map(|(key, value)| {
                if overlay_keys.contains_key(key) {
                    None
                } else {
                    Some((key.clone(), value.clone()))
                }
            })
            .collect::<Vec<_>>()
    };
    let overlay = self.overlay();
    for (key, value) in cloned_missing {
        overlay.entry(key).or_insert_with(|| OverlayValue::Value(value));
    }
}
```

At 10K modules, `OverlayMap<ModuleIdentifier, ModuleGraphModule>` stores ~10K entries. When `par_iter_mut` is triggered, it clones ALL `ModuleGraphModule` values that haven't been modified. `ModuleGraphModule` contains:
- `all_dependencies: Vec<DependencyId>`
- `outgoing_connections`/`incoming_connections` sets
- Multiple optional fields

**Opportunity**:
1. Avoid overlay mode when not needed — only use overlay during seal phase when rollback is possible
2. Use `UnsafeCell` with careful synchronization instead of clone-on-write for parallel iteration
3. Consider `im` (immutable data structures) crate for structural sharing instead of full clones
4. Batch overlay operations to reduce materialization frequency

**Impact**: Medium. Materialization happens during `flag_dependency_exports` and similar parallel optimization passes. At 10K modules, cloning all `ModuleGraphModule` values is significant.

**Estimated Gain**: 2-8% of OptimizeDependenciesPass time

---

## 9. Incremental Compilation Mutex Contention

**File**: `crates/rspack_core/src/incremental/mod.rs`

The incremental compilation system uses a `Mutex<Mutations>` for tracking changes:

```rust
pub struct Incremental {
    silent: bool,
    passes: AtomicU16,
    state: IncrementalState,
}

enum IncrementalState {
    Cold,
    Hot { mutations: Mutex<Mutations> },
}
```

Every mutation (module add/remove/update, chunk add/remove, hash changes) acquires this mutex:
```rust
pub fn mutations_write(&self) -> Option<MutexGuard<'_, Mutations>> {
    if let IncrementalState::Hot { mutations } = &self.state
        && self.passes().allow_write()
    {
        return Some(mutations.lock()...);
    }
    None
}
```

During parallel operations (code generation, hashing), multiple threads may contend on this mutex.

**Opportunity**:
1. Use thread-local mutation buffers that get merged at synchronization points
2. Use `parking_lot::Mutex` instead of `std::sync::Mutex` (lower overhead for short critical sections)
3. Use lock-free append-only log for mutations instead of a mutex-protected collection
4. Batch mutations — instead of recording one mutation per module, record ranges

**Impact**: Low in cold builds (no mutations tracked), Medium in hot rebuilds.

**Estimated Gain**: 1-3% of parallel phases in hot rebuilds

---

## 10. StealCell Pattern & Memory

**File**: `crates/rspack_core/src/utils/steal_cell.rs` (referenced throughout compilation)

The `StealCell` pattern is used to temporarily take ownership of artifacts:

```rust
pub build_module_graph_artifact: StealCell<BuildModuleGraphArtifact>,
pub side_effects_optimize_artifact: StealCell<SideEffectsOptimizeArtifact>,
pub module_ids_artifact: StealCell<ModuleIdsArtifact>,
// ... 15+ more StealCell fields
```

Each `steal()` call takes the value out, and it must be put back after use. This means:
- The compilation struct holds Option-like wrappers for ~15 artifacts
- Extra memory for the wrapper overhead
- Runtime panics if accessed while stolen

**Opportunity**: Consider using a pass-based architecture where each pass receives only the artifacts it needs as function parameters, avoiding the need for StealCell entirely.

**Impact**: Low. The overhead is primarily ergonomic, not performance-critical. But it does add an indirection layer.

**Estimated Gain**: Negligible for performance, significant for code clarity

---

## 11. Compilation Struct Size

**File**: `crates/rspack_core/src/compilation/mod.rs`

The `Compilation` struct is massive — it contains ~40+ fields including:
- Module graph, chunk graph, and all artifacts
- Plugin driver (Arc)
- Resolver factory (Arc)
- Multiple HashMaps, DashMaps, IdentifierMaps
- 15+ StealCell artifacts
- File/context/missing dependencies (IndexSet)
- Runtime modules map
- Code generation results

At 10K modules, this struct is multiple megabytes in size. It's passed as `&mut self` to every pass, which means:
- Poor cache locality — passes only use a fraction of the fields
- All data lives in one allocation (or scattered across heap)

**Opportunity**:
1. Split `Compilation` into focused sub-structs (e.g., `CompilationModules`, `CompilationChunks`, `CompilationArtifacts`)
2. Use `#[repr(C)]` with careful field ordering to improve cache line utilization
3. Group hot fields together (fields accessed in the same pass should be adjacent)

**Impact**: Low-Medium. Cache effects are hard to measure but compound across all passes.

**Estimated Gain**: 1-5% overall (cache locality improvement)

---

## 12. Module Graph `.modules()` Allocation on Every Call

**File**: `crates/rspack_core/src/module_graph/mod.rs`

Expanding on item #2, the pattern of collecting into a new map is pervasive:

```rust
pub fn modules(&self) -> IdentifierMap<&BoxModule> {
    self.inner.modules.iter().map(|(k, v)| (*k, v)).collect()
}

pub fn module_graph_modules(&self) -> IdentifierMap<&ModuleGraphModule> {
    self.inner.module_graph_modules.iter().map(|(k, v)| (*k, v)).collect()
}
```

These are called from:
- `create_module_hashes_pass_impl` (twice — once for modules, once for check)
- `code_generation_pass_impl` (to get module keys)
- `finish_modules_inner` (collect diagnostics)
- `process_modules_runtime_requirements` (filter modules)
- Many other places

**Opportunity**: Add `modules_iter()` and `module_graph_modules_iter()` that return iterators instead of collected maps. Most callers only need to iterate, not random-access.

For callers that do need random access, expose `module_by_identifier()` which already exists and does a direct lookup.

**Estimated Gain**: 1-3% of total build (significant allocation reduction)

---

## 13. DashMap in Hot Paths

**File**: Various

`DashMap` is used in several places:

```rust
// compilation/mod.rs
pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
import_var_map: IdentifierDashMap<RuntimeKeyMap<ImportVarMap>>,

// normal_module.rs  
cached_source_sizes: DashMap<SourceType, f64, BuildHasherDefault<FxHasher>>,
```

Also `IdentifierDashMap` is used in the module concatenation plugin for `bailout_reason_map`.

**Opportunity**: 
- `cached_source_sizes` on `NormalModule` uses DashMap but is typically accessed from a single thread — a simple `HashMap` with `RefCell` or just computing on-demand would be cheaper
- `import_var_map` could use a regular HashMap if access patterns are analyzed

**Impact**: Low. DashMap overhead is small per-access but adds up across 10K modules.

**Estimated Gain**: <1%

---

## 14. Pass Framework Overhead

**File**: `crates/rspack_core/src/compilation/pass.rs`

Each pass goes through:
```rust
async fn run(&self, compilation: &mut Compilation, cache: &mut dyn Cache) -> Result<()> {
    if !self.is_enabled(compilation) { return Ok(()); }
    let logger = compilation.get_logger("rspack.Compilation");
    let start = logger.time(self.name());
    self.before_pass(compilation, cache).await;
    let result = self.run_pass_with_cache(compilation, cache).await;
    if result.is_ok() {
        self.after_pass(compilation, cache).await;
    }
    logger.time_end(start);
    result
}
```

Each pass has:
- Logger creation (string allocation)
- `before_pass` cache hook (virtual dispatch + potential async)
- `after_pass` cache hook (same)
- Timing overhead

For lightweight passes like `SealPass`, `AssignRuntimeIdsPass`, the framework overhead can be a significant fraction of the pass time.

**Opportunity**: 
- Fold trivial passes together
- Use `#[inline]` on pass traits
- Skip timing for sub-millisecond passes

**Impact**: Low. Framework overhead is small per-pass but there are 21 passes.

**Estimated Gain**: <1%

---

## 15. Chunk Graph Lookups

**File**: `crates/rspack_core/src/chunk_graph/chunk_graph_module.rs`

`get_module_runtimes` collects a new `RuntimeSpecSet` per call:

```rust
pub fn get_module_runtimes(
    &self,
    module: ModuleIdentifier,
    chunk_by_ukey: &ChunkByUkey,
) -> RuntimeSpecSet {
    let cgm = self.expect_chunk_graph_module(module);
    let mut runtimes = RuntimeSpecSet::default();
    for chunk_ukey in &cgm.chunks {
        let chunk = chunk_by_ukey.expect_get(chunk_ukey);
        runtimes.set(chunk.runtime().clone());
    }
    runtimes
}
```

This is called for every module in:
- `create_module_hashes`
- `code_generation` (job creation)
- `process_modules_runtime_requirements`

At 10K modules, this creates 10K × 3 = 30K `RuntimeSpecSet` allocations.

**Opportunity**: Cache module-to-runtimes mapping. The chunk graph doesn't change between CreateModuleHashesPass and CreateHashPass, so this could be computed once and reused.

There's also `get_module_runtimes_iter` which avoids the collection but still iterates chunks per module.

**Estimated Gain**: 1-2% (reduced allocation in hot loops)

---

## 16. Memory GC Strategy

**File**: `crates/rspack_core/src/utils/memory_gc.rs`

The `MemoryGCStorage` uses generation-based garbage collection:

```rust
pub(crate) fn start_next_generation(&self) {
    let generation = self.generation.fetch_add(1, Ordering::Relaxed) + 1;
    self.data.retain(|_, cache_data| {
        cache_data.generation.saturating_add(self.max_generations) >= generation
    });
}
```

This iterates ALL entries in the DashMap to check generations. At 10K modules with cached data, this retention scan is O(n) and takes a DashMap write lock on each shard.

**Opportunity**:
1. Use a generation ring buffer instead of scanning all entries
2. Batch evictions — only GC when memory pressure is detected
3. Use an LRU cache with bounded size instead of generation-based eviction

**Impact**: Low for cold builds, Medium for watch mode (repeated rebuilds).

**Estimated Gain**: 1-3% of rebuild time

---

## Summary — Top 5 Opportunities by Impact

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Replace BigUint with FixedBitSet in CodeSplitter | 10-30% of chunk graph | Medium |
| 2 | Reduce main-thread bottleneck in task loop | 5-20% of make phase | High |
| 3 | Avoid `.modules()` collecting into new HashMap | 1-3% total | Low |
| 4 | Optimize hashing pipeline (merge passes, reduce sequential) | 5-15% of seal phase | High |
| 5 | Pipeline independent compilation passes | 5-15% of seal phase | High |
