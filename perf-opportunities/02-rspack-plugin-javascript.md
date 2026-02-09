# rspack_plugin_javascript — Performance Opportunities

**Size**: 39,435 lines of Rust across 151 files  
**Role**: JavaScript parsing (SWC), dependency scanning (AST walk), tree shaking (flag exports/usage/side effects), module concatenation, JS code generation  
**Impact**: Critical — this crate handles the core JavaScript compilation for every JS/TS module

---

## Table of Contents

1. [SWC Parsing & AST Transform Pipeline](#1-swc-parsing--ast-transform-pipeline)
2. [Dependency Scanning (AST Walk) Overhead](#2-dependency-scanning-ast-walk-overhead)
3. [FlagDependencyExportsPlugin Iterative Convergence](#3-flagdependencyexportsplugin-iterative-convergence)
4. [FlagDependencyUsagePlugin BFS + Clone Pattern](#4-flagdependencyusageplugin-bfs--clone-pattern)
5. [SideEffectsFlagPlugin Glob Matching](#5-sideeffectsflagplugin-glob-matching)
6. [Module Concatenation Plugin Quadratic Analysis](#6-module-concatenation-plugin-quadratic-analysis)
7. [MangleExportsPlugin Global Effect](#7-mangleexportsplugin-global-effect)
8. [JS Code Generation (ReplaceSource)](#8-js-code-generation-replacesource)
9. [Parser Plugin System Dynamic Dispatch](#9-parser-plugin-system-dynamic-dispatch)
10. [Scope Info Database Allocation](#10-scope-info-database-allocation)
11. [Expression Evaluation Framework](#11-expression-evaluation-framework)

---

## 1. SWC Parsing & AST Transform Pipeline

**File**: `crates/rspack_plugin_javascript/src/parser_and_generator/mod.rs`

The parsing pipeline for each JavaScript module consists of:

```rust
// Step 1: SWC Lexer creation
let parser_lexer = Lexer::new(Syntax::Es(...), target, SourceFileInput::new(...), Some(&comments));

// Step 2: Parse to AST
let (mut ast, tokens) = javascript_compiler.parse_with_lexer(...)

// Step 3: Three AST transforms
ast.transform(|program, context| {
    program.visit_mut_with(&mut paren_remover(Some(&comments)));       // Remove parens
    program.visit_mut_with(&mut resolver(context.unresolved_mark, ...)); // Resolve scopes
    program.visit_with(&mut semicolon::InsertedSemicolons { ... });     // Track semicolons
});

// Step 4: Dependency scanning (full AST walk)
ast.visit(|program, _| { scan_dependencies(...) })
```

At 10K modules, this runs 10K times. Each parse involves:
1. **Lexer**: Tokenizes the entire source
2. **Parser**: Builds full AST (allocates many nodes on heap)
3. **Three transform passes**: Each walks the entire AST
4. **Dependency scan**: Another full AST walk

**Opportunity**:
1. **Merge transform passes**: The `paren_remover`, `resolver`, and `InsertedSemicolons` could be combined into a single visitor pass. Each separate pass has full AST traversal overhead.
2. **Skip paren_remover for production builds**: Parenthesis removal is only needed for certain code transformations; in production mode it may be skippable.
3. **Lazy AST materialization**: Instead of building a full AST, use a streaming/lazy parser that only materializes AST nodes that are actually inspected by the dependency scanner.
4. **Token reuse**: The tokens from parsing are only used for semicolon detection. Consider extracting semicolons during parsing itself.

**Impact**: High. Parsing is the dominant cost in the make phase. At 10K modules, even a 10% improvement in per-module parse time saves significant wall-clock time.

**Estimated Gain**: 10-20% of per-module parse time (compounding to 5-10% of make phase)

---

## 2. Dependency Scanning (AST Walk) Overhead

**Files**:
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk.rs` (1700+ lines)
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_pre.rs`
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_block_pre.rs`
- `crates/rspack_plugin_javascript/src/visitors/dependency/parser/walk_module_pre.rs`

The dependency scanner walks the entire AST three times:
1. **Module pre-walk** (`walk_module_pre`): Pre-processes module declarations
2. **Block pre-walk** (`walk_block_pre`): Pre-processes block-level declarations
3. **Full walk** (`walk.rs`): Deep traversal of every node to find dependencies

For each node, the scanner invokes a **plugin drive chain**:
```rust
fn walk_statement(&mut self, statement: Statement) {
    self.enter_statement(&statement, |parser, _| {
        parser.plugin_drive.clone().statement(parser, statement).unwrap_or_default()
    }, |parser, _| match statement { ... });
}
```

The `plugin_drive.clone()` is `Rc::clone` (cheap), but every statement and expression goes through dynamic dispatch to check if any plugin wants to handle it. The plugin system has ~35 parser plugins, each with multiple hooks.

**Opportunity**:
1. **Pre-compute plugin interest**: Before scanning, build a bitset of which node types any plugin is interested in. Skip the plugin dispatch entirely for nodes nobody cares about.
2. **Combine pre-walks with main walk**: The three-pass approach (module pre-walk → block pre-walk → walk) exists for ordering reasons. Consider a single-pass design with deferred processing for order-dependent items.
3. **Fast-path for simple modules**: Many modules only have `import`/`export` statements at the top level. Detect this pattern early and use a lightweight scanner instead of the full AST walker.
4. **Plugin drive method inlining**: Many plugin hooks return `None` for most AST nodes. Consider generating specialized scanners based on which plugins are active.

**Impact**: High. The AST walk runs for every module and touches every node in the AST.

**Estimated Gain**: 5-15% of per-module scan time

---

## 3. FlagDependencyExportsPlugin Iterative Convergence

**File**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`

This plugin determines what each module exports. It works iteratively:

```rust
while !batch.is_empty() {
    let modules = std::mem::take(&mut batch);
    
    // Parallel: collect export specs from each module
    let module_exports_specs = modules.into_par_iter()
        .map(|module_id| {
            let exports_specs = collect_module_exports_specs(&module_id, self.mg, self.mg_cache);
            (module_id, exports_specs)
        })
        .collect::<Vec<_>>();
    
    // Split: non-nested (parallel) vs nested (sequential)
    let (non_nested_specs, has_nested_specs) = partition...;
    
    // Parallel: merge non-nested specs
    let non_nested_tasks = non_nested_specs.into_iter()
        .map(|(module_id, exports_specs)| {
            let exports_info = self.mg.get_exports_info_data(&module_id).clone(); // CLONE!
            // ...merge...
        })
        .par_bridge()
        .map(|...| { ... })
        .collect();
    
    // Sequential: merge nested specs (can't parallelize)
    for (module_id, exports_specs) in has_nested_specs { ... }
    
    // Backtrack: add dependent modules to next batch
    batch.extend(changed_modules...);
}
```

**Issues identified**:
1. **ExportsInfoData clone for parallel processing**: Each module's exports info is cloned for parallel merging (`self.mg.get_exports_info_data(&module_id).clone()`). At 10K modules, this is 10K clones of non-trivial data structures.
2. **Iterative convergence**: The while loop typically runs 2-4 iterations. Each iteration re-collects export specs from changed modules.
3. **`collect_module_exports_specs` walks all dependencies**: For each module, it iterates all dependencies and calls `get_exports()` on each — this duplicates work from the dependency scanning phase.
4. **Nested exports fall back to sequential**: Re-exports from CJS modules require sequential processing, which becomes a bottleneck if many modules re-export from CJS.

**Opportunity**:
1. **Cache export specs**: The export specs don't change between iterations; only the exports info data changes. Cache the specs and only re-merge changed modules.
2. **Avoid clone**: Use `UnsafeCell`-based parallel mutation or split exports info into per-module segments that can be mutated independently.
3. **Pre-compute export spec during parse**: Export specs could be computed during the initial dependency scan instead of re-walking the dependency tree.
4. **Topological ordering**: Process modules in topological order so re-exports can be resolved in a single pass.

**Impact**: Medium-High. At 10K modules, the iterative loop and per-module cloning add measurable overhead.

**Estimated Gain**: 10-30% of FlagDependencyExportsPlugin time

---

## 4. FlagDependencyUsagePlugin BFS + Clone Pattern

**File**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`

This plugin tracks how exports are used across the module graph via BFS:

```rust
loop {
    let mut batch = vec![];
    while let Some(task) = q.dequeue() { batch.push(task); }
    
    // Parallel: collect referenced exports
    let batch_res = batch.into_par_iter()
        .map(|(block_id, runtime, force_side_effects)| {
            Self::process_module(compilation, module_graph, block_id, runtime, ...)
        })
        .collect::<Vec<_>>();
    
    // Split into non-nested (parallel) and nested (sequential)
    // Non-nested: clone ExportsInfoData, process in parallel, set back
    let non_nested_res = non_nested_tasks.into_par_iter()
        .map(|(module_id, tasks)| {
            let mut exports_info = mg.get_exports_info_data(&module_id).clone(); // CLONE!
            // ...process...
        })
        .collect::<Vec<_>>();
    
    // Set back to module graph
    for (exports_info, res) in non_nested_res {
        mg.set_exports_info(exports_info.id(), exports_info);
    }
    
    // Nested: sequential processing
    for (...) in nested_tasks { ... }
}
```

**Same clone pattern as FlagDependencyExportsPlugin**. Additionally:
1. The BFS explores the entire module graph starting from entry points
2. `process_module` calls `get_referenced_exports` for each dependency, which involves:
   - Checking connection active state (requires runtime spec matching)
   - Computing referenced export names (string comparisons)
   - Building `IdentifierMap<Vec<ExtendedReferencedExport>>` per module

**Opportunity**:
1. **Shared parallel mutation**: Instead of clone → process → set_back, use per-module `RwLock` or atomic operations on exports info fields
2. **Batch referenced exports**: Instead of per-dependency referenced exports, pre-compute a merged reference set per module
3. **Cache connection active states**: `is_active()` checks are repeated across multiple passes; cache results for the same runtime

**Impact**: Medium-High. This is one of the heavier optimization passes.

**Estimated Gain**: 10-25% of FlagDependencyUsagePlugin time

---

## 5. SideEffectsFlagPlugin Glob Matching

**File**: `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs`

The plugin evaluates `sideEffects` from `package.json`:

```rust
fn get_side_effects_from_package_json(side_effects: SideEffects, relative_path: &Utf8Path) -> bool {
    match side_effects {
        SideEffects::Bool(s) => s,
        SideEffects::String(s) => glob_match_with_normalized_pattern(&s, relative_path.as_str()),
        SideEffects::Array(patterns) => patterns.iter()
            .any(|pattern| glob_match_with_normalized_pattern(pattern, relative_path.as_str())),
    }
}

fn glob_match_with_normalized_pattern(pattern: &str, string: &str) -> bool {
    let trim_start = pattern.trim_start_matches("./");
    let normalized_glob = if trim_start.contains('/') {
        trim_start.to_string()
    } else {
        String::from("**/") + trim_start
    };
    fast_glob::glob_match(&normalized_glob, string.trim_start_matches("./"))
}
```

For each module, this function:
1. Allocates a new String for the normalized glob pattern
2. Runs glob matching

At 10K modules from the same package (common in monorepos), this is 10K glob match operations with 10K string allocations.

**Opportunity**:
1. **Cache normalized patterns**: The `sideEffects` pattern from `package.json` is the same for all modules in a package. Cache the normalized pattern.
2. **Cache glob match results per directory**: Modules in the same directory will have the same relative path prefix. Cache results.
3. **Compile regex once**: Convert glob patterns to compiled regex once, reuse across all modules.

**Impact**: Low-Medium. Glob matching is fast but happens for every module.

**Estimated Gain**: 1-3% of OptimizeDependenciesPass time

---

## 6. Module Concatenation Plugin Quadratic Analysis

**File**: `crates/rspack_plugin_javascript/src/plugin/module_concatenation_plugin.rs`

The module concatenation plugin (`ModuleConcatenationPlugin`) evaluates every ESM module for scope hoisting eligibility:

```rust
// For each possible root module
for current_root in relevant_modules.iter() {
    // Try to build a concatenation configuration
    let mut config = ConcatConfiguration::new(*current_root, runtime.clone());
    
    // For each import of the root module
    let imports = Self::get_imports(mg, mg_cache, *current_root, runtime, &mut imports_cache, &module_cache);
    for imp in imports.iter() {
        // Try to add the imported module
        let problem = Self::try_to_add(
            compilation, &mut config, imp, runtime, ...
            possible_modules, &mut candidates, &mut failure_cache, &mut success_cache,
            false, &mut statistics, &mut imports_cache, &module_cache
        );
        // ...
    }
}
```

The `try_to_add` method is recursive and can explore deep import chains:
```rust
fn try_to_add(...) -> Option<Warning> {
    // Check failure cache
    // Check if already in config
    // Check chunk membership
    // Check incoming connections
    // Recursively try to add all imports of this module
    for imp in imports.iter() {
        let problem = Self::try_to_add(..., imp, ...); // RECURSIVE
    }
}
```

At 10K modules with many ESM imports, this can become **O(n²)** or worse:
- The outer loop iterates all relevant modules (~10K)
- For each root, `try_to_add` explores imports recursively
- Cache (`failure_cache`, `success_cache`) helps but doesn't eliminate repeated work across different roots

**Opportunity**:
1. **Pre-compute connected components**: Instead of trying each module as a root, first compute connected ESM components, then evaluate entire components at once.
2. **Better failure propagation**: When a module fails to concatenate, propagate this to all modules that depend on it (currently each root re-discovers the failure).
3. **Limit exploration depth**: Add a configurable depth limit for concatenation chains.
4. **Parallel evaluation**: Different root modules can be evaluated in parallel (they don't modify the module graph during analysis).

The plugin also uses `IdentifierDashMap` for `bailout_reason_map`:
```rust
pub struct ModuleConcatenationPlugin {
    bailout_reason_map: IdentifierDashMap<Arc<Cow<'static, str>>>,
}
```

**Impact**: Medium. Module concatenation is typically not the bottleneck, but at 10K ESM modules it becomes significant.

**Estimated Gain**: 10-30% of OptimizeChunkModulesPass time

---

## 7. MangleExportsPlugin Global Effect

**File**: `crates/rspack_plugin_javascript/src/plugin/mangle_exports_plugin.rs`

The mangle exports plugin **disables incremental hashing** when active:

```rust
if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "MangleExportsPlugin (optimization.mangleExports = true)",
    "it requires calculating the export names of all the modules, which is a global effect",
)
```

This means **every module hash must be recomputed** even if only one module changes. This is a rebuild-time concern but significant for watch mode.

The mangling algorithm uses `assign_deterministic_ids`:
```rust
assign_deterministic_ids(
    exports_info_caches.clone(),
    |a| comparator(a.id, &module_graph),
    |a| { /* assign names */ },
    NUMBER_OF_IDENTIFIER_START_CHARS * NUMBER_OF_IDENTIFIER_CONTINUATION_CHARS,
    NUMBER_OF_IDENTIFIER_START_CHARS,
    0,
    0,
);
```

**Opportunity**:
1. **Incremental mangling**: Track which modules' exports changed and only re-mangle those, then only re-hash affected modules.
2. **Stable mangling**: Use a hash-based naming scheme that produces the same names regardless of module order, eliminating the global effect.

**Impact**: Low for cold builds, High for watch mode (forces full re-hash).

**Estimated Gain**: Significant watch-mode improvement

---

## 8. JS Code Generation (ReplaceSource)

**File**: `crates/rspack_plugin_javascript/src/parser_and_generator/mod.rs`

Code generation uses `ReplaceSource` to apply dependency templates:

```rust
async fn generate(&self, source: &BoxSource, module: &dyn Module, generate_context: &mut GenerateContext) -> Result<BoxSource> {
    let mut source = ReplaceSource::new(source.clone());
    // Apply all dependency templates
    module.get_dependencies().iter().for_each(|dependency_id| {
        self.source_dependency(compilation, dependency_id, &mut source, &mut context)
    });
    module.get_blocks().iter().for_each(|block_id| {
        self.source_block(compilation, block_id, &mut source, &mut context)
    });
    // Render init fragments
    let result = render_init_fragments(source.boxed(), init_fragments, ...);
    Ok(result)
}
```

Each `source_dependency` call invokes a dependency template's `render` method, which typically calls `source.replace()` or `source.insert()`. At 10K modules with ~50 dependencies each, that's 500K replace operations.

**Opportunity**:
1. **Batch replacements**: Instead of one replacement per dependency, collect all replacements and apply them in a single pass over the source.
2. **Source clone avoidance**: `ReplaceSource::new(source.clone())` clones the entire source. If the source is a `CachedSource`, this can be expensive.
3. **Init fragments deduplication**: `render_init_fragments` sorts and deduplicates fragments. Pre-sort during dependency template rendering.

**Impact**: Medium. Code generation is parallelized, but the per-module cost adds up.

**Estimated Gain**: 5-10% of CodeGenerationPass time

---

## 9. Parser Plugin System Dynamic Dispatch

**Files**:
- `crates/rspack_plugin_javascript/src/parser_plugin/mod.rs`
- Various `*_plugin.rs` files in `parser_plugin/`

There are ~35 parser plugins, each implementing `JavascriptParserPlugin`. The drive system calls each plugin for each AST node:

```rust
pub fn statement(&self, parser: &mut JavascriptParser, stmt: Statement) -> Option<bool> {
    for plugin in &self.plugins {
        if let Some(result) = plugin.statement(parser, stmt) {
            return Some(result);
        }
    }
    None
}
```

For each expression, statement, call expression, etc., all plugins are checked. With 35 plugins and ~1000 nodes per module, that's 35,000 virtual dispatch calls per module, 350 million for 10K modules.

**Opportunity**:
1. **Plugin interest masks**: Pre-compute which hooks each plugin implements (most plugins only implement 2-3 hooks). Skip plugins that don't implement the current hook.
2. **Compile-time specialization**: Use proc macros to generate specialized drive functions that skip known-noop plugins.
3. **Group plugins by hook interest**: Keep separate lists for statement plugins, expression plugins, call-expression plugins, etc.

**Impact**: Medium. Each virtual dispatch is cheap (~1 indirect call), but at 350M dispatches it adds up.

**Estimated Gain**: 3-8% of parse/scan time

---

## 10. Scope Info Database Allocation

**File**: `crates/rspack_plugin_javascript/src/visitors/scope_info.rs`

The parser maintains a scope info database for tracking variable definitions:

```rust
pub struct ScopeInfoDB {
    // Arena-style storage
    scope_info_map: Vec<ScopeInfo>,
    variable_info_map: Vec<VariableInfo>,
    tag_info_map: Vec<TagInfo>,
}
```

The `definitions` field uses a tree of `DefinitionsId` that can be cloned to create child scopes:
```rust
fn in_block_scope<F>(&mut self, f: F) {
    let old_definitions = self.definitions;
    self.definitions = self.definitions_db.create_child(old_definitions);
    f(self);
    self.definitions = old_definitions;
}
```

For each block scope (function, if/else, for loop, etc.), a new child scope is created. At 10K modules with deeply nested code, this creates millions of scope entries.

**Opportunity**:
1. **Arena reuse across modules**: Clear and reuse the scope database between modules instead of allocating new Vecs
2. **Compaction**: Periodically compact the arena to reclaim space from dropped scopes
3. **Stack-based scope management**: Use a scope stack instead of a tree for linear scoping patterns

**Impact**: Low. Arena allocation is already efficient, but the cumulative allocation across 10K modules is non-trivial.

**Estimated Gain**: 1-2% of parse/scan time

---

## 11. Expression Evaluation Framework

**File**: `crates/rspack_plugin_javascript/src/utils/eval/*.rs`

The expression evaluator (`BasicEvaluatedExpression`) is used for constant folding and dead-code elimination:

```rust
// eval_binary_expr.rs, eval_call_expr.rs, eval_cond_expr.rs, etc.
```

The evaluator creates `BasicEvaluatedExpression` instances for each evaluated expression. These are heap-allocated and contain:
- Expression type
- Optional string/number/boolean values
- Optional range information
- Optional array/template information

**Opportunity**:
1. **Stack-allocate small evaluations**: Most evaluations result in simple types (bool, number). These could be stack-allocated.
2. **Cache evaluation results**: Expressions that appear multiple times (e.g., `process.env.NODE_ENV`) could be cached.

**Impact**: Low. Evaluation is typically fast per-expression.

**Estimated Gain**: <1%

---

## Summary — Top 5 Opportunities by Impact

| Rank | Opportunity | Estimated Gain | Effort |
|------|-----------|----------------|--------|
| 1 | Merge SWC transform passes (paren_remover + resolver + semicolons) | 5-10% of make phase | Medium |
| 2 | Eliminate ExportsInfoData clone in FlagDependencyExports/Usage | 10-30% of optimize deps | High |
| 3 | Pre-compute plugin interest masks for parser dispatch | 3-8% of parse/scan | Medium |
| 4 | Optimize module concatenation analysis (pre-compute components) | 10-30% of chunk modules opt | Medium |
| 5 | Fast-path for simple modules in dependency scanner | 5-15% of scan time | Medium |
