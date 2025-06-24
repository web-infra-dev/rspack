# Rspack Module Federation Tree-Shaking System

## Overview

The Rspack Module Federation tree-shaking system is a comprehensive solution that combines PURE annotation generation with sophisticated export/import analysis to enable optimal bundle size reduction. This system specifically targets ConsumeShared modules and their descendants, applying advanced tree-shaking techniques to eliminate unused code in federated module scenarios.

## Core Components

### 1. PURE Annotation System
Applies `/* #__PURE__ */` annotations to import statements for dead code elimination targeting ConsumeShared module descendants.

### 2. Export/Import Analysis Engine
Provides comprehensive guidance on determining exports and imports for tree-shaking annotations through multi-phase analysis.

### 3. ConsumeShared Integration
Seamlessly integrates with Module Federation's sharing system to enable precise unused import elimination.

## System Architecture

### Key Data Structures

```rust
// Module type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
    ConsumeShared,    // Shared modules in Module Federation
    JsAuto,          // Regular JavaScript modules  
    JsEsm,           // ES Module JavaScript
}

// Dependency type classification for imports
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
    EsmImport,         // "esm import" - bare imports like `import './module'`
    EsmImportSpecifier, // "esm import specifier" - named imports like `import { func } from './module'`
}

// Tree-shaking annotation structure
#[derive(Debug, Clone)]
pub struct TreeShakingAnnotation {
    pub export_name: String,
    pub action: TreeShakingAction,
    pub reason: String,
    pub confidence: AnnotationConfidence,
}

#[derive(Debug, Clone)]
pub enum TreeShakingAction {
    Keep,
    Eliminate,
    Transform,
    Warning,
}

#[derive(Debug, Clone)]
pub enum AnnotationConfidence {
    High,    // Very confident - safe to act
    Medium,  // Moderately confident - consider acting
    Low,     // Low confidence - warning only
}
```

## PURE Annotation Implementation

### Core Logic in `runtime_template.rs`

The PURE annotation system is implemented in the `import_statement` function:

```rust
pub fn import_statement(
  module: &dyn Module,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  id: &DependencyId,
  request: &str,
  update: bool,
) -> (String, String) {
  // ... module resolution logic ...

  // Check if this import should be marked as pure
  // Only apply PURE annotations to modules that descend from ConsumeShared modules
  let is_pure = compilation
    .get_module_graph()
    .dependency_by_id(id)
    .is_some_and(|dep| {
      // Check the dependency type to distinguish between different import types
      let dep_type = dep.dependency_type();
      // Include both "esm import" (bare imports) and "esm import specifier" (named imports)
      // but exclude __webpack_require__ calls themselves
      let is_esm_import = matches!(dep_type.as_str(), "esm import" | "esm import specifier") 
        && import_var != "__webpack_require__";
      
      // Only apply PURE annotation if this is an ESM import AND descends from ConsumeShared
      if is_esm_import {
        // Check if the current module or any ancestor is ConsumeShared
        let module_graph = compilation.get_module_graph();
        is_consume_shared_descendant(&module_graph, &module.identifier())
      } else {
        false
      }
    });

  let pure_annotation = if is_pure { "/* #__PURE__ */ " } else { "" };

  let import_content = format!(
    "/* ESM import */{opt_declaration}{import_var} = {}{}({module_id_expr});\n",
    pure_annotation,
    RuntimeGlobals::REQUIRE
  );

  // ... rest of implementation
}
```

### ConsumeShared Ancestry Detection

```rust
/// Check if a module is a descendant of a ConsumeShared module
/// by traversing incoming connections recursively
fn is_consume_shared_descendant(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
) -> bool {
  let mut visited = std::collections::HashSet::new();
  is_consume_shared_descendant_recursive(module_graph, module_identifier, &mut visited, 10)
}

/// Recursively search for ConsumeShared modules in the module graph ancestry
fn is_consume_shared_descendant_recursive(
  module_graph: &ModuleGraph,
  current_module: &ModuleIdentifier,
  visited: &mut std::collections::HashSet<ModuleIdentifier>,
  max_depth: usize,
) -> bool {
  // Prevent infinite loops and excessive depth
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

  // Check all incoming connections for ConsumeShared ancestors
  for connection in module_graph.get_incoming_connections(current_module) {
    if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
      if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
        // Found a ConsumeShared module - this is a descendant
        if origin_module.module_type() == &ModuleType::ConsumeShared {
          return true;
        }
        
        // Recursively check this module's incoming connections
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

## Export/Import Analysis Engine

### Core Determination Algorithm

The tree-shaking annotation system follows a sophisticated multi-phase approach:

```rust
// Core Export/Import Determination Process
pub fn determine_exports_and_imports(
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
  runtimes: &[RuntimeSpec],
) -> Result<ExportImportAnalysis> {
  // Phase 1: Discover provided exports from fallback module
  let provided_exports = discover_fallback_exports(module_graph, consume_shared_module)?;
  
  // Phase 2: Analyze incoming connections for import patterns
  let (imported_exports, used_exports) = analyze_incoming_connections(
    module_graph, 
    consume_shared_module, 
    runtimes
  )?;
  
  // Phase 3: Detect unused imports for elimination
  let unused_imports = detect_unused_imports(
    &imported_exports, 
    &used_exports, 
    &provided_exports
  );
  
  // Phase 4: Generate tree-shaking annotations
  let annotations = generate_tree_shaking_annotations(
    &provided_exports,
    &imported_exports, 
    &used_exports,
    &unused_imports
  );
  
  Ok(ExportImportAnalysis {
    provided_exports,
    imported_exports,
    used_exports,
    unused_imports,
    annotations,
  })
}
```

### Phase 1: Export Discovery from Fallback Modules

```rust
fn discover_fallback_exports(
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
) -> Result<Vec<String>> {
  // Step 1: Find the fallback module
  let fallback_module_id = find_fallback_module_id(module_graph, consume_shared_module)?;
  
  // Step 2: Prefetch export information efficiently
  let fallback_exports_info = module_graph.get_exports_info(&fallback_module_id);
  let prefetched = ExportsInfoGetter::prefetch(
    &fallback_exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
  );
  
  // Step 3: Extract provided exports
  let provided_exports = match prefetched.get_provided_exports() {
    ProvidedExports::ProvidedNames(names) => {
      names.iter().map(|name| name.to_string()).collect()
    }
    ProvidedExports::ProvidedAll => {
      // Handle dynamic exports - return special marker
      vec!["*".to_string()]
    }
    ProvidedExports::Unknown => {
      // Return empty list for unknown exports
      Vec::new()
    }
  };
  
  Ok(provided_exports)
}
```

### Phase 2: Import Pattern Analysis

```rust
fn analyze_incoming_connections(
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
  runtimes: &[RuntimeSpec],
) -> Result<(Vec<String>, Vec<String>)> {
  let mut imported_exports = Vec::new();
  let mut actually_used_exports = Vec::new();
  let mut processed_dependencies = HashSet::new();
  
  // Get all incoming connections to the ConsumeShared module
  let connections: Vec<_> = module_graph
    .get_incoming_connections(consume_shared_module)
    .collect();
  
  for connection in connections {
    // Avoid processing the same dependency multiple times
    if processed_dependencies.contains(&connection.dependency_id) {
      continue;
    }
    processed_dependencies.insert(connection.dependency_id);
    
    // Check if connection is active for current runtime
    let connection_active = match connection.active_state(
      module_graph,
      runtimes.first(),
      &Default::default(),
    ) {
      ConnectionState::Active(active) => active,
      ConnectionState::TransitiveOnly => true,
      ConnectionState::CircularConnection => false,
    };
    
    if !connection_active {
      continue;
    }
    
    // Extract referenced exports from dependency
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
      let referenced_exports = dependency.get_referenced_exports(
        module_graph,
        &ModuleGraphCacheArtifact::default(),
        None,
      );
      
      for export_ref in referenced_exports {
        match export_ref {
          ExtendedReferencedExport::Array(names) => {
            for name in names {
              let export_name = name.to_string();
              
              // Track as imported
              if !imported_exports.contains(&export_name) {
                imported_exports.push(export_name.clone());
              }
              
              // Check if actually used vs just imported
              if is_export_actually_used(
                module_graph,
                consume_shared_module,
                &export_name,
                runtimes,
              ) {
                if !actually_used_exports.contains(&export_name) {
                  actually_used_exports.push(export_name);
                }
              }
            }
          }
          ExtendedReferencedExport::Export(export_info) => {
            // Handle export info structures
            process_export_info_reference(
              &export_info,
              &mut imported_exports,
              &mut actually_used_exports,
              module_graph,
              consume_shared_module,
              runtimes,
            );
          }
        }
      }
    }
  }
  
  Ok((imported_exports, actually_used_exports))
}
```

### Phase 3: Unused Import Detection

```rust
fn detect_unused_imports(
  imported_exports: &[String],
  actually_used_exports: &[String],
  provided_exports: &[String],
) -> Vec<String> {
  let mut unused_imports = Vec::new();
  
  for export_name in imported_exports {
    // Skip special exports that shouldn't be eliminated
    if is_special_export(export_name) {
      continue;
    }
    
    // Check if export is actually provided by the module
    if !provided_exports.contains(export_name) && !provided_exports.contains(&"*".to_string()) {
      // Export not provided - might be a type-only import
      continue;
    }
    
    // Check if imported but not actually used
    if !actually_used_exports.contains(export_name) {
      unused_imports.push(export_name.clone());
    }
  }
  
  unused_imports
}

fn is_special_export(export_name: &str) -> bool {
  matches!(
    export_name,
    "*" | "default" | "__esModule" | "__webpack_exports__"
  )
}
```

### Phase 4: Tree-Shaking Annotation Generation

```rust
fn generate_tree_shaking_annotations(
  provided_exports: &[String],
  imported_exports: &[String],
  used_exports: &[String],
  unused_imports: &[String],
) -> Vec<TreeShakingAnnotation> {
  let mut annotations = Vec::new();
  
  // Process all provided exports
  for export_name in provided_exports {
    let annotation = if used_exports.contains(export_name) {
      // Export is actively used - must keep
      TreeShakingAnnotation {
        export_name: export_name.clone(),
        action: TreeShakingAction::Keep,
        reason: "Used in application code".to_string(),
        confidence: AnnotationConfidence::High,
      }
    } else if imported_exports.contains(export_name) {
      // Export is imported but not used - candidate for elimination
      TreeShakingAnnotation {
        export_name: export_name.clone(),
        action: TreeShakingAction::Eliminate,
        reason: "Imported but never used".to_string(),
        confidence: AnnotationConfidence::High,
      }
    } else {
      // Export is not imported - safe to eliminate
      TreeShakingAnnotation {
        export_name: export_name.clone(),
        action: TreeShakingAction::Eliminate,
        reason: "Not imported".to_string(),
        confidence: AnnotationConfidence::Medium,
      }
    };
    
    annotations.push(annotation);
  }
  
  annotations
}
```

## Import Statement Classification

### ESM Import Types

The system distinguishes between two types of ES module imports:

#### 1. ESM Import ("esm import")
- **Definition**: Bare import statements without specific specifiers
- **Examples**:
  ```javascript
  import './side-effects-module';
  import 'shared-library';
  ```
- **Dependency Type**: `DependencyType::EsmImport`
- **String Representation**: `"esm import"`

#### 2. ESM Import Specifier ("esm import specifier")  
- **Definition**: Named import statements with specific specifiers
- **Examples**:
  ```javascript
  import { func, Component } from 'shared-library';
  import defaultExport from 'shared-library';
  import * as namespace from 'shared-library';
  ```
- **Dependency Type**: `DependencyType::EsmImportSpecifier`
- **String Representation**: `"esm import specifier"`

### Classification Logic

```rust
let dep_type = dep.dependency_type();
let is_esm_import = matches!(dep_type.as_str(), "esm import" | "esm import specifier") 
  && import_var != "__webpack_require__";
```

**Key Points**:
- Both import types are eligible for PURE annotations
- `__webpack_require__` calls are explicitly excluded
- Only ES module imports are considered (CommonJS requires are not)

## Code Generation Examples

### Without PURE Annotation (Regular Module)

```javascript
/* ESM import */var module_a = __webpack_require__(/*! ./moduleA */ "./src/moduleA.js");
```

### With PURE Annotation (ConsumeShared Descendant)

```javascript
/* ESM import */var shared_lib = /* #__PURE__ */ __webpack_require__(/*! shared-lib */ "./node_modules/shared-lib/index.js");
```

### Dynamic Export Handling with PURE

```javascript
/* ESM import */var shared_lib = /* #__PURE__ */ __webpack_require__(/*! shared-lib */ "./node_modules/shared-lib/index.js");
/* ESM import */var shared_lib_default = /*#__PURE__*/__webpack_require__.n(shared_lib);
```

## Practical Implementation Example

### Configuration Setup

```javascript
// rspack.config.js
module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'main-app',
      remotes: {
        'shared-lib': 'sharedLib@http://localhost:3001/remoteEntry.js'
      },
      shared: {
        'lodash': {
          singleton: true,
          requiredVersion: '^4.17.0'
        },
        'react': {
          singleton: true,
          requiredVersion: '^18.0.0'
        }
      }
    }),
    new EnhancedShareUsagePlugin({
      filename: 'share-usage-analysis.json',
      include_export_details: true,
      detect_unused_imports: true,
      enable_caching: true,
      runtime_analysis: true,
    })
  ]
};
```

### Application Code

```typescript
// src/app.ts
import { debounce, map, filter } from 'lodash';
import { Component } from 'react';

// Only debounce is actually used
const debouncedFunction = debounce(myFunction, 300);

// map and filter are imported but never used
```

### Generated Analysis Output

```json
{
  "consume_shared_modules": {
    "lodash": {
      "used_exports": ["debounce"],
      "unused_imports": ["map", "filter"],
      "provided_exports": [
        "debounce", "map", "filter", "throttle", "chunk", "compact",
        "concat", "difference", "drop", "dropRight", "dropRightWhile",
        "dropWhile", "fill", "findIndex", "findLastIndex", "first"
      ],
      "export_details": [
        {
          "export_name": "debounce",
          "usage_state": "Used",
          "can_mangle": true,
          "can_inline": false,
          "is_provided": true,
          "used_name": "debounce",
          "annotation": "KEEP"
        },
        {
          "export_name": "map",
          "usage_state": "ImportedButUnused",
          "can_mangle": true,
          "can_inline": true,
          "is_provided": true,
          "used_name": null,
          "annotation": "ELIMINATE"
        },
        {
          "export_name": "filter",
          "usage_state": "ImportedButUnused",
          "can_mangle": true,
          "can_inline": true,
          "is_provided": true,
          "used_name": null,
          "annotation": "ELIMINATE"
        }
      ],
      "has_unused_imports": true,
      "fallback_info": {
        "module_identifier": "./node_modules/lodash/index.js",
        "provided_exports": ["debounce", "map", "filter", "..."],
        "side_effects": false,
        "optimization_bailout": []
      }
    }
  },
  "analysis_metadata": {
    "timestamp": 1698765432000,
    "module_count": 150,
    "analysis_duration_ms": 45,
    "cache_hit_rate": 0.85
  },
  "diagnostics": [],
  "performance_metrics": {
    "batch_processing_time_ms": 12,
    "prefetch_operations": 25,
    "connection_analysis_time_ms": 18,
    "unused_detection_time_ms": 8
  }
}
```

## Tree-Shaking Integration Strategies

### Strategy 1: Aggressive Elimination
```rust
// Configuration for maximum tree-shaking
pub struct AggressiveTreeShakingConfig {
  eliminate_unused_imports: true,
  eliminate_unused_exports: true,
  eliminate_property_access: true,
  warn_on_dynamic_imports: true,
  confidence_threshold: AnnotationConfidence::Medium,
}
```

### Strategy 2: Conservative Elimination
```rust
// Configuration for safe tree-shaking
pub struct ConservativeTreeShakingConfig {
  eliminate_unused_imports: true,
  eliminate_unused_exports: false,  // Keep exports for potential use
  eliminate_property_access: false,
  warn_on_dynamic_imports: false,
  confidence_threshold: AnnotationConfidence::High,
}
```

### Strategy 3: Development Mode
```rust
// Configuration for development builds
pub struct DevelopmentTreeShakingConfig {
  eliminate_unused_imports: false,  // Keep imports for hot reload
  eliminate_unused_exports: false,
  eliminate_property_access: false,
  warn_on_dynamic_imports: true,
  confidence_threshold: AnnotationConfidence::High,
  generate_warnings_only: true,     // Don't eliminate, just warn
}
```

## Advanced Features

### Runtime-Conditional Tree-Shaking

```rust
fn generate_runtime_conditional_annotations(
  export_name: &str,
  runtimes: &[RuntimeSpec],
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
) -> RuntimeConditionalAnnotation {
  let mut runtime_usage = HashMap::new();
  
  for runtime in runtimes {
    let usage_in_runtime = check_export_usage_in_runtime(
      module_graph,
      consume_shared_module,
      export_name,
      runtime,
    );
    runtime_usage.insert(runtime.clone(), usage_in_runtime);
  }
  
  RuntimeConditionalAnnotation {
    export_name: export_name.to_string(),
    runtime_usage,
    conditional_elimination: determine_conditional_elimination(&runtime_usage),
  }
}
```

### Side Effect Analysis Integration

```rust
fn analyze_side_effects_for_tree_shaking(
  module_graph: &ModuleGraph,
  fallback_module: &ModuleIdentifier,
  export_name: &str,
) -> SideEffectAnalysis {
  let module = module_graph.module_by_identifier(fallback_module).unwrap();
  
  // Check module-level side effects
  let module_side_effects = module
    .get_build_meta()
    .and_then(|meta| meta.side_effect_free)
    .unwrap_or(false);
  
  // Check export-specific side effects
  let export_side_effects = check_export_side_effects(module_graph, fallback_module, export_name);
  
  SideEffectAnalysis {
    module_side_effect_free: module_side_effects,
    export_side_effect_free: export_side_effects,
    safe_to_eliminate: module_side_effects && export_side_effects,
    elimination_risk: assess_elimination_risk(module_side_effects, export_side_effects),
  }
}
```

## Performance Optimization

### Batch Processing

```rust
fn batch_process_tree_shaking_analysis(
  module_graph: &ModuleGraph,
  consume_shared_modules: &[(ModuleIdentifier, String)],
  batch_size: usize,
) -> Result<HashMap<String, TreeShakingAnalysis>> {
  let mut results = HashMap::new();
  
  for batch in consume_shared_modules.chunks(batch_size) {
    // Batch prefetch export information
    let prefetch_cache = batch_prefetch_exports(module_graph, batch)?;
    
    // Process batch in parallel
    let batch_results: Vec<_> = batch
      .par_iter()
      .map(|(module_id, share_key)| {
        let analysis = analyze_tree_shaking_for_module(
          module_graph,
          module_id,
          &prefetch_cache,
        );
        (share_key.clone(), analysis)
      })
      .collect();
    
    // Merge results
    for (share_key, analysis) in batch_results {
      results.insert(share_key, analysis?);
    }
  }
  
  Ok(results)
}
```

### Caching Strategy

```rust
pub struct TreeShakingCache {
  export_analysis_cache: LruCache<ModuleIdentifier, ExportAnalysis>,
  usage_analysis_cache: LruCache<(ModuleIdentifier, RuntimeSpec), UsageAnalysis>,
  annotation_cache: LruCache<AnnotationKey, TreeShakingAnnotation>,
  last_analysis_timestamp: SystemTime,
}

impl TreeShakingCache {
  fn get_or_compute_export_analysis(
    &mut self,
    module_id: &ModuleIdentifier,
    compute_fn: impl FnOnce() -> ExportAnalysis,
  ) -> &ExportAnalysis {
    self.export_analysis_cache.get_or_insert_with(*module_id, compute_fn)
  }
  
  fn invalidate_on_change(&mut self, changed_modules: &[ModuleIdentifier]) {
    for module_id in changed_modules {
      self.export_analysis_cache.remove(module_id);
    }
  }
}
```

## Error Handling and Edge Cases

### Missing Modules

```rust
if compilation
  .get_module_graph()
  .module_identifier_by_dependency_id(id)
  .is_none()
{
  return (missing_module_statement(request), String::new());
};
```

### Circular Dependencies

The visited HashSet prevents infinite loops in circular dependency scenarios:

```rust
if max_depth == 0 || visited.contains(current_module) {
  return false;
}
```

### Invalid Connections

The system gracefully handles missing or invalid module connections:

```rust
if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
  if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
    // Process valid connection
  }
}
```

## Integration with Module Federation

### Module Federation Context

The tree-shaking system is specifically designed for Module Federation scenarios where:

1. **Shared Dependencies**: Multiple micro-frontends share common dependencies
2. **Runtime Resolution**: Dependencies are resolved at runtime from shared scopes
3. **Tree Shaking**: Unused exports need to be eliminated to reduce bundle size
4. **Fallback Handling**: Local fallback modules are used when shared dependencies are unavailable

### Practical Module Federation Example

```javascript
// Module Federation Configuration
const ModuleFederationPlugin = require('@module-federation/webpack');

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        mfe1: 'mfe1@http://localhost:3001/remoteEntry.js'
      },
      shared: {
        'react': { singleton: true },
        'lodash': { singleton: true }
      }
    })
  ]
};

// Application Code
import { debounce } from 'lodash'; // This import gets PURE annotation
import { Component } from 'react'; // This import gets PURE annotation

// Generated Code (with PURE annotations)
/* ESM import */var lodash = /* #__PURE__ */ __webpack_require__(/*! lodash */ "webpack/sharing/consume/default/lodash");
/* ESM import */var react = /* #__PURE__ */ __webpack_require__(/*! react */ "webpack/sharing/consume/default/react");
```

## Debugging and Diagnostics

### Module Identification

ConsumeShared modules can be identified by their module type:

```rust
if module.module_type() == &ModuleType::ConsumeShared {
  // This is a ConsumeShared module
}
```

### Dependency Analysis

Track dependency relationships for debugging:

```rust
let dep_type = dep.dependency_type();
println!("Dependency type: {}", dep_type.as_str());
// Outputs: "esm import" or "esm import specifier"
```

### Connection Traversal

Debug ancestry detection by logging connection traversal:

```rust
for connection in module_graph.get_incoming_connections(current_module) {
  if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
    println!("Checking connection from: {:?}", origin_module_id);
  }
}
```

## Future Enhancements

### Potential Improvements

1. **Enhanced Caching**: Cache ancestry detection results to avoid repeated traversal
2. **Parallel Processing**: Process multiple ancestry checks concurrently
3. **Heuristic Optimization**: Use module naming patterns to optimize detection
4. **Statistical Analysis**: Track and report PURE annotation effectiveness
5. **Configuration Options**: Fine-tuning parameters for different optimization strategies

### Configuration Framework

```rust
pub struct TreeShakingConfig {
  pub max_traversal_depth: usize,
  pub enable_caching: bool,
  pub include_dynamic_imports: bool,
  pub strict_consume_shared_detection: bool,
  pub batch_size: usize,
  pub confidence_threshold: AnnotationConfidence,
}
```

## Conclusion

The Rspack Module Federation tree-shaking system provides comprehensive optimization capabilities through:

1. **Sophisticated Detection**: Accurately identifying ConsumeShared modules and their descendants
2. **Multi-Phase Analysis**: Comprehensive export discovery, import pattern analysis, and unused detection
3. **PURE Annotation Generation**: Standards-compliant annotations for bundler compatibility
4. **Performance Optimization**: Efficient graph traversal, batch processing, and caching
5. **Runtime Awareness**: Support for conditional tree-shaking across multiple runtime environments
6. **Integration Excellence**: Seamless integration with Module Federation sharing system

This system enables optimal bundle size reduction in complex Module Federation setups while maintaining correctness, performance, and providing detailed analysis for debugging and optimization purposes.