# Rspack Module Federation Tree-Shaking Annotations

## Overview
This document provides comprehensive guidance on how to correctly determine exports and imports for tree-shaking annotations in Rspack's Module Federation sharing system, enabling optimal bundle size reduction through precise unused import elimination.

## Export and Import Determination Strategy

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

#### Fallback Module Analysis
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

#### Export Capability Analysis
```rust
fn analyze_export_capabilities(
  module_graph: &ModuleGraph,
  fallback_module_id: &ModuleIdentifier,
  export_name: &str,
) -> ExportCapabilities {
  let exports_info = module_graph.get_exports_info(fallback_module_id);
  let export_info = exports_info.get_export_info(module_graph, export_name);
  
  ExportCapabilities {
    can_mangle: export_info.as_data(module_graph).can_mangle_provide().unwrap_or(false),
    can_inline: match export_info.as_data(module_graph).inlined() {
      Some(Inlinable::Boolean(_)) | Some(Inlinable::String(_)) => true,
      _ => false,
    },
    is_provided: export_info.as_data(module_graph).provided().unwrap_or(false),
    terminal_binding: export_info.as_data(module_graph).terminal_binding(),
    side_effects: export_info.as_data(module_graph).has_side_effects(),
  }
}
```

### Phase 2: Import Pattern Analysis

#### Connection-Based Import Detection
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

#### Usage vs Import Distinction
```rust
fn is_export_actually_used(
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
  export_name: &str,
  runtimes: &[RuntimeSpec],
) -> bool {
  let exports_info = module_graph.get_exports_info(consume_shared_module);
  let export_info = exports_info.get_export_info(module_graph, export_name);
  
  // Check usage state for each runtime
  for runtime in runtimes {
    let usage_state = export_info.get_used_name(Some(runtime));
    match usage_state {
      Some(UsedName::Normal(_)) => return true,
      Some(UsedName::Inlined(_)) => return true,
      None => continue,
    }
  }
  
  // Also check usage state enum
  match export_info.as_data(module_graph).usage_state() {
    UsageState::Used => true,
    UsageState::OnlyPropertiesUsed => true,
    UsageState::Unused => false,
    UsageState::NoInfo | UsageState::Unknown => {
      // Conservative approach - assume used if unknown
      true
    }
  }
}
```

### Phase 3: Unused Import Detection

#### Comprehensive Unused Detection Algorithm
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

#### Advanced Unused Detection with Context
```rust
fn detect_unused_imports_with_context(
  module_graph: &ModuleGraph,
  consume_shared_module: &ModuleIdentifier,
  imported_exports: &[String],
  actually_used_exports: &[String],
  provided_exports: &[String],
) -> Vec<UnusedImportDetail> {
  let mut unused_details = Vec::new();
  
  for export_name in imported_exports {
    if is_special_export(export_name) {
      continue;
    }
    
    if !actually_used_exports.contains(export_name) {
      let export_info = get_export_info(module_graph, consume_shared_module, export_name);
      
      let unused_detail = UnusedImportDetail {
        export_name: export_name.clone(),
        reason: determine_unused_reason(&export_info),
        elimination_safety: assess_elimination_safety(&export_info),
        side_effects: export_info.has_side_effects,
        can_eliminate: can_safely_eliminate(&export_info),
      };
      
      unused_details.push(unused_detail);
    }
  }
  
  unused_details
}

#[derive(Debug, Clone)]
pub struct UnusedImportDetail {
  pub export_name: String,
  pub reason: UnusedReason,
  pub elimination_safety: EliminationSafety,
  pub side_effects: bool,
  pub can_eliminate: bool,
}

#[derive(Debug, Clone)]
pub enum UnusedReason {
  ImportedButNeverReferenced,
  OnlyTypeUsage,
  ConditionalUsage,
  DeadCode,
}

#[derive(Debug, Clone)]
pub enum EliminationSafety {
  Safe,
  SafeWithWarning,
  Unsafe(String),
}
```

### Phase 4: Tree-Shaking Annotation Generation

#### Annotation Generation Algorithm
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

## Practical Implementation for Basic Example

### Basic Example Configuration
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

### Basic Example Analysis Flow
```typescript
// src/app.ts
import { debounce, map, filter } from 'lodash';
import { Component } from 'react';

// Only debounce is actually used
const debouncedFunction = debounce(myFunction, 300);

// map and filter are imported but never used
```

#### Generated Analysis Output
```json
{
  "consume_shared_modules": {
    "lodash": {
      "used_exports": ["debounce"],
      "unused_imports": ["map", "filter"],
      "provided_exports": [
        "debounce", "map", "filter", "throttle", "chunk", "compact",
        "concat", "difference", "drop", "dropRight", "dropRightWhile",
        "dropWhile", "fill", "findIndex", "findLastIndex", "first",
        // ... 200+ more exports
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

## Advanced Annotation Features

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

## Performance Optimization for Tree-Shaking

### Batch Processing Optimization
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

## Conclusion

The Rspack Module Federation tree-shaking annotation system provides comprehensive export and import determination through:

1. **Sophisticated Export Discovery**: Multi-phase analysis from fallback modules with capability detection
2. **Precise Import Tracking**: Connection-based analysis distinguishing imported vs used exports
3. **Advanced Unused Detection**: Context-aware elimination with safety assessment
4. **Flexible Annotation Generation**: Configurable strategies for different optimization goals
5. **Performance Optimization**: Batch processing, caching, and parallel analysis
6. **Runtime Awareness**: Support for conditional tree-shaking across multiple runtime environments

This system enables optimal bundle size reduction in Module Federation scenarios while maintaining correctness and providing detailed analysis for debugging and optimization purposes.