# Deep Dive: Module Build Inner Path & Hook Dispatch Overhead

**Files**:
- `crates/rspack_core/src/normal_module.rs:414` — `NormalModule::build()`
- `crates/rspack_macros/src/hook.rs` — `define_hook!` macro expansion
- `crates/rspack_plugin_real_content_hash/src/lib.rs` — RealContentHashPlugin

---

## Module Build Pipeline (Per Module)

Each module's `build()` method runs as a `BackgroundTask` in the task loop:

```
1. no_parse check              — Regex match against module request
2. before_loaders hook         — Plugin hook (NormalModule hooks)
3. run_loaders()               — Execute the loader chain (SWC transform, etc.)
   a. process_resource         — Read file from disk
   b. pitch phase              — Run loader pitch functions (forward)
   c. normal phase             — Run loader normal functions (backward)
4. additional_data hook        — Plugin hook
5. create_source()             — Create BoxSource from loader output + source map
6. parser_and_generator.parse() — Parse the source (SWC parse + dependency scan)
7. init_build_hash()           — Compute build hash for caching
```

### Where Time Goes (Estimated for a typical React component)

| Step | Time | Notes |
|------|------|-------|
| File I/O (process_resource) | ~0.05ms | Cached by OS |
| SWC loader transform | ~0.5-2ms | JSX transform, TypeScript strip |
| create_source (source map merge) | ~0.1ms | If source maps enabled |
| SWC parse (lexer + parser) | ~0.2-0.5ms | Depends on file size |
| AST transforms (3 passes) | ~0.1-0.3ms | paren_remover, resolver, semicolons |
| Dependency scan (4 passes) | ~0.2-0.5ms | 35 plugin hooks per node |
| init_build_hash | ~0.01ms | xxhash64 of module identity |
| **Total** | **~1-4ms** | Per module in release mode |

At 10K modules: **10-40 seconds** of module building (but parallelized across N cores).

With 8 cores: **1.25-5 seconds** actual wall-clock time for the background tasks.

### Key Overhead: Module Ownership Transfer

```rust
let (mut loader_result, err) = self
    .with_ownership(
        |mut module| {
            Box::pin(async move {
                let inner = module.inner_mut();
                let (loader_result, err) = run_loaders(
                    inner.loaders.clone(),  // Clone loader Vec
                    inner.resource_data.clone(),  // Arc clone
                    Some(plugin.clone()),  // Arc clone
                    RunnerContext { ... module ... },  // Move module
                    fs,
                ).await;
                (loader_result, err)
            })
        },
        |(loader_result, _)| {
            std::mem::replace(&mut loader_result.context.module, NormalModule::Transferred)
        },
    )
    .await;
```

The `with_ownership` pattern temporarily moves the `NormalModule` into the loader context, then extracts it back. This involves:
- `Box::pin` — heap allocation for the future
- `loaders.clone()` — clones `Vec<Arc<dyn Loader>>` (Arc increments)
- Moving the module back via `std::mem::replace`

**Opportunity**: Avoid the ownership transfer by passing the module as a mutable reference. This would require the loader runner to not take ownership.

---

## Hook Dispatch System

### Generated Code from `define_hook!`

The `define_hook!` macro generates a struct with a `Vec<Box<dyn Trait>>` of tapped plugins:

```rust
// Generated for: define_hook!(CompilationBuildModule: Series(...))
pub struct CompilationBuildModuleHook {
    taps: Vec<Box<dyn CompilationBuildModule + Send + Sync>>,
    interceptors: Vec<Box<dyn Interceptor<Self> + Send + Sync>>,
}

impl CompilationBuildModuleHook {
    pub async fn call(&self, compiler_id: CompilerId, ...) -> Result<()> {
        // When tracing=false:
        for tap in &self.taps {
            tap.run(compiler_id, ...).await?;
        }
        Ok(())
        
        // When tracing=true (default):
        // Wraps in tracing::info_span
    }
}
```

### Dispatch Overhead Per Hook Call

Each hook call involves:
1. **Iterate taps vector** — typically 0-3 taps for most hooks
2. **Virtual dispatch** — `Box<dyn Trait>` → vtable lookup → function call
3. **Async await** — each tap is awaited sequentially (Series hooks)
4. **Tracing span** (if enabled) — creates a tracing instrument span

For hooks with `tracing=false` (hot hooks like `build_module`, `succeed_module`):
- Cost: ~50ns per call (vector iteration + vtable lookup)
- At 10K modules × 2 hooks (build + succeed): ~1ms total

For hooks with `tracing=true` (most hooks):
- Cost: ~200ns per call (+ tracing span creation)
- For frequently called hooks, this adds up

### Per-Module Hook Calls

During `FactorizeTask → BuildTask → BuildResultTask`:
- `NormalModuleFactoryBeforeResolve` (1 call)
- `NormalModuleFactoryFactorize` (1 call)
- `NormalModuleFactoryResolve` (1 call)
- `NormalModuleFactoryAfterResolve` (1 call)
- `NormalModuleFactoryCreateModule` (1 call)
- `NormalModuleFactoryModule` (1 call — SideEffectsPlugin taps here)
- `NormalModuleFactoryParser` (1 call)
- `NormalModuleFactoryAfterFactorize` (1 call)
- `NormalModuleBeforeLoaders` (1 call)
- `CompilationBuildModule` (1 call, tracing=false)
- `CompilationSucceedModule` (1 call, tracing=false)
- **Total: 11 hook calls per module**

At 10K modules × 11 hooks × ~100ns avg = **~11ms** of hook overhead.

**Opportunity**: 
1. For hooks with 0 taps (most hooks in builtin-only configs), the Vec iteration is still O(1) but the async machinery has overhead. Add an `is_empty()` fast-path.
2. For hooks with exactly 1 tap (common), call directly without iteration.

---

## RealContentHashPlugin Analysis

Showed 17ms in the async chunks profile (50 chunks):

```
<t> create hash regexp: 4ms     — AhoCorasick pattern compilation
<t> create ordered hashes: 4ms  — Build dependency graph of hashes
<t> old hash to new hash: 5ms   — Compute real content hashes
<t> collect hash updates: 0ms
<t> update assets: 2ms          — Apply hash replacements
```

### Key Operations

1. **AhoCorasick compilation**: Builds a multi-pattern matcher for all content hashes
2. **Hash dependency ordering**: Topological sort of hash dependencies (recursive)
3. **Hash computation**: For each asset, replaces old hashes with new content-based hashes
4. **Asset source replacement**: Uses AhoCorasick for string replacement

### Scaling

With 50 chunks producing ~60 assets:
- AhoCorasick compilation: O(total_hash_chars) — scales with chunk count
- Hash ordering: O(assets × hash_deps) — roughly quadratic in worst case
- Hash computation: O(assets × source_size) — parallel with rayon

At 500 chunks (10K module SPA):
- ~600 assets
- AhoCorasick: ~40ms (10x for 10x patterns)
- Hash ordering: ~40ms (100x for 10x assets if O(n²))
- Hash computation: ~50ms (parallel)
- **Total: ~130ms** (vs 17ms at 50 chunks)

### Opportunities

1. **Cache AhoCorasick across rebuilds**: Pattern set only changes when content hashes change
2. **Limit hash search scope**: Only search for hashes in asset types that can reference other assets (JS, CSS, HTML) — skip binary assets
3. **Use `memchr` for single-hash replacements**: When an asset has only one hash, use simple string find instead of AhoCorasick

---

## Summary

| Finding | Impact | Effort |
|---------|--------|--------|
| Module ownership transfer overhead | 1-2% of make phase | Medium |
| Hook dispatch for 0-tap hooks | ~11ms at 10K modules | Low |
| RealContentHashPlugin at 500 chunks | ~130ms | Medium |
| Source map merge in create_source | 1-3% of make phase | High |
| Vec<Arc<dyn Loader>> clone per module | <1% | Low |
