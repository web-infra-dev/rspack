# Rspack Module Federation Sharing System - Complete Overview

## Executive Summary

The Rspack Module Federation sharing system implements a sophisticated architecture for sharing modules between micro-frontends with advanced tree-shaking capabilities, export/import tracking, and runtime optimization. The system consists of provider and consumer plugins that coordinate to enable seamless module sharing across applications.

## Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────────────┐
│                 MODULE FEDERATION SHARING SYSTEM                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ ConsumeShared   │  │ ProvideShared   │  │ ShareRuntime    │ │
│  │ Plugin          │  │ Plugin          │  │ Plugin          │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│           │                      │                      │      │
│           ▼                      ▼                      ▼      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ ConsumeShared   │  │ ProvideShared   │  │ Runtime         │ │
│  │ Modules         │  │ Modules         │  │ Code Gen        │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│           │                      │                      │      │
│           ▼                      ▼                      ▼      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Fallback        │  │ Module          │  │ JavaScript      │ │
│  │ Dependencies    │  │ Factories       │  │ Loaders         │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

           ┌─────────────────────────────────────┐
           │        ENHANCED ANALYSIS             │
           ├─────────────────────────────────────┤
           │  ┌─────────────────┐  ┌─────────────┐ │
           │  │ Enhanced Share  │  │ Export      │ │
           │  │ Usage Plugin    │  │ Usage       │ │
           │  │                 │  │ Analysis    │ │
           │  └─────────────────┘  └─────────────┘ │
           │           │                  │        │
           │           ▼                  ▼        │
           │  ┌─────────────────┐  ┌─────────────┐ │
           │  │ Tree-Shaking    │  │ Usage       │ │
           │  │ Annotations     │  │ Tracking    │ │
           │  └─────────────────┘  └─────────────┘ │
           └─────────────────────────────────────┘
```

## Complete Sharing Flow Analysis

### Phase 1: Configuration and Setup

#### Plugin Registration Sequence
```rust
// 1. ConsumeSharedPlugin registration
impl Plugin for ConsumeSharedPlugin {
    fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
        ctx.context.compiler_hooks.this_compilation.tap(this_compilation::new(self));
        ctx.context.normal_module_factory_hooks.factorize.tap(factorize::new(self));
        ctx.context.normal_module_factory_hooks.create_module.tap(create_module::new(self));
        ctx.context.compilation_hooks.finish_modules.tap(finish_modules::new(self));
    }
}

// 2. ProvideSharedPlugin registration  
impl Plugin for ProvideSharedPlugin {
    fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
        ctx.context.compiler_hooks.this_compilation.tap(this_compilation::new(self));
        ctx.context.normal_module_factory_hooks.module.tap(normal_module_factory_module::new(self));
        ctx.context.compilation_hooks.finish_make.tap(finish_make::new(self));
    }
}
```

#### Configuration Resolution Pattern
```
┌─────────────────────────────────────────────────────────────────┐
│                 CONSUME CONFIGURATION RESOLUTION                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Input: ["lodash", "react", "@company/utils/", "./local.js"]    │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Unresolved      │  │ Prefixed        │  │ Resolved        │ │
│  │ Packages        │  │ Patterns        │  │ Paths           │ │
│  │                 │  │                 │  │                 │ │
│  │ "lodash"        │  │ "@company/      │  │ "./local.js" →  │ │
│  │ "react"         │  │ utils/"         │  │ /abs/path.js    │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 2: Import Interception and Module Creation

#### Import Resolution Flow
```
Application Code: import { debounce } from 'lodash';
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FACTORIZE HOOK                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Check Unresolved Patterns                                  │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ if consumes.unresolved.get("lodash") {              │   │
│     │   create_consume_shared_module(...)                 │   │
│     │ }                                                   │   │
│     └─────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  2. Check Prefix Patterns                                      │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ for (prefix, options) in consumes.prefixed {        │   │
│     │   if request.starts_with(prefix) {                  │   │
│     │     create_dynamic_consume_shared_module(...)       │   │
│     │   }                                                 │   │
│     │ }                                                   │   │
│     └─────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  3. Create ConsumeSharedModule                                 │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ ConsumeSharedModule {                            │   │
│     │   share_key: "lodash",                           │   │
│     │   share_scope: "default",                       │   │
│     │   fallback: "./node_modules/lodash/index.js",   │   │
│     │   version: "^4.17.0",                           │   │
│     │   singleton: true,                              │   │
│     │   eager: false                                  │   │
│     │ }                                               │   │
│     └─────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 3: Module Building and Fallback Resolution

#### ConsumeShared Module Structure
```rust
impl Module for ConsumeSharedModule {
    async fn build(&mut self, _: BuildContext, _: Option<&Compilation>) -> Result<BuildResult> {
        let mut blocks = vec![];
        let mut dependencies = vec![];
        
        if let Some(fallback) = &self.options.import {
            let dep = Box::new(ConsumeSharedFallbackDependency::new(fallback.to_owned()));
            
            if self.options.eager {
                // Eager loading: direct dependency
                dependencies.push(dep as BoxDependency);
            } else {
                // Lazy loading: async dependency block
                let block = AsyncDependenciesBlock::new(
                    self.identifier, None, None, vec![dep], None
                );
                blocks.push(Box::new(block));
            }
        }
        
        Ok(BuildResult { dependencies, blocks, ..Default::default() })
    }
}
```

#### Fallback Dependency Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                    FALLBACK RESOLUTION                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ConsumeShared Request: "lodash"                               │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Fallback Configuration                                  │   │
│  │ ┌─────────────────────────────────────────────────────┐ │   │
│  │ │ import: "lodash"                                    │ │   │
│  │ │ ├─ Resolver: NormalModuleFactory                    │ │   │
│  │ │ ├─ Resolution: "./node_modules/lodash/index.js"     │ │   │
│  │ │ └─ Dependency: ConsumeSharedFallbackDependency      │ │   │
│  │ └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Loading Strategy                                        │   │
│  │ ┌─────────────────┐  ┌─────────────────────────────────┐ │   │
│  │ │ Eager Loading   │  │ Lazy Loading                    │ │   │
│  │ │ ├─ Direct Dep   │  │ ├─ AsyncDependenciesBlock       │ │   │
│  │ │ └─ Sync Access  │  │ └─ Promise-based Loading        │ │   │
│  │ └─────────────────┘  └─────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 4: Export Metadata Analysis and Copying

#### Export Information Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                  EXPORT METADATA COPYING                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Source: Fallback Module (lodash/index.js)                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ ExportsInfo                                             │   │
│  │ ├─ ProvidedExports::ProvidedNames([                     │   │
│  │ │   "debounce", "throttle", "map", "filter", ...       │   │
│  │ │ ])                                                    │   │
│  │ ├─ ExportInfo("debounce") {                             │   │
│  │ │   ├─ provided: Some(true)                             │   │
│  │ │   ├─ can_mangle: Some(true)                           │   │
│  │ │   ├─ usage_state: Unknown                             │   │
│  │ │   └─ terminal_binding: true                           │   │
│  │ │ }                                                     │   │
│  │ └─ ... (other exports)                                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                    │                                            │
│                    ▼ COPY METADATA                              │
│  Target: ConsumeShared Module (lodash)                         │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ ExportsInfo                                             │   │
│  │ ├─ Copies all provided export names                     │   │
│  │ ├─ Copies mangle capabilities                           │   │
│  │ ├─ Copies nested export information                     │   │
│  │ ├─ Sets has_provide_info = true                         │   │
│  │ └─ Sets unknown_exports_provided = false                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Phase 5: Runtime Code Generation

#### ConsumeShared Runtime Generation
```javascript
// Generated Runtime Code Example
__webpack_require__.consumesLoadingData = {
  chunkMapping: {
    "main": ["default-lodash", "default-react"]
  },
  moduleIdToConsumeDataMapping: {
    "./node_modules/lodash/index.js": {
      shareScope: "default",
      shareKey: "lodash", 
      import: false,
      requiredVersion: "^4.17.0",
      strictVersion: false,
      singleton: true,
      eager: false,
      fallback: function() {
        return __webpack_require__("./node_modules/lodash/index.js");
      }
    }
  },
  initialConsumes: []
};
```

#### Runtime Loader Selection
```javascript
// Dynamic Loader Function Selection
var resolveHandler = function(data) {
  var strict = data.strictVersion;
  var singleton = data.singleton; 
  var versionCheck = !!data.requiredVersion;
  var fallback = !!data.fallback;
  
  // Select appropriate loader based on configuration
  if (strict && singleton && versionCheck && fallback) 
    return loadStrictSingletonVersionCheckFallback;
  if (strict && versionCheck && fallback) 
    return loadStrictVersionCheckFallback;
  if (singleton && versionCheck && fallback)
    return loadSingletonVersionCheckFallback;
  if (versionCheck && fallback)
    return loadVersionCheckFallback;
  if (singleton && fallback)
    return loadSingletonFallback;
  if (fallback)
    return loadFallback;
  if (strict && singleton && versionCheck)
    return loadStrictSingletonVersionCheck;
  if (strict && versionCheck)
    return loadStrictVersionCheck;
  if (singleton && versionCheck)
    return loadSingletonVersionCheck;
  if (versionCheck)
    return loadVersionCheck;
  if (singleton)
    return loadSingleton;
  return load;
};
```

## Export and Import Determination Logic

### Enhanced Usage Analysis Architecture

#### Usage Detection Through Incoming Connections
```rust
fn analyze_usage_through_incoming_connections(
  &self,
  module_graph: &ModuleGraph,
  consume_shared_id: &ModuleIdentifier,
  runtimes: &[RuntimeSpec],
) -> Result<(Vec<String>, Vec<String>)> {
  let mut imported_exports = Vec::new();
  let mut actually_used_exports = Vec::new();
  
  // Get all incoming connections to the ConsumeShared module
  let connections: Vec<_> = module_graph
    .get_incoming_connections(consume_shared_id)
    .collect();
  
  for connection in connections {
    // Check if connection is active for current runtime
    let connection_active = match connection.active_state(
      module_graph, 
      runtimes.first(), 
      &Default::default()
    ) {
      ConnectionState::Active(active) => active,
      ConnectionState::TransitiveOnly => true,
      ConnectionState::CircularConnection => false,
    };
    
    if connection_active {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Extract referenced exports from the dependency
        let referenced_exports = dependency.get_referenced_exports(
          module_graph, 
          &ModuleGraphCacheArtifact::default(), 
          None
        );
        
        // Process each export reference
        for export_ref in referenced_exports {
          match export_ref {
            ExtendedReferencedExport::Array(names) => {
              // Specific named exports
              for name in names {
                imported_exports.push(name.to_string());
                // Determine if actually used based on usage state
                if is_export_used(module_graph, consume_shared_id, &name, runtimes) {
                  actually_used_exports.push(name.to_string());
                }
              }
            }
            ExtendedReferencedExport::Export(export_info) => {
              // Export info structure with usage details
              process_export_info_usage(export_info, &mut imported_exports, &mut actually_used_exports);
            }
          }
        }
      }
    }
  }
  
  Ok((imported_exports, actually_used_exports))
}
```

### Tree-Shaking Integration Flow

#### Export State Classification
```
┌─────────────────────────────────────────────────────────────────┐
│                  EXPORT STATE CLASSIFICATION                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ConsumeShared Module: "lodash"                                │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Export Analysis                                         │   │
│  │ ┌─────────────────┐  ┌─────────────────────────────────┐ │   │
│  │ │ Provided        │  │ Usage Analysis                  │ │   │
│  │ │ Exports         │  │                                 │ │   │
│  │ │                 │  │ ┌─────────────────────────────┐ │ │   │
│  │ │ ✓ debounce      │  │ │ Imported: ["debounce",      │ │ │   │
│  │ │ ✓ throttle      │  │ │          "map", "filter"]   │ │ │   │
│  │ │ ✓ map           │  │ │                             │ │ │   │
│  │ │ ✓ filter        │  │ │ Used: ["debounce"]          │ │ │   │
│  │ │ ✓ chunk         │  │ │                             │ │ │   │
│  │ │ ✓ ... (200+)    │  │ │ Unused: ["map", "filter"]   │ │ │   │
│  │ └─────────────────┘  │ └─────────────────────────────┘ │ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                    │                                            │
│                    ▼                                            │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ Tree-Shaking Annotations                                │   │
│  │ ┌─────────────────────────────────────────────────────┐ │   │
│  │ │ {                                                   │ │   │
│  │ │   "debounce": {                                     │ │   │
│  │ │     "usage_state": "Used",                          │ │   │
│  │ │     "can_mangle": true,                             │ │   │
│  │ │     "can_inline": false,                            │ │   │
│  │ │     "annotation": "KEEP"                            │ │   │
│  │ │   },                                                │ │   │
│  │ │   "map": {                                          │ │   │
│  │ │     "usage_state": "ImportedButUnused",             │ │   │
│  │ │     "can_mangle": true,                             │ │   │
│  │ │     "annotation": "ELIMINATE"                       │ │   │
│  │ │   }                                                 │ │   │
│  │ │ }                                                   │ │   │
│  │ └─────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## ConsumeShared Pure Annotation System

### Pure Annotation Logic Enhancement
Based on the updated runtime template code, Rspack now includes sophisticated pure annotation logic:

```rust
// Enhanced Pure Annotation Detection
let is_pure = compilation
  .get_module_graph()
  .dependency_by_id(id)
  .is_some_and(|dep| {
    let dep_type = dep.dependency_type();
    
    // Only apply to named/default imports (not side-effect imports)
    let is_named_import = matches!(dep_type.as_str(), "esm import specifier") 
      && import_var != "__webpack_require__";
    
    if is_named_import {
      // Check if current module or ancestors are ConsumeShared
      let module_graph = compilation.get_module_graph();
      is_consume_shared_descendant(&module_graph, &module.identifier())
    } else {
      false
    }
  });

// Recursive ConsumeShared Detection
fn is_consume_shared_descendant(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
) -> bool {
  let mut visited = std::collections::HashSet::new();
  is_consume_shared_descendant_recursive(module_graph, module_identifier, &mut visited, 10)
}

fn is_consume_shared_descendant_recursive(
  module_graph: &ModuleGraph,
  current_module: &ModuleIdentifier,
  visited: &mut std::collections::HashSet<ModuleIdentifier>,
  max_depth: usize,
) -> bool {
  if max_depth == 0 || visited.contains(current_module) {
    return false;
  }
  visited.insert(current_module.clone());

  // Check if current module is ConsumeShared
  if let Some(module) = module_graph.module_by_identifier(current_module) {
    if module.module_type() == &ModuleType::ConsumeShared {
      return true;
    }
  }

  // Check incoming connections for ConsumeShared ancestors
  for connection in module_graph.get_incoming_connections(current_module) {
    if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
      if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
        if origin_module.module_type() == &ModuleType::ConsumeShared {
          return true;
        }
        
        if is_consume_shared_descendant_recursive(
          module_graph,
          origin_module_id,
          visited,
          max_depth - 1,
        ) {
          return true;
        }
      }
    }
  }

  false
}
```

### Generated Code with Pure Annotations
```javascript
// Without Pure Annotation (side-effect imports)
import './styles.css'; // Side effect - no pure annotation
var __webpack_require__("./styles.css");

// With Pure Annotation (ConsumeShared descendants)
import { debounce } from 'lodash'; // ConsumeShared descendant
/* ESM import */var lodash = /* #__PURE__ */ __webpack_require__("lodash");
/* ESM import */var lodash_default = /*#__PURE__*/__webpack_require__.n(lodash);
```

## Tree-Shaking Annotation Requirements

### Annotation Generation Process
```rust
// Enhanced Share Usage Data Structure
pub struct ShareUsageData {
  pub used_exports: Vec<String>,           // Actually used exports
  pub unused_imports: Vec<String>,         // Imported but unused
  pub provided_exports: Vec<String>,       // Available exports
  pub export_details: Vec<ExportUsageDetail>, // Detailed analysis
  pub has_unused_imports: bool,            // Quick check flag
  pub fallback_info: Option<ModuleExportUsage>, // Fallback module data
}

pub struct ExportUsageDetail {
  pub export_name: String,      // Export identifier
  pub usage_state: String,      // Used/Unused/OnlyPropertiesUsed
  pub can_mangle: Option<bool>, // Name mangling capability
  pub can_inline: Option<bool>, // Inlining optimization potential
  pub is_provided: Option<bool>,// Export provision status
  pub used_name: Option<String>,// Mangled name if applicable
}
```

### Tree-Shaking Decision Flow
```
┌─────────────────────────────────────────────────────────────────┐
│                TREE-SHAKING DECISION PROCESS                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. Export Discovery                                           │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ • Scan fallback module exports                      │   │
│     │ • Extract provided export names                     │   │
│     │ • Determine export capabilities (mangle/inline)     │   │
│     └─────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  2. Usage Analysis                                             │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ • Trace incoming dependency connections             │   │
│     │ • Extract referenced exports from dependencies      │   │
│     │ • Filter by runtime-specific usage                  │   │
│     │ • Distinguish imported vs actually used             │   │
│     └─────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  3. Unused Import Detection                                    │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ • Compare imported_exports vs actually_used         │   │
│     │ • Filter out special exports (*, default)           │   │
│     │ • Verify exports are actually provided              │   │
│     │ • Generate elimination candidates                   │   │
│     └─────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  4. Annotation Generation                                      │
│     ┌─────────────────────────────────────────────────────┐   │
│     │ • Generate usage state classifications              │   │
│     │ • Create optimization hints (mangle/inline)         │   │
│     │ • Produce tree-shaking directives                   │   │
│     │ • Output analysis reports for tooling              │   │
│     └─────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Performance Optimization Strategies

### Batch Processing and Caching
```rust
// Batch Processing for Performance
fn process_module_batch(
  &self,
  module_graph: &ModuleGraph,
  batch: &[(ModuleIdentifier, String)],
  runtimes: &[RuntimeSpec],
) -> Result<HashMap<String, ShareUsageData>> {
  // Batch prefetch export information for all modules in batch
  let prefetch_results = self.batch_prefetch_exports(module_graph, batch)?;
  
  let mut results = HashMap::new();
  let mut diagnostics = Vec::new();
  
  for (module_id, share_key) in batch {
    match self.analyze_single_consume_shared_module(
      module_graph, 
      module_id, 
      runtimes, 
      &prefetch_results
    ) {
      Ok(analysis_result) => {
        results.insert(share_key.clone(), analysis_result.inner);
        diagnostics.extend(analysis_result.diagnostic);
      }
      Err(e) => {
        diagnostics.push(Diagnostic::warn(
          "Failed to analyze module".to_string(), 
          format!("{}: {}", module_id, e)
        ));
      }
    }
  }
  
  Ok(results)
}
```

### Incremental Analysis
```rust
// Cache-Aware Processing
fn needs_analysis(&self, compilation: &Compilation) -> bool {
  if !self.options.enable_caching {
    return true;
  }
  
  let cache = match self.cache.read() {
    Ok(cache) => cache,
    Err(_) => return true,
  };
  
  let current_module_count = compilation.get_module_graph().modules().len();
  let cached_module_count = cache.module_exports.len();
  
  // Trigger analysis if module count changed significantly
  let change_threshold = (cached_module_count as f64 * 0.1).max(5.0) as usize;
  current_module_count.abs_diff(cached_module_count) > change_threshold
}
```

## Conclusion

The Rspack Module Federation sharing system represents a sophisticated architecture that provides:

1. **Comprehensive Module Sharing**: Seamless sharing between micro-frontends with fallback support
2. **Advanced Tree-Shaking**: Precise export usage tracking with unused import elimination
3. **Performance Optimization**: Batch processing, caching, and incremental analysis
4. **Runtime Flexibility**: Dynamic loader selection based on sharing configuration
5. **Pure Annotation System**: Intelligent pure marking for ConsumeShared descendants
6. **Enterprise Features**: Version compatibility, singleton management, and comprehensive error handling

The system successfully balances functionality, performance, and developer experience while providing the foundation for advanced Module Federation scenarios with optimal bundle size through sophisticated tree-shaking capabilities.