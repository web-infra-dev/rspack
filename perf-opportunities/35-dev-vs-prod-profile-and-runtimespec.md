# Development vs Production Profile & RuntimeSpec System Analysis

---

## Development vs Production Mode Comparison

**Same benchmark**: 531 modules (300 shared + 200 components + 30 async pages), 30 async chunks

| Phase | **Production** (ms) | **Development** (ms) | Diff | Notes |
|-------|---------------------|---------------------|------|-------|
| build module graph | 453 | 408 | -10% | Slightly less parse work in dev |
| optimize dependencies | 112 | **0** | -100% | **SideEffects skipped in dev!** |
| build chunk graph | 1,210 | 158 | -87% | Far fewer queue items in dev |
| optimize chunks | 330 | 67 | -80% | Simpler split chunks in dev |
| optimize chunk modules | 33 | 0 | -100% | **ModuleConcatenation skipped in dev!** |
| code generation | 60 | 35 | -42% | No concatenated modules in dev |
| create module hashes | 16 | 6 | -63% | Simpler hashing in dev |
| runtime requirements | 28 | 27 | -4% | Similar |
| hashing | 19 | 21 | +11% | Similar (full hash in both) |
| create chunk assets | 26 | 43 | +65% | Eval wrapping is expensive |
| process assets | 63 | 0 | -100% | **No RealContentHash in dev** |
| emit assets | 75 | 51 | -32% | Fewer assets |
| **Total** | **2,479** | **868** | **-65%** | |

### Key Observations

1. **Development mode is 2.85x faster** — but not because of smarter algorithms, because **3 expensive plugins are completely disabled**:
   - SideEffectsFlagPlugin (112ms saved)
   - ModuleConcatenationPlugin (33ms saved)
   - RealContentHashPlugin (63ms saved)

2. **BuildChunkGraph is 7.6x faster in dev** (158ms vs 1,210ms). This is because:
   - dev mode processes 3,168 queue items vs prod's 26,562 (8.4x fewer)
   - The BFS explores fewer blocks (1,221 vs 9,051)
   - This suggests that production mode's tree-shaking/export analysis creates many more active connections that the code splitter must traverse

3. **SplitChunks is 4.9x faster in dev** (67ms vs 330ms). Fewer module group candidates.

4. **Create chunk assets is SLOWER in dev** (43ms vs 26ms). The `eval()` wrapping used by `eval-cheap-module-source-map` devtool is more expensive than the production chunk rendering.

### Implication for Optimization Strategy

The biggest gains come from optimizing **production-only** paths:
- BuildChunkGraph's BFS explosion happens because tree-shaking creates more active connections
- SideEffects O(n²) only matters in production mode
- ModuleConcatenation only runs in production

**For development mode**, the make phase (build module graph) dominates at 47% of total time. Optimizations targeting the make phase (SWC pass merging, task loop improvements) have the highest impact for dev mode.

---

## RuntimeSpec / RuntimeSpecMap System Analysis

**File**: `crates/rspack_core/src/runtime.rs`

### Architecture

`RuntimeSpec` represents a set of runtime names (entry point identifiers):

```rust
pub struct RuntimeSpec {
    inner: UstrSet,      // Set of interned strings (runtime names)
    key: String,         // Pre-computed sorted key for hashing/comparison
}
```

`RuntimeSpecMap<T>` provides per-runtime storage with optimization for the common single-runtime case:

```rust
pub struct RuntimeSpecMap<T> {
    pub mode: RuntimeMode,           // Empty, SingleEntry, or Map
    pub map: RuntimeKeyMap<T>,       // FxHashMap<String, T> for multi-runtime
    pub single_runtime: Option<RuntimeSpec>,  // For single-runtime optimization
    pub single_value: Option<T>,
}
```

### Where RuntimeSpec Appears in Hot Paths

| Hot Path | Usage | Frequency |
|----------|-------|-----------|
| `get_module_runtimes` | Collect runtimes for a module | 10K modules × 3 passes |
| `code_generation` job grouping | Group runtimes by hash | 10K modules |
| `create_module_hashes` | Hash per module × runtime | 10K modules × runtimes |
| `process_modules_runtime_requirements` | Runtime requirements per runtime | 10K modules × runtimes |
| `RuntimeSpecMap::get/set` | Per-runtime data access | Millions of times |
| `get_runtime_key` | Compute string key for HashMap lookup | Millions of times |

### Performance Characteristics

**`RuntimeSpec::update_key()`** — Called on every mutation:
```rust
fn update_key(&mut self) {
    let mut ordered = self.inner.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    ordered.sort_unstable();
    self.key = ordered.join("_");  // String allocation!
}
```

Each mutation (insert, extend) allocates a new sorted string. For a project with 5 runtimes, that's a 5-element sort + string join per key update.

**`get_runtime_key()`** — Creates a new String every call:
```rust
pub fn get_runtime_key(runtime: &RuntimeSpec) -> &RuntimeKey {
    &runtime.key
}
```

This is already optimized to return a reference. The key is pre-computed. Good.

**`RuntimeSpecMap::get()`** — Optimized for single-runtime:
```rust
pub fn get(&self, runtime: &RuntimeSpec) -> Option<&T> {
    match self.mode {
        RuntimeMode::Empty => None,
        RuntimeMode::SingleEntry => {
            if is_runtime_equal(single_runtime, runtime) {
                self.single_value.as_ref()
            } else { None }
        }
        RuntimeMode::Map => self.map.get(get_runtime_key(runtime)),
    }
}
```

The single-entry fast path avoids HashMap lookup for the common case (most projects have 1 runtime). This is well-designed.

### Issues Found

**1. RuntimeSpecSet creates new String keys on every `set()`:**
```rust
pub fn set(&mut self, runtime: RuntimeSpec) {
    self.map.insert(get_runtime_key(&runtime).clone(), runtime);
    //                                       ^^^^^^^^ String clone!
}
```

`get_runtime_key` returns `&String`, then `.clone()` allocates a new `String` for the HashMap key. At 10K modules with `get_module_runtimes()` creating a `RuntimeSpecSet` each time, this is significant.

**Opportunity**: Use `Cow<str>` or intern the runtime key to avoid cloning.

**2. `RuntimeSpecMap` stores `RuntimeSpec` in `single_runtime`:**
```rust
pub single_runtime: Option<RuntimeSpec>,
```

`RuntimeSpec` contains `UstrSet` (heap-allocated) + `String` key. Even for single-runtime maps, this is a non-trivial allocation.

**Opportunity**: For single-runtime, store just the runtime key (a String) instead of the full RuntimeSpec.

**3. `RuntimeSpecMap::set()` transitions from SingleEntry to Map:**
```rust
pub fn set(&mut self, runtime: RuntimeSpec, value: T) {
    match self.mode {
        RuntimeMode::SingleEntry => {
            if is_runtime_equal(self.single_runtime.as_ref().unwrap(), &runtime) {
                self.single_value = Some(value);
            } else {
                // Transition to Map mode — allocates HashMap
                let mut map = RuntimeKeyMap::default();
                let single_runtime = self.single_runtime.take().unwrap();
                let single_value = self.single_value.take().unwrap();
                map.insert(get_runtime_key(&single_runtime).clone(), single_value);
                map.insert(get_runtime_key(&runtime).clone(), value);
                self.map = map;
                self.mode = RuntimeMode::Map;
            }
        }
        // ...
    }
}
```

The transition allocates a new HashMap and moves the single value into it. This is fine for correctness but could thrash if a map oscillates between 1 and 2 entries.

**4. `for_each_runtime` macro creates per-runtime RuntimeSpec:**
```rust
pub fn for_each_runtime(runtime: Option<&RuntimeSpec>, mut f: impl FnMut(Option<&RuntimeSpec>)) {
    match runtime {
        None => f(None),
        Some(runtime) => {
            for &r in runtime.iter() {
                let cur = RuntimeSpec::from_iter([r]);  // NEW RuntimeSpec per runtime!
                f(Some(&cur));
            }
        }
    }
}
```

For multi-runtime projects, this creates a new `RuntimeSpec` (with String allocation) for each runtime in the set, per call. Called in hot paths like module hashing.

**Opportunity**: Pre-compute individual runtime RuntimeSpecs and cache them.

---

## Quantified Impact

### RuntimeSpec allocations at 10K modules, 3 runtimes:

| Operation | Calls | Allocations | Bytes |
|-----------|-------|-------------|-------|
| get_module_runtimes → RuntimeSpecSet::set | 30K+ | 30K String clones | ~600KB |
| for_each_runtime per-runtime RuntimeSpec | 90K+ | 90K RuntimeSpec + String | ~3.6MB |
| RuntimeSpecMap transitions | ~10K | 10K HashMap allocs | ~500KB |
| **Total** | | ~130K+ allocations | ~4.7MB |

### At 10K modules, 1 runtime (most common):

Most operations hit the `SingleEntry` fast path — minimal overhead. The issues above primarily affect multi-runtime configurations (rare for most projects, but important for Module Federation).

---

## Summary

| # | Finding | Impact | Fix |
|---|---------|--------|-----|
| 1 | Production is 2.85x slower than dev (disabled plugins) | High | Optimize the 3 production-only plugins |
| 2 | BuildChunkGraph 7.6x slower in prod (more active connections) | High | FixedBitSet + reduce connection explosion |
| 3 | RuntimeSpecSet::set clones key strings | Low-Medium | Use Cow or intern |
| 4 | for_each_runtime creates per-runtime RuntimeSpec | Low-Medium | Pre-cache individual specs |
| 5 | Dev mode create_chunk_assets slower (eval wrapping) | Low | Optimize eval template |
