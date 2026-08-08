# rspack_plugin_runtime — Performance Opportunities

**Size**: 7,480 lines of Rust across 63 files (plus 69 `.ejs` template files)  
**Role**: Generates webpack runtime code — chunk loading, module execution, HMR, require implementation  
**Impact**: Medium — runtime module code generation runs for each entry chunk and affects the CreateHashPass

---

## Table of Contents

1. [Runtime Module Code Generation](#1-runtime-module-code-generation)
2. [Template Rendering (EJS)](#2-template-rendering-ejs)
3. [Runtime Module Hash Computation](#3-runtime-module-hash-computation)
4. [Runtime Requirement Resolution Loop](#4-runtime-requirement-resolution-loop)

---

## 1. Runtime Module Code Generation

Each runtime module generates JavaScript source code. The code generation happens in `runtime_modules_code_generation()` within `CreateHashPass`:

```rust
pub async fn runtime_modules_code_generation(&mut self) -> Result<()> {
    let results = rspack_futures::scope(|token| {
        self.runtime_modules.iter().for_each(|(id, module)| {
            s.spawn(|(compilation, id, module)| async {
                let result = module.code_generation(&mut context).await?;
                let source = result.get(&SourceType::Runtime).expect("should have source");
                Ok((*id, source.clone()))
            })
        })
    }).await;
}
```

Runtime modules are generated in parallel, which is good. However:
- Each module creates a `ModuleCodegenRuntimeTemplate` 
- The source is cloned after generation
- Template rendering involves string formatting

**Opportunity**:
1. **Cache runtime module sources**: Many runtime modules produce identical output across rebuilds (e.g., the `__webpack_require__` implementation). Cache by content hash.
2. **Pre-render static templates**: Templates that don't depend on compilation data could be pre-rendered once.

**Impact**: Low-Medium. Runtime modules are typically few (10-50 per compilation) but their hash computation is on the critical path.

**Estimated Gain**: 2-5% of CreateHashPass time

---

## 2. Template Rendering (EJS)

Runtime modules use `.ejs` templates rendered via `rspack_dojang`:

The template engine processes 69 template files. Each template may include conditional logic, loops, and variable interpolation.

**Opportunity**:
1. **Compile templates at build time**: Use proc macros to compile EJS templates into Rust code at compile time
2. **Use a faster template engine**: `rspack_dojang` may be slower than directly constructing strings with `format!` or a `String` builder

**Impact**: Low. Template rendering is fast relative to other operations.

**Estimated Gain**: <1%

---

## 3. Runtime Module Hash Computation

Runtime module hashes are computed multiple times during `CreateHashPass`:

```rust
// For non-runtime chunks (parallel):
let digest = runtime_module.get_runtime_hash(compilation, None).await?;

// For runtime chunks (sequential):
for runtime_chunk_ukey in runtime_chunks {
    let digest = runtime_module.get_runtime_hash(compilation, None).await?;
}

// For full-hash chunks (sequential):
if runtime_module.full_hash() || runtime_module.dependent_hash() {
    let digest = runtime_module.get_runtime_hash(self, None).await?;
}
```

A single runtime module may have its hash computed 2-3 times.

**Opportunity**: Cache runtime module hashes. Only recompute when the module's dependencies change. The `full_hash()` and `dependent_hash()` flags already indicate which modules need recomputation.

**Estimated Gain**: 1-3% of CreateHashPass time

---

## 4. Runtime Requirement Resolution Loop

The runtime requirements system uses an iterative loop (analyzed in `01-rspack-core.md`) that creates runtime modules as a side effect:

```rust
plugin_driver.compilation_hooks.runtime_requirement_in_tree.call(
    self, &entry_ukey, &all_runtime_requirements,
    &runtime_requirements_added, &mut runtime_requirements_to_add,
    &mut runtime_modules_to_add,
).await?;
```

Each iteration may add new runtime modules, which then have their own requirements checked.

**Opportunity**: Pre-compute the complete set of runtime modules needed for a given set of runtime requirements, avoiding the iterative discovery.

**Estimated Gain**: 1-2% of RuntimeRequirementsPass time

---

## Summary

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Cache runtime module sources across rebuilds | 2-5% of CreateHashPass | Medium |
| 2 | Cache runtime module hashes | 1-3% of CreateHashPass | Low |
| 3 | Pre-render static templates | <1% | Medium |
