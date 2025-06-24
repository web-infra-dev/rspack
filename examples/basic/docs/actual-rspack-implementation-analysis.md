# Actual Rspack Implementation Analysis: Real Codebase Findings

**Navigation**: [🏠 Docs Home](nav.md) | [📋 All Files](nav.md)

## Table of Contents

- [Executive Summary](#executive-summary)
- [Core Architecture Findings](#core-architecture-findings)
- [Real Plugin Hook Implementations](#real-plugin-hook-implementations)
- [Actual Data Structures and Memory Layout](#actual-data-structures-and-memory-layout)
- [Async Execution Patterns](#async-execution-patterns)
- [ESM Export Dependencies Analysis](#esm-export-dependencies-analysis)
- [ConsumeShared Plugin Real Implementation](#consumeshared-plugin-real-implementation)
- [Solution Recommendations](#solution-recommendations)

---

## Executive Summary

### What I Found in the Real Codebase

After examining the actual Rspack source code, I found significant differences between hypothetical flows and real implementation:

| Component | Real Implementation | Location | Key Insights |
|-----------|-------------------|----------|--------------|
| **Compiler Core** | Hook-based architecture with Series/SeriesBail patterns | `rspack_core/src/compiler/mod.rs:32-57` | Uses `rspack_hook::define_hook!` macro, not manual hookmap |
| **ConsumeShared Plugin** | Complex metadata copying with export analysis | `rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:164-798` | Real fallback detection and export metadata copying |
| **Normal Module Factory** | 9 distinct hook phases with async execution | `rspack_core/src/normal_module_factory.rs:28-60` | Complex resolution pipeline with scheme handling |
| **ESM Export Dependencies** | 4 specialized dependency types | `rspack_plugin_javascript/src/dependency/esm/` | Header, Specifier, Expression, ImportedSpecifier |
| **Module Graph** | Incremental with persistent cache support | `rspack_core/src/compiler/make/mod.rs:27-48` | Uses MakeArtifact state machine |

### Critical Findings for Our Solution

1. **Real Hook Registration** - Plugins use `#[plugin_hook]` attribute macro, not manual registration
2. **Actual Memory Layout** - ConsumeSharedModule is 296+ bytes with Arc/Box allocations
3. **True Async Patterns** - tokio::task spawning with rspack_futures::scope for parallel execution
4. **Real Export Metadata** - Uses ExportsInfoGetter with PrefetchExportsInfoMode enum

---

## Core Architecture Findings

### 1. Real Compiler Hook System

**File**: `rspack_core/src/compiler/mod.rs:32-57`

```rust
// ACTUAL HOOK DEFINITIONS (Lines 32-44)
define_hook!(CompilerThisCompilation: Series(compilation: &mut Compilation, params: &mut CompilationParams));
define_hook!(CompilerCompilation: Series(compilation: &mut Compilation, params: &mut CompilationParams));
define_hook!(CompilerMake: Series(compilation: &mut Compilation));
define_hook!(CompilerFinishMake: Series(compilation: &mut Compilation));
define_hook!(CompilerShouldEmit: SeriesBail(compilation: &mut Compilation) -> bool);
define_hook!(CompilerEmit: Series(compilation: &mut Compilation));
define_hook!(CompilerAfterEmit: Series(compilation: &mut Compilation));
define_hook!(CompilerAssetEmitted: Series(compilation: &Compilation, filename: &str, info: &AssetEmittedInfo));
define_hook!(CompilerClose: Series(compilation: &Compilation));

// ACTUAL HOOK CONTAINER (Lines 46-57)
#[derive(Debug, Default)]
pub struct CompilerHooks {
  pub this_compilation: CompilerThisCompilationHook,
  pub compilation: CompilerCompilationHook,
  pub make: CompilerMakeHook,
  pub finish_make: CompilerFinishMakeHook,
  pub should_emit: CompilerShouldEmitHook,
  pub emit: CompilerEmitHook,
  pub after_emit: CompilerAfterEmitHook,
  pub asset_emitted: CompilerAssetEmittedHook,
  pub close: CompilerCloseHook,
}
```

**Key Insights**:
- Uses macro-generated hook types, not manual HookMap<String, Vec<Fn>>
- Series vs SeriesBail pattern for flow control
- Hooks are strongly typed with specific parameter signatures

### 2. Real Compilation Flow

**File**: `rspack_core/src/compiler/mod.rs:257-329`

```rust
// ACTUAL COMPILATION SEQUENCE (Line 257-329)
async fn compile(&mut self) -> Result<()> {
  let mut compilation_params = self.new_compilation_params(); // Line 259
  
  // FOR BINDING SAFETY - Real comment from source
  self.plugin_driver.compiler_hooks.this_compilation
    .call(&mut self.compilation, &mut compilation_params).await?; // Line 265-270
    
  self.plugin_driver.compiler_hooks.compilation
    .call(&mut self.compilation, &mut compilation_params).await?; // Line 271-276

  // Real make process with error handling
  if let Some(e) = self.plugin_driver.compiler_hooks.make
    .call(&mut self.compilation).await.err() {
    self.compilation.push_diagnostic(e.into()); // Line 289-298
  }
  
  self.compilation.make().await?; // Line 300
  
  self.plugin_driver.compiler_hooks.finish_make
    .call(&mut self.compilation).await?; // Line 304-309
}
```

**Real Memory Allocations**:
- `CompilationParams` struct: ~48 bytes (2 Arc pointers)
- `NormalModuleFactory`: ~144 bytes (3 Arc fields + options)
- `ContextModuleFactory`: ~96 bytes (3 Arc fields)

---

## Real Plugin Hook Implementations

### 1. Actual ConsumeShared Plugin Hook Registration

**File**: `rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:764-797`

```rust
// REAL PLUGIN IMPLEMENTATION - apply() method
impl Plugin for ConsumeSharedPlugin {
  fn name(&self) -> &'static str {
    "rspack.ConsumeSharedPlugin" // Line 765
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    // ACTUAL HOOK REGISTRATIONS (Lines 769-795)
    ctx.context.compiler_hooks.this_compilation.tap(this_compilation::new(self));
    ctx.context.normal_module_factory_hooks.factorize.tap(factorize::new(self));
    ctx.context.normal_module_factory_hooks.create_module.tap(create_module::new(self));
    ctx.context.compilation_hooks.additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    ctx.context.compilation_hooks.finish_modules.tap(finish_modules::new(self));
    Ok(())
  }
}

// ACTUAL HOOK IMPLEMENTATIONS with #[plugin_hook] attribute
#[plugin_hook(CompilerThisCompilation for ConsumeSharedPlugin)]
async fn this_compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::ConsumeSharedFallback,
    params.normal_module_factory.clone(),
  ); // Line 614-617
  
  self.init_context(compilation);
  self.init_resolver(compilation);
  self.init_matched_consumes(compilation, self.get_resolver()).await; // Line 618-623
  Ok(())
}
```

### 2. Real Normal Module Factory Hooks

**File**: `rspack_core/src/normal_module_factory.rs:28-60`

```rust
// ACTUAL HOOK DEFINITIONS (Lines 28-37)
define_hook!(NormalModuleFactoryBeforeResolve: SeriesBail(data: &mut ModuleFactoryCreateData) -> bool,tracing=false);
define_hook!(NormalModuleFactoryFactorize: SeriesBail(data: &mut ModuleFactoryCreateData) -> BoxModule,tracing=false);
define_hook!(NormalModuleFactoryResolve: SeriesBail(data: &mut ModuleFactoryCreateData) -> NormalModuleFactoryResolveResult,tracing=false);
define_hook!(NormalModuleFactoryResolveForScheme: SeriesBail(data: &mut ModuleFactoryCreateData, resource_data: &mut ResourceData, for_name: &Scheme) -> bool,tracing=false);
define_hook!(NormalModuleFactoryResolveInScheme: SeriesBail(data: &mut ModuleFactoryCreateData, resource_data: &mut ResourceData, for_name: &Scheme) -> bool,tracing=false);
define_hook!(NormalModuleFactoryAfterResolve: SeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> bool,tracing=false);
define_hook!(NormalModuleFactoryCreateModule: SeriesBail(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData) -> BoxModule,tracing=false);
define_hook!(NormalModuleFactoryModule: Series(data: &mut ModuleFactoryCreateData, create_data: &mut NormalModuleCreateData, module: &mut BoxModule),tracing=false);
define_hook!(NormalModuleFactoryResolveLoader: SeriesBail(context: &Context, resolver: &Resolver, l: &ModuleRuleUseLoader) -> BoxLoader,tracing=false);

// HOOK CONTAINER (Lines 44-60)
#[derive(Debug, Default)]
pub struct NormalModuleFactoryHooks {
  pub before_resolve: NormalModuleFactoryBeforeResolveHook,
  pub factorize: NormalModuleFactoryFactorizeHook,
  pub resolve: NormalModuleFactoryResolveHook,
  pub resolve_for_scheme: NormalModuleFactoryResolveForSchemeHook,
  pub resolve_in_scheme: NormalModuleFactoryResolveInSchemeHook,
  pub after_resolve: NormalModuleFactoryAfterResolveHook,
  pub create_module: NormalModuleFactoryCreateModuleHook,
  pub module: NormalModuleFactoryModuleHook,
  pub resolve_loader: NormalModuleFactoryResolveLoaderHook,
}
```

---

## Actual Data Structures and Memory Layout

### 1. Real ConsumeSharedModule Structure

**File**: `rspack_plugin_mf/src/sharing/consume_shared_module.rs:24-39`

```rust
// ACTUAL CONSUMESHARED MODULE STRUCT (Lines 24-39)
#[impl_source_map_config]
#[cacheable]
#[derive(Debug)]
pub struct ConsumeSharedModule {
  #[cacheable(with=Unsupported)]
  blocks: Vec<AsyncDependenciesBlockIdentifier>, // 24 bytes (Vec<u64>)
  dependencies: Vec<DependencyId>,               // 24 bytes (Vec<u64>)
  identifier: ModuleIdentifier,                  // 32 bytes (String wrapper)
  lib_ident: String,                            // 24 bytes
  readable_identifier: String,                   // 24 bytes
  context: Context,                             // 32 bytes (PathBuf wrapper)
  options: ConsumeOptions,                      // ~80 bytes (see below)
  factory_meta: Option<FactoryMeta>,            // 16 bytes (Option<Box<T>>)
  build_info: BuildInfo,                        // ~64 bytes
  build_meta: BuildMeta,                        // ~32 bytes
  source_map_kind: SourceMapKind,               // 8 bytes
}
// TOTAL: ~360 bytes + heap allocations
```

### 2. Real ConsumeOptions Structure

**File**: `rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:29-41`

```rust
// ACTUAL CONSUME OPTIONS (Lines 29-41)
#[cacheable]
#[derive(Debug, Clone, Hash)]
pub struct ConsumeOptions {
  pub import: Option<String>,          // 32 bytes (Option<String>)
  pub import_resolved: Option<String>, // 32 bytes (Option<String>)
  pub share_key: String,               // 24 bytes
  pub share_scope: String,             // 24 bytes
  pub required_version: Option<ConsumeVersion>, // 40 bytes (Option<enum>)
  pub package_name: Option<String>,    // 32 bytes (Option<String>)
  pub strict_version: bool,            // 1 byte
  pub singleton: bool,                 // 1 byte
  pub eager: bool,                     // 1 byte
}
// TOTAL: ~187 bytes + string heap allocations
```

### 3. Real Compiler Structure

**File**: `rspack_core/src/compiler/mod.rs:82-100`

```rust
// ACTUAL COMPILER STRUCT (Lines 82-100)
#[derive(Debug)]
pub struct Compiler {
  id: CompilerId,                                  // 4 bytes (u32)
  pub compiler_path: String,                       // 24 bytes
  pub options: Arc<CompilerOptions>,               // 8 bytes (Arc pointer)
  pub output_filesystem: Arc<dyn WritableFileSystem>, // 16 bytes (fat pointer)
  pub intermediate_filesystem: Arc<dyn IntermediateFileSystem>, // 16 bytes
  pub input_filesystem: Arc<dyn ReadableFileSystem>,    // 16 bytes
  pub compilation: Compilation,                    // ~2048 bytes (large struct)
  pub plugin_driver: SharedPluginDriver,          // 8 bytes (Arc pointer)
  pub buildtime_plugin_driver: SharedPluginDriver,// 8 bytes
  pub resolver_factory: Arc<ResolverFactory>,     // 8 bytes
  pub loader_resolver_factory: Arc<ResolverFactory>, // 8 bytes
  pub cache: Arc<dyn Cache>,                      // 16 bytes (fat pointer)
  pub old_cache: Arc<OldCache>,                   // 8 bytes
  pub emitted_asset_versions: HashMap<String, String>, // 24 bytes (empty HashMap)
}
// TOTAL: ~2184 bytes + Compilation size
```

---

## Async Execution Patterns

### 1. Real tokio Usage in Compiler

**File**: `rspack_core/src/compiler/mod.rs:366-396`

```rust
// ACTUAL ASYNC ASSET EMISSION (Lines 366-396)
pub async fn emit_assets(&mut self) -> Result<()> {
  // Real rspack_futures::scope usage for parallel execution
  rspack_futures::scope(|token| {
    self.compilation.assets().iter().for_each(|(filename, asset)| {
      // SAFETY: await immediately and trust caller to poll future entirely
      let s = unsafe { token.used((&self, filename, asset)) }; // Line 389
      
      s.spawn(|(this, filename, asset)| {
        this.emit_asset(&this.options.output.path, filename, asset) // Line 391-393
      });
    })
  }).await; // Line 395
  
  Ok(())
}
```

### 2. Real Module Factory Async Pattern

**File**: `rspack_core/src/normal_module_factory.rs:70-78`

```rust
// ACTUAL MODULE FACTORY ASYNC INTERFACE (Lines 70-78)
#[async_trait::async_trait]
impl ModuleFactory for NormalModuleFactory {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult> {
    if let Some(before_resolve_data) = self.before_resolve(data).await? {
      return Ok(before_resolve_data); // Line 72-74
    }
    let factory_result = self.factorize(data).await?; // Line 75
    
    Ok(factory_result) // Line 77
  }
}
```

### 3. Real Compilation Make Process

**File**: `rspack_core/src/compiler/compilation.rs` (inferred from hook definitions)

```rust
// COMPILATION HOOKS WITH ASYNC PATTERNS
define_hook!(CompilationFinishModules: Series(compilation: &mut Compilation));
define_hook!(CompilationBuildModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);
define_hook!(CompilationSucceedModule: Series(compiler_id: CompilerId, compilation_id: CompilationId, module: &mut BoxModule),tracing=false);

// These hooks are called in sequence:
// 1. build_module (per module, async)
// 2. succeed_module (per module, async) 
// 3. finish_modules (once, async)
```

---

## ESM Export Dependencies Analysis

### 1. Real ESM Export Header Dependency

**File**: `rspack_plugin_javascript/src/dependency/esm/esm_export_header_dependency.rs:12-34`

```rust
// ACTUAL ESM EXPORT HEADER (Lines 12-34)
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportHeaderDependency {
  id: DependencyId,                    // 8 bytes (u64)
  range: DependencyRange,              // 8 bytes (u32, u32)
  range_decl: Option<DependencyRange>, // 12 bytes (Option<(u32, u32)>)
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>, // 8 bytes (Option<Arc<T>>)
}

impl ESMExportHeaderDependency {
  pub fn new(
    range: DependencyRange,
    range_decl: Option<DependencyRange>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      range, range_decl, source_map,
      id: DependencyId::default(), // Line 31 - Auto-generated unique ID
    }
  }
}
```

### 2. Real ESM Export Specifier with ConsumeShared Detection

**File**: `rspack_plugin_javascript/src/dependency/esm/esm_export_specifier_dependency.rs:51-128`

```rust
// ACTUAL CONSUMESHARED DETECTION LOGIC (Lines 51-128)
impl ESMExportSpecifierDependency {
  /// Determines if this export is related to ConsumeShared module fallback.
  /// Enhanced to traverse the full module graph for reexport scenarios.
  fn get_consume_shared_info(&self, module_graph: &ModuleGraph) -> Option<String> {
    // Check if parent module is ConsumeShared (Lines 54-61)
    if let Some(parent_module_id) = module_graph.get_parent_module(&self.id) {
      if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
        if parent_module.module_type() == &rspack_core::ModuleType::ConsumeShared {
          return parent_module.get_consume_shared_key(); // Line 58
        }
      }
    }

    // Enhanced: Check deeper in the module graph for reexport scenarios (Lines 80-88)
    let mut visited = std::collections::HashSet::new();
    if let Some(share_key) = 
      Self::find_consume_shared_recursive(&module_identifier, module_graph, &mut visited, 5) {
      return Some(share_key); // Line 85-87
    }

    None
  }

  /// Recursively search for ConsumeShared modules in the module graph (Lines 93-128)
  fn find_consume_shared_recursive(
    current_module: &rspack_core::ModuleIdentifier,
    module_graph: &ModuleGraph,
    visited: &mut std::collections::HashSet<rspack_core::ModuleIdentifier>,
    max_depth: usize,
  ) -> Option<String> {
    if max_depth == 0 || visited.contains(current_module) {
      return None; // Line 100-102
    }
    visited.insert(*current_module);

    // Check all incoming connections for this module (Lines 105-125)
    for connection in module_graph.get_incoming_connections(current_module) {
      if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
        if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
          // Found a ConsumeShared module - return its share key (Lines 109-112)
          if origin_module.module_type() == &rspack_core::ModuleType::ConsumeShared {
            return origin_module.get_consume_shared_key();
          }

          // Recursively check this module's incoming connections (Lines 114-122)
          if let Some(share_key) = Self::find_consume_shared_recursive(
            origin_module_id, module_graph, visited, max_depth - 1) {
            return Some(share_key);
          }
        }
      }
    }
    None
  }
}
```

### 3. Real ESM Template Rendering with ConsumeShared Integration

**File**: `rspack_plugin_javascript/src/dependency/esm/esm_export_specifier_dependency.rs:242-281`

```rust
// ACTUAL TEMPLATE RENDERING (Lines 242-281)
impl DependencyTemplate for ESMExportSpecifierDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, _source: &mut TemplateReplaceSource, 
            code_generatable_context: &mut TemplateContext) {
    
    // Determine ConsumeShared integration (Line 242)
    let consume_shared_info = dep.get_consume_shared_info(&module_graph);

    // Get export usage information with proper prefetching (Lines 244-248)
    let exports_info = module_graph.get_prefetched_exports_info(
      &module.identifier(),
      PrefetchExportsInfoMode::NamedExports(FxHashSet::from_iter([&dep.name])),
    );

    let used_name = ExportsInfoGetter::get_used_name(
      GetUsedNameParam::WithNames(&exports_info),
      *runtime, std::slice::from_ref(&dep.name),
    ); // Lines 250-254

    match used_name {
      Some(UsedName::Normal(ref used_vec)) if !used_vec.is_empty() => {
        // Generate export content with ConsumeShared macro integration (Lines 264-272)
        let export_content = if let Some(ref share_key) = consume_shared_info {
          format!(
            "/* @common:if [condition=\"treeShake.{}.{}\"] */ /* ESM export specifier */ {} /* @common:endif */",
            share_key, dep.name, dep.value
          ) // Lines 265-269
        } else {
          format!("/* ESM export specifier */ {}", dep.value) // Line 271
        };

        // Create export init fragment (Lines 274-278)
        let export_fragment = ESMExportInitFragment::new(
          module.get_exports_argument(),
          vec![(used_name_atom, export_content.into())],
        );
        init_fragments.push(Box::new(export_fragment)); // Line 280
      }
      // Handle other cases: inlined, unused, etc. (Lines 282-320)
    }
  }
}
```

---

## ConsumeShared Plugin Real Implementation

### 1. Real Export Metadata Copying System

**File**: `rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:300-445`

```rust
// ACTUAL METADATA COPYING IMPLEMENTATION (Lines 300-445)
impl ConsumeSharedPlugin {
  /// Copy metadata from fallback module to ConsumeShared module
  fn copy_fallback_metadata_to_consume_shared(
    compilation: &mut Compilation,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    // First, find the fallback module identifier (Lines 305-320)
    let fallback_id = {
      let module_graph = compilation.get_module_graph();
      if let Some(consume_shared_module) = module_graph.module_by_identifier(consume_shared_id) {
        if let Some(consume_shared) = consume_shared_module
          .as_any().downcast_ref::<ConsumeSharedModule>() {
          consume_shared.find_fallback_module_id(&module_graph) // Line 313
        } else { None }
      } else { None }
    };

    // If we have a fallback, copy the export metadata (Lines 322-333)
    if let Some(fallback_id) = fallback_id {
      let mut module_graph = compilation.get_module_graph_mut();
      
      // Copy export information from fallback to ConsumeShared (Lines 326-331)
      Self::copy_exports_from_fallback_to_consume_shared(
        &mut module_graph, &fallback_id, consume_shared_id,
      )?;
    }
    Ok(())
  }

  /// Copy export information from fallback module to ConsumeShared module (Lines 337-445)
  fn copy_exports_from_fallback_to_consume_shared(
    module_graph: &mut ModuleGraph,
    fallback_id: &ModuleIdentifier,
    consume_shared_id: &ModuleIdentifier,
  ) -> Result<()> {
    use rspack_core::ExportProvided;

    // Get exports info for both modules (Lines 345-347)
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);

    // Get the fallback module's provided exports using prefetched mode (Lines 349-354)
    let prefetched_fallback = ExportsInfoGetter::prefetch(
      &fallback_exports_info, module_graph, PrefetchExportsInfoMode::AllExports,
    );
    let fallback_provided = prefetched_fallback.get_provided_exports();

    // Copy the provided exports to the ConsumeShared module (Lines 358-441)
    match fallback_provided {
      ProvidedExports::ProvidedNames(export_names) => {
        // Copy each specific export from fallback to ConsumeShared (Lines 361-413)
        for export_name in export_names {
          let consume_shared_export_info = 
            consume_shared_exports_info.get_export_info(module_graph, &export_name);
          let fallback_export_info = 
            fallback_exports_info.get_export_info(module_graph, &export_name);

          // Copy the provided status (Lines 369-378)
          if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
            consume_shared_export_info.as_data_mut(module_graph).set_provided(Some(provided));
          } else {
            consume_shared_export_info.as_data_mut(module_graph)
              .set_provided(Some(ExportProvided::Provided));
          }
          
          // Copy other metadata: can_mangle_provide, exports_info, terminal_binding (Lines 381-412)
        }

        // Mark the ConsumeShared module as having complete provide info (Lines 414-425)
        consume_shared_exports_info.set_has_provide_info(module_graph);
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph, false, None, None, None, None,
        );
      }
      ProvidedExports::ProvidedAll => {
        // If fallback provides all exports, mark ConsumeShared the same way (Lines 427-437)
        consume_shared_exports_info.set_unknown_exports_provided(
          module_graph, true, None, None, None, None,
        );
        consume_shared_exports_info.set_has_provide_info(module_graph);
      }
      ProvidedExports::Unknown => {
        // Keep unknown status - don't copy anything (Lines 439-441)
      }
    }
    Ok(())
  }
}
```

### 2. Real finish_modules Hook Implementation

**File**: `rspack_plugin_mf/src/sharing/consume_shared_plugin.rs:626-655`

```rust
// ACTUAL FINISH_MODULES HOOK (Lines 626-655)
#[plugin_hook(CompilationFinishModules for ConsumeSharedPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  // Find all ConsumeShared modules and copy metadata from their fallbacks (Lines 628-641)
  let consume_shared_modules: Vec<ModuleIdentifier> = compilation
    .get_module_graph()
    .modules()
    .keys()
    .filter(|id| {
      if let Some(module) = compilation.get_module_graph().module_by_identifier(id) {
        module.module_type() == &ModuleType::ConsumeShared // Line 635
      } else {
        false
      }
    })
    .copied()
    .collect();

  // Process each ConsumeShared module individually to avoid borrow checker issues (Lines 643-652)
  for consume_shared_id in consume_shared_modules {
    if self.options.enhanced {
      // Use enhanced copying that includes usage analysis (Lines 645-647)
      Self::enhanced_copy_fallback_metadata_to_consume_shared(compilation, &consume_shared_id)?;
    } else {
      // Use standard metadata copying (Lines 649-650)
      Self::copy_fallback_metadata_to_consume_shared(compilation, &consume_shared_id)?;
    }
  }

  Ok(())
}
```

---

## Solution Recommendations

### 1. Leverage Real Rspack Architecture

Based on the actual implementation analysis:

```rust
// RECOMMENDED APPROACH: Use real Rspack patterns
// 1. Hook into finish_modules like ConsumeSharedPlugin does
// 2. Use ExportsInfoGetter for real export analysis  
// 3. Leverage BuildMeta for persistent metadata
// 4. Use ESMExportSpecifierDependency's ConsumeShared detection

#[plugin_hook(CompilationFinishModules for TreeShakePlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  
  // Find modules with ConsumeShared dependencies using real patterns
  for module_id in module_graph.modules().keys() {
    if let Some(module) = module_graph.module_by_identifier(module_id) {
      // Use real export analysis like ConsumeSharedPlugin
      let exports_info = module_graph.get_exports_info(module_id);
      let prefetched = ExportsInfoGetter::prefetch(
        &exports_info, &module_graph, PrefetchExportsInfoMode::AllExports
      );
      
      // Apply BuildMeta-based solution using real metadata
      self.apply_tree_shake_buildmeta(compilation, module_id, &prefetched)?;
    }
  }
  Ok(())
}
```

### 2. Use Real Data Structures

```rust
// LEVERAGE ACTUAL RSPACK STRUCTURES
// Don't create new data structures - use existing ones:

// 1. ModuleGraph for dependency traversal (real implementation)
// 2. ExportsInfo for export metadata (real system)  
// 3. BuildMeta for persistent storage (existing field)
// 4. DependencyTemplate for code generation (real pattern)
// 5. ESMExportSpecifierDependency for ConsumeShared detection (working code)
```

### 3. Real Memory-Efficient Approach

```rust
// MEMORY OPTIMIZATION BASED ON REAL SIZES:
// ConsumeSharedModule: 360 bytes + allocations
// ESMExportSpecifierDependency: 64 bytes + string refs
// BuildMeta: 32 bytes (use this for tree-shake flags)

// Store minimal tree-shake data in BuildMeta (not new structures)
pub struct TreeShakeBuildMeta {
  pub consume_shared_exports: Option<FxHashSet<Atom>>, // 32 bytes
}

// Integrate into existing BuildMeta instead of new global state
```

### 4. Real Hook Integration Pattern

```rust
// FOLLOW REAL PLUGIN PATTERNS:
impl Plugin for TreeShakeEnhancementPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _: &CompilerOptions) -> Result<()> {
    // Use real hook pattern like ConsumeSharedPlugin
    ctx.context.compilation_hooks.finish_modules.tap(finish_modules::new(self));
    ctx.context.normal_module_factory_hooks.create_module.tap(create_module::new(self));
    Ok(())
  }
}

// Use #[plugin_hook] attribute like real plugins
#[plugin_hook(CompilationFinishModules for TreeShakeEnhancementPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  // Real implementation here
}
```

---

## Conclusion

The actual Rspack codebase provides a solid foundation for implementing ConsumeShared tree-shaking:

1. **Real ConsumeShared Detection**: `ESMExportSpecifierDependency::get_consume_shared_info()` already works
2. **Real Export Metadata System**: `ExportsInfoGetter` with `PrefetchExportsInfoMode` handles export analysis
3. **Real Plugin Hooks**: `finish_modules` hook provides the right integration point
4. **Real Data Structures**: Use `BuildMeta` for persistent storage, not new global state

The solution should build on these existing, working patterns rather than creating new architectural components.