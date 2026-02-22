# Deep Dive: NormalModuleFactory & Rebuild Path

**Files**:
- `crates/rspack_core/src/normal_module_factory.rs` (~1000 lines)
- `crates/rspack_core/src/compiler/rebuild.rs` (~240 lines)
- `crates/rspack_core/src/utils/module_rules.rs`

---

## NormalModuleFactory Pipeline

Each module goes through this pipeline (inside `FactorizeTask::background_run`):

```
1. before_resolve hook → Can skip/ignore the module
2. resolve_normal_module:
   a. Parse inline loaders (!=! syntax, !/!! prefixes)
   b. Resolve the module path (filesystem resolution)
   c. calculate_module_rules — Match against all module.rules
   d. Resolve loaders for matched rules
   e. Calculate module type, layer, parser/generator options
   f. after_resolve hook
   g. create_module hook / NormalModule::new()
   h. module hook (SideEffectsFlagPlugin taps here)
3. after_factorize hook
```

### Performance Concerns

#### Module Rule Matching (Step 2c)

```rust
async fn calculate_module_rules(&self, resource_data, dependency, issuer, issuer_layer)
    -> Result<Vec<&ModuleRuleEffect>> {
    let mut rules = Vec::new();
    module_rules_matcher(&self.options.module.rules, ..., &mut rules).await?;
    Ok(rules)
}
```

`module_rules_matcher` iterates ALL configured rules and tests each against the current module. For a typical rspack config with 20-50 rules (including nested `oneOf` rules), each module goes through 20-50 regex/function tests.

At 10K modules × 30 rules = **300K rule evaluations**.

**Opportunity**: 
1. **Pre-classify by extension**: Build an extension→rules map so `.js` files skip CSS rules immediately
2. **Cache rule results per directory**: Modules in the same directory with the same extension match the same rules
3. **Short-circuit `oneOf`**: Stop after the first match in `oneOf` groups (already done, but verify nested `oneOf`)

#### Loader Resolution (Step 2d)

Each matched loader needs to be resolved:
```rust
for l in post_loaders { resolve_each(plugin_driver, &context, &loader_resolver, &l).await? }
for l in normal_loaders { resolve_each(plugin_driver, &context, &loader_resolver, &l).await? }
for l in pre_loaders { resolve_each(plugin_driver, &context, &loader_resolver, &l).await? }
```

`resolve_each` calls the loader resolver to find the loader file. For builtin loaders (like `builtin:swc-loader`), this is fast. But for npm loaders, it involves filesystem resolution.

**Opportunity**: Cache loader resolution results. The same loader name always resolves to the same path.

#### Hook Overhead

The factory pipeline calls 8+ hooks per module:
- `before_resolve`, `factorize`, `resolve`, `resolve_for_scheme`, `after_resolve`, `create_module`, `module`, `parser`, `after_factorize`

Most have `tracing=false` for performance, but the dynamic dispatch overhead remains.

**Opportunity**: For builtin-only configurations, skip hook dispatch entirely.

---

## Rebuild Path Analysis

**File**: `crates/rspack_core/src/compiler/rebuild.rs`

### Compilation Transfer During Rebuild

```rust
async fn rebuild_inner(&mut self, changed_files, deleted_files) -> Result<()> {
    let records = CompilationRecords::record(&self.compilation);
    
    // Create new compilation with incremental state
    let mut next_compilation = Compilation::new(
        // ... same options, new incremental state ...
        Incremental::new_hot(self.options.incremental),
        modified_files, removed_files,
        true,  // is_rebuild = true
    );
    
    // Transfer artifacts from old compilation
    if next_compilation.incremental.mutations_readable(IncrementalPasses::BUILD_MODULE_GRAPH) {
        next_compilation.module_executor = std::mem::take(&mut self.compilation.module_executor);
    }
    
    // Store old compilation for artifact recovery
    self.cache.store_old_compilation(Box::new(std::mem::replace(
        &mut self.compilation, next_compilation
    )));
    
    // Run the build
    self.compile().await?;
    self.compile_done().await?;
}
```

### Critical path for rebuild:

1. **`CompilationRecords::record`**: Snapshots module IDs, chunk IDs, hashes from old compilation
2. **`Compilation::new`**: Creates a fresh compilation (allocates all data structures from scratch)
3. **`cache.store_old_compilation`**: Boxes the entire old compilation (~2KB inline + heap data)
4. **`cache.before_compile`**: Memory cache recovers artifacts from old compilation
5. **`compile()`**: Runs the full pass pipeline (but passes use incremental data)

### Rebuild Performance Issues

#### Issue 1: Fresh Compilation Allocation

Even for a single-file change, `Compilation::new` allocates:
- Fresh `IdentifierMap`/`IdentifierSet` for all fields
- Fresh `StealCell` wrappers for all artifacts
- Fresh `Incremental` state

The memory cache then **copies** artifacts from the old compilation into the new one. This involves cloning large data structures.

**Opportunity**: Use a `Compilation::reset_for_rebuild()` method that reuses the existing allocation instead of creating a new one and transferring data.

#### Issue 2: `CompilationRecords::record` Cost

```rust
pub fn record(compilation: &Compilation) -> CompilationRecords {
    // Records module IDs, chunk IDs, hashes
    // Iterates ALL modules and chunks
}
```

At 10K modules, this iterates the entire module graph to record current state.

**Opportunity**: Record incrementally — only record changed items.

#### Issue 3: Full Pass Pipeline

Even with incremental data, all 21 passes run:
```rust
for pass in &passes {
    pass.run(self, cache).await?;  // Each pass checks if it has work to do
}
```

Passes that have no affected modules still:
- Create logger, start timer
- Call `before_pass` (cache hook)
- Check incremental mutations
- Call `after_pass` (cache hook)

For a single-file change where only 1-2 modules are affected, 15+ passes have nothing to do but still pay overhead.

**Opportunity**: Skip passes entirely when no mutations are relevant. Instead of checking inside the pass, check before invoking it:

```rust
for pass in &passes {
    if pass.has_work(compilation) {
        pass.run(self, cache).await?;
    }
}
```

#### Issue 4: Resolver Cache Clearing

```rust
// In build_inner:
let plugin_driver_clone = self.plugin_driver.clone();
let _guard = scopeguard::guard((), move |_| plugin_driver_clone.clear_cache(compilation_id));
```

The resolver cache is cleared at the end of each build. For rebuilds, this means re-resolving all modules on the next build.

**Opportunity**: Keep resolver cache across rebuilds, only invalidating entries for changed directories.

---

## Summary of Factory + Rebuild Opportunities

| # | Opportunity | Impact | Effort |
|---|-----------|--------|--------|
| 1 | **Pre-classify module rules by extension** | 50% fewer rule evaluations | Low |
| 2 | **Cache loader resolution results** | Skip 90% of loader resolves | Low |
| 3 | **`Compilation::reset_for_rebuild()`** | Eliminate fresh allocation cost | High |
| 4 | **Skip empty passes in rebuild** | Save 5-10ms per rebuild | Low |
| 5 | **Keep resolver cache across rebuilds** | Faster re-resolution | Medium |
| 6 | **Incremental CompilationRecords** | O(changed) instead of O(all) | Medium |
