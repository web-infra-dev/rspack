# Deep Dive: ConcatenatedModule Code Generation (Scope Hoisting)

**File**: `crates/rspack_core/src/concatenated_module.rs` (3,323 lines)  
**Role**: When `ModuleConcatenationPlugin` merges ESM modules, the merged result is a `ConcatenatedModule`. Its code generation is the most complex single code path in rspack.

---

## Architecture

A `ConcatenatedModule` contains multiple inner modules merged into one scope:

```rust
pub struct ConcatenatedModule {
    inner: ConcatenatedModuleInner,
    modules: Vec<ConcatenatedInnerModule>,  // All merged modules
    root_module_ctxt: RootModuleContext,
    runtime: Option<RuntimeSpec>,
    // ...
}
```

### Code Generation Pipeline

```
1. get_modules_with_info()           — Build info map for all inner modules
2. analyze_module() [parallel]       — Parse each inner module, analyze scopes
3. Collect all used names             — Build global used names set
4. Generate unique names              — Avoid collisions across inner modules
5. source_module() per module         — Generate source with replacements
6. Assemble final ConcatSource        — Combine all module sources
```

### Step 1: get_modules_with_info — O(n × m)

For each inner module, collects:
- Module source code
- Export/import bindings
- Connection information
- Namespace object requirements

This involves repeated `module_graph` lookups for every connection of every inner module.

### Step 2: analyze_module — **Parallelized but heavy**

```rust
let tmp = rspack_futures::scope(|token| {
    arc_map.iter().for_each(|(id, info)| {
        s.spawn(|(module, compilation, runtime, id, info)| async move {
            let updated_module_info = module
                .analyze_module(compilation, info.clone(), runtime, concatenation_scope)
                .await?;
            Ok((*id, updated_module_info))
        });
    })
}).await;
```

Each `analyze_module` call:
1. **Re-parses the module source** using SWC experimental parser
2. Runs the SWC resolver
3. Walks the AST to collect:
   - All identifier declarations (function/class/var/let/const names)
   - All top-level declarations
   - Namespace object references

**Key issue**: The module source was **already parsed** during the build phase. ConcatenatedModule **re-parses it** during code generation using a different SWC parser (`swc_experimental_ecma_parser`).

### Step 3-4: Name collision avoidance

```rust
let mut all_used_names: HashSet<Atom> = RESERVED_NAMES.iter().map(|s| Atom::new(*s)).collect();
// ... collect from all modules ...

// For each inner module, ensure unique names
for (id, info) in module_to_info_map.iter_mut() {
    if let ModuleInfo::Concatenated(info) = info {
        // Generate unique internal names for this module's bindings
        // This involves iterating all declarations and checking against all_used_names
    }
}
```

At 100 concatenated modules with 50 declarations each, this is 5,000 name checks against a growing HashSet.

### Step 5-6: Source generation

For each inner module, generates source code with import/export rewriting, then assembles into a `ConcatSource`.

---

## Performance Issues

### Issue 1: Double Parsing

The biggest issue: **modules are parsed twice**.

1. **Build phase**: SWC parses the module to discover dependencies (using `swc_core::ecma::parser`)
2. **ConcatenatedModule code gen**: Re-parses the same source (using `swc_experimental_ecma_parser`)

At 10K modules where 5K are concatenated, that's 5K unnecessary re-parses.

**Fix**: Cache the AST from the build phase and reuse it in code generation. The `Program` could be stored in the module's build artifacts.

**Challenge**: The two parsers (`swc_core` vs `swc_experimental`) produce different AST types. Unifying them would eliminate the re-parse.

**Estimated savings**: 20-40% of ConcatenatedModule code gen time.

### Issue 2: Arc + Clone Pattern for Module Info Map

```rust
let arc_map = Arc::new(module_to_info_map);

// Parallel analysis
rspack_futures::scope(|token| {
    arc_map.iter().for_each(|(id, info)| {
        s.spawn(|(module, compilation, runtime, id, info)| async move {
            module.analyze_module(compilation, info.clone(), ...) // CLONE!
        });
    })
});

// Take back ownership
let mut module_to_info_map = Arc::into_inner(arc_map).expect("...");
```

The `info.clone()` inside the parallel spawn clones module info for each inner module. For large concatenated modules (100+ inner modules), this creates significant allocation pressure.

### Issue 3: RESERVED_NAMES HashSet Created Per Code Gen

```rust
let mut all_used_names: HashSet<Atom> = RESERVED_NAMES.iter().map(|s| Atom::new(*s)).collect();
```

`RESERVED_NAMES` is a static list of JS reserved words. A new `HashSet` is created and populated for every `ConcatenatedModule` code generation.

**Fix**: Use a `LazyLock<HashSet<Atom>>` to compute once.

### Issue 4: Sequential Name Resolution After Parallel Analysis

After parallel analysis, name resolution is sequential:
```rust
for (module_info_id, _) in references_info.iter() {
    // Sequential processing of all inner modules' names
}
```

This could be parallelized since each module's names can be resolved independently (conflicts are resolved by appending suffixes).

---

## Optimization Opportunities

| # | Opportunity | Savings | Effort |
|---|-----------|---------|--------|
| 1 | **Cache AST from build phase** — eliminate re-parsing | 20-40% of concat codegen | High |
| 2 | **Static RESERVED_NAMES HashSet** | Eliminates per-codegen allocation | Low |
| 3 | **Avoid info clone in parallel analysis** | Reduces allocation pressure | Medium |
| 4 | **Parallelize name resolution** | 10-20% of name resolution | Medium |
| 5 | **Unify SWC parsers** | Enables AST caching | Very High |

---

## Impact at Scale

For the react-10k benchmark with `optimization.concatenateModules: true`:
- Assume 5,000 modules are concatenated into ~200 concatenated modules
- Each concatenated module averages 25 inner modules
- Each inner module is re-parsed during code generation

**Current**: 5,000 re-parses × ~0.1ms each = ~500ms of unnecessary parsing
**After fix**: 0ms of re-parsing

This is a significant win for production builds where module concatenation is enabled.
