# Module Federation Export Patterns and Proxy Module Implementation

## Overview

This document captures advanced patterns discovered during implementation of ConsumeShared module export copying, extending the research documentation with real-world module federation insights.

## Current Implementation Status

**✅ IMPLEMENTED**: ConsumeShared export metadata copying is already implemented and working correctly in rspack.

### Implementation Location
- **Primary Implementation**: `ConsumeSharedModule` and `ConsumeSharedPlugin`
- **Hook Integration**: Uses `CompilationFinishModules` hook for proper timing
- **API Integration**: Leverages existing export analysis infrastructure

### Key Implementation Details
- **Metadata Copying Methods**: `copy_metadata_from_fallback()` and `copy_exports_from_fallback()` are implemented
- **Lifecycle Timing**: `CompilationFinishModules` executes after build phase, before optimization
- **Plugin Integration**: `ConsumeSharedPlugin` already has `finish_modules` hook implementation

## Investigation Findings

### Key Discoveries from Research

1. **Plugin Integration Status**: `ConsumeSharedPlugin` already has `finish_modules` hook implementation with proper metadata copying
2. **API Usage Patterns**: Investigation revealed correct usage of `ExportsInfoGetter`, `PrefetchExportsInfoMode`, and `ExportInfoGetter`
3. **Export Analysis Results**: `ShareUsagePlugin` investigation showed ConsumeShared modules display empty usage data due to proxy pattern design
4. **Expected Behavior**: ConsumeShared modules showing empty export usage arrays is correct behavior, not a bug

### Technical API Usage Insights

#### Correct API Usage Patterns
```rust
// ExportsInfoGetter vs ExportInfoGetter usage
let exports_info_getter = ExportsInfoGetter::prefetch(
    &fallback_exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports, // Efficient bulk operations
);

// Individual export analysis
let export_info_getter = fallback_exports_info.get_export_info(module_graph, &export_name);
let usage_state = export_info_getter.get_used(); // Usage state analysis
```

#### ProvidedExports Handling
```rust
match provided_exports {
    ProvidedExports::ProvidedNames(names) => {
        // Handle specific named exports
        for name in names {
            // Copy metadata for each named export
        }
    }
    ProvidedExports::ProvidedAll => {
        // Module provides all possible exports dynamically
        consume_exports_info.set_has_provide_info(module_graph);
    }
    ProvidedExports::Unknown => {
        // Preserve unknown status - cannot determine exports statically
    }
}
```

### Usage Analysis Findings

#### ConsumeShared Module Behavior
- **Empty Usage Arrays**: Expected behavior due to proxy pattern
- **Real Usage Data**: Requires analyzing incoming dependencies or fallback module usage
- **Text Analysis vs Runtime**: Different data sources may show different results

#### Data Source Considerations
- **Runtime Analysis**: Shows actual usage patterns during execution
- **Text Analysis Scripts**: May use different data sources than runtime analysis
- **Dependency Analysis**: Real usage data comes from analyzing modules that depend on ConsumeShared modules

## ConsumeShared Proxy Module Pattern

### Architecture

ConsumeShared modules implement a **transparent proxy pattern** where they must perfectly mimic their fallback module's export behavior for accurate tree-shaking:

```rust
// ConsumeShared Module (Proxy)          Fallback Module (Real Implementation)
//       ↓                                        ↓
//   Export Info ←------ COPY METADATA ----→ Export Info
//   Build Meta  ←------ COPY METADATA ----→ Build Meta
//   Provided    ←------ COPY METADATA ----→ Provided
```

### Two-Phase Metadata Copying

#### Phase 1: Direct Metadata Copy
```rust
// Copy build metadata for compilation consistency
consume_shared.build_meta = fallback_module.build_meta().clone();
consume_shared.build_info = fallback_module.build_info().clone();
```

#### Phase 2: Export Information Copy
```rust
// Use prefetched export analysis for efficient copying
let prefetched_fallback = ExportsInfoGetter::prefetch(
    &fallback_exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
);

match prefetched_fallback.get_provided_exports() {
    ProvidedExports::ProvidedNames(export_names) => {
        // Copy each specific export with full metadata
        for export_name in export_names {
            // Copy provided status, can_mangle_provide, nested exports_info
        }
    }
    ProvidedExports::ProvidedAll => {
        // Mark ConsumeShared as providing all exports
    }
    ProvidedExports::Unknown => {
        // Preserve unknown status
    }
}
```

## Plugin Hook Integration Patterns

### CompilationFinishModules Hook Usage

**Timing**: After all modules are built and analyzed, before optimization phases

```rust
#[plugin_hook(CompilationFinishModules for ConsumeSharedPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    // 1. Collect all ConsumeShared modules
    let consume_shared_modules: Vec<ModuleIdentifier> = /* ... */;
    
    // 2. Process each individually to avoid borrow checker issues
    for consume_shared_id in consume_shared_modules {
        Self::copy_fallback_metadata_to_consume_shared(compilation, &consume_shared_id)?;
    }
}
```

### Borrow Checker Patterns

**Problem**: Multiple mutable borrows of ModuleGraph
**Solution**: Separate scopes and helper methods

```rust
// ❌ Problematic - multiple mutable borrows
let mut module_graph = compilation.get_module_graph_mut();
let module = module_graph.module_by_identifier_mut(&id);
// Still borrowing module_graph mutably

// ✅ Correct - separate scopes
{
    let module_graph = compilation.get_module_graph(); // immutable
    let fallback_id = find_fallback(&module_graph);
}
{
    let mut module_graph = compilation.get_module_graph_mut(); // mutable
    copy_exports(&mut module_graph, &fallback_id, &consume_id);
}
```

## Advanced Export Copying Techniques

### Handling Complex Export Types

#### Named Exports with Metadata
```rust
for export_name in export_names {
    let consume_export_info = consume_exports_info.get_export_info(module_graph, &export_name);
    let fallback_export_info = fallback_exports_info.get_export_info(module_graph, &export_name);
    
    // Copy all relevant metadata
    if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
        consume_export_info.as_data_mut(module_graph).set_provided(Some(provided));
    }
    
    // Copy mangling capabilities
    if let Some(can_mangle) = fallback_export_info.as_data(module_graph).can_mangle_provide() {
        consume_export_info.as_data_mut(module_graph).set_can_mangle_provide(Some(can_mangle));
    }
    
    // Copy nested export structures
    if let Some(nested_exports_info) = fallback_export_info.as_data(module_graph).exports_info() {
        consume_export_info.as_data_mut(module_graph).set_exports_info(Some(nested_exports_info));
    }
}
```

#### Setting Complete Provide Info
```rust
// Mark as having complete export information
consume_shared_exports_info.set_has_provide_info(module_graph);

// Set "other exports" to not provided for specific export lists
consume_shared_exports_info.set_unknown_exports_provided(
    module_graph,
    false, // not provided
    None,  // no exclude exports
    None,  // no can_mangle
    None,  // no terminal_binding
    None,  // no target_key
);
```

## Module Federation Specific Considerations

### Fallback Module Detection
```rust
pub fn find_fallback_module_id(&self, module_graph: &ModuleGraph) -> Option<ModuleIdentifier> {
    for dep_id in self.get_dependencies() {
        if let Some(dep) = module_graph.dependency_by_id(dep_id) {
            if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
                if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
                    return Some(*fallback_id);
                }
            }
        }
    }
    None
}
```

### Context Handling for Direct vs Indirect Fallbacks
```rust
let direct_fallback = matches!(&config.import, Some(i) if RELATIVE_REQUEST.is_match(i) | ABSOLUTE_REQUEST.is_match(i));

let context = if direct_fallback {
    self.get_context()  // Use plugin context for direct paths
} else {
    context.clone()     // Use request context for module resolution
};
```

## Performance Considerations

### Efficient Export Analysis
- **Use Prefetched Mode**: `PrefetchExportsInfoMode::AllExports` for bulk analysis
- **Batch Processing**: Process all ConsumeShared modules in one hook invocation
- **Scope Separation**: Avoid holding multiple mutable references

### Error Handling
```rust
if let Err(e) = consume_shared_module.copy_metadata_from_fallback(&mut module_graph) {
    compilation.push_diagnostic(
        rspack_error::Diagnostic::warn(
            "ConsumeSharedPlugin".into(),
            format!("Failed to copy metadata from fallback module: {}", e),
        )
    );
}
```

## Integration with Tree-Shaking System

### Export Tracking Flow
1. **FlagDependencyExportsPlugin**: Analyzes fallback module exports
2. **ConsumeSharedPlugin**: Copies export metadata to proxy module
3. **FlagDependencyUsagePlugin**: Tracks usage through proxy module
4. **Tree-Shaking**: Eliminates unused exports based on copied metadata

### Benefits for Module Federation
- **Accurate Analysis**: Proxy modules reflect real export capabilities
- **Proper Tree-Shaking**: Unused exports in shared modules are eliminated
- **Performance**: Leverages existing export analysis infrastructure
- **Compatibility**: Works with all module formats (ESM, CommonJS)

## ShareUsagePlugin Investigation Findings

Based on the comprehensive ShareUsagePlugin implementation investigation, including the latest enhancement for advanced dependency analysis using `get_referenced_exports()`, the following key findings were discovered about export analysis APIs and ConsumeShared module behavior. For complete plugin development patterns incorporating these learnings, see **[09_plugin_development_patterns.md](./09_plugin_development_patterns.md)**.

### Latest Enhancement: Advanced Dependency Analysis

The ShareUsagePlugin has been enhanced with sophisticated dependency analysis that uses incoming connections to extract specific export usage:

```rust
// Enhanced ConsumeShared analysis using incoming connections
pub fn analyze_consume_shared_usage_from_consumers(
  module_graph: &ModuleGraph,
  consume_shared_id: &ModuleIdentifier,
  _runtimes: &[RuntimeSpec],
) -> ConsumeSharedUsageInfo {
  let mut used_exports = Vec::new();
  let mut uses_namespace = false;
  let mut import_types = std::collections::HashMap::new();

  // Use incoming connections for accurate dependency analysis
  for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
      // Use get_referenced_exports to extract specific export names
      let referenced_exports = dependency.get_referenced_exports(
        module_graph,
        &rspack_core::ModuleGraphCacheArtifact::default(),
        None,
      );
      
      // Process referenced exports to extract used export names
      for export_ref in referenced_exports {
        match export_ref {
          ExtendedReferencedExport::Array(names) => {
            // Multiple specific exports are referenced
            for name in names {
              let export_name = name.to_string();
              if !used_exports.contains(&export_name) {
                used_exports.push(export_name.clone());
                import_types.insert(export_name, "named_import".to_string());
              }
            }
          },
          ExtendedReferencedExport::Export(export_info) => {
            // Single export or namespace reference
            if export_info.name.is_empty() {
              // No specific name indicates namespace usage
              uses_namespace = true;
              import_types.insert("*".to_string(), "namespace_import".to_string());
            } else {
              for name in export_info.name {
                let export_name = name.to_string();
                if !used_exports.contains(&export_name) {
                  used_exports.push(export_name.clone());
                  import_types.insert(export_name, "named_import".to_string());
                }
              }
            }
          },
        }
      }
    }
  }

  ConsumeSharedUsageInfo {
    used_exports: if used_exports.is_empty() { None } else { Some(used_exports) },
    uses_namespace: Some(uses_namespace),
    import_types,
  }
}
```

**Key Enhancement Features:**
1. **Incoming Connection Analysis**: Uses `module_graph.get_incoming_connections()` to find all modules that import from ConsumeShared modules
2. **Referenced Export Extraction**: Calls `dependency.get_referenced_exports()` to extract specific export names being used
3. **Pattern Matching**: Handles both `ExtendedReferencedExport::Array` and `ExtendedReferencedExport::Export` patterns
4. **Cross-referencing**: Compares used exports with provided exports for accurate filtering

### Export Analysis API Patterns

#### Correct API Usage for Export Analysis
```rust
// Use ExportsInfoGetter::prefetch() with appropriate modes
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports, // For comprehensive analysis
);

// Use ExportInfoGetter::get_used() for usage state checking (not ExportsInfoGetter)
let export_info_data = prefetched.get_read_only_export_info(&export_atom);
let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);
```

**Key API Distinctions:**
- `ExportsInfoGetter::prefetch()` - Efficient bulk export analysis
- `ExportInfoGetter::get_used()` - Individual export usage state checking
- `PrefetchExportsInfoMode::AllExports` - Comprehensive analysis mode
- `PrefetchExportsInfoMode::Default` - Lightweight analysis mode

#### ProvidedExports Enum Handling
```rust
match provided_exports {
    ProvidedExports::ProvidedNames(names) => {
        // Handle specific named exports
        for name in names {
            let export_atom = rspack_util::atom::Atom::from(name.as_str());
            // Process each specific export
        }
    },
    ProvidedExports::ProvidedAll => {
        // Module provides all possible exports dynamically
        vec!["*".to_string()]
    },
    ProvidedExports::Unknown => {
        // Cannot determine exports statically - preserve unknown status
        vec![] // Empty vec indicates unknown exports
    }
}
```

### ConsumeShared Module Analysis Patterns

#### Expected Proxy Module Behavior
**Key Finding: ConsumeShared modules showing empty usage arrays is correct behavior, not a bug.**

```rust
// ConsumeShared modules act as proxy modules
// Export usage data is typically empty on proxy modules themselves
// Real usage data requires analyzing:
// 1. Incoming dependencies (modules that depend on ConsumeShared) using get_referenced_exports()
// 2. Fallback modules (the actual implementation)
// 3. Usage through module connections with ExtendedReferencedExport pattern matching

// Enhanced pattern for ConsumeShared analysis with get_referenced_exports():
let consumer_usage = analyze_consume_shared_usage_from_consumers(
    module_graph, 
    consume_shared_id, 
    runtimes
);

// Analysis focuses on:
// - Incoming connections to the ConsumeShared module
// - dependency.get_referenced_exports() for specific export extraction
// - ExtendedReferencedExport::Array and ExtendedReferencedExport::Export pattern handling
// - Cross-referencing used exports with provided exports for accurate filtering
```

#### Usage State Interpretation
- `UsageState::Used` and `UsageState::OnlyPropertiesUsed` indicate actual usage
- Empty usage arrays on ConsumeShared modules are expected due to proxy pattern
- Real usage must be determined through dependency graph traversal

### Plugin Development Insights

#### Proper Plugin Structure for Export Analysis
```rust
#[plugin_hook(CompilerEmit for SharedExportUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    
    // Collect all runtimes for analysis
    let runtimes: Vec<RuntimeSpec> = compilation
        .chunk_by_ukey
        .values()
        .map(|chunk| chunk.runtime())
        .cloned()
        .collect();
    
    // Analyze each module
    for (module_id, _module) in module_graph.modules() {
        let usage_info = analyze_module(
            &module_id,
            &module_graph, 
            &runtimes,
            self.options.detailed_analysis
        )?;
    }
}
```

#### Module Graph Traversal Patterns
```rust
// Proper module graph traversal for export analysis
let exports_info = module_graph.get_exports_info(module_id);
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
);

// Extract provided exports
let provided_exports = prefetched.get_provided_exports();

// Analyze usage for each export
for export_name in provided_exports_vec {
    let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
    let export_info_data = prefetched.get_read_only_export_info(&export_atom);
    let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);
}
```

#### ConsumeShared-Specific Analysis Requirements
```rust
// ConsumeShared modules require special handling
if module.module_type() == &ModuleType::ConsumeShared {
    // 1. Find the fallback module
    let fallback_module_id = find_fallback_module(module_graph, module_id);
    
    // 2. Analyze usage from consumers
    let consumer_usage = analyze_consume_shared_usage_from_consumers(
        module_graph, module_id, runtimes
    );
    
    // 3. Get fallback module exports if available
    let (fallback_exports, _) = if let Some(fallback_id) = fallback_module_id {
        get_fallback_module_exports(module_graph, &fallback_id, runtimes, detailed)
    } else {
        (vec!["*".to_string()], Vec::new())
    };
    
    // 4. Merge analysis results
    let merged_usage = merge_consume_shared_usage_data(
        &consumer_usage, &fallback_exports, &fallback_details
    );
}
```

### Data Source Analysis Findings

#### Text Analysis vs Runtime Analysis
- **Text Analysis Scripts**: May show different results than runtime analysis
- **Runtime Analysis**: Shows actual usage patterns during execution  
- **Dependency Analysis**: Most reliable for ConsumeShared modules
- **Module Graph Analysis**: Required for accurate proxy module understanding

#### Dependency Connection Analysis
```rust
// Use incoming connections for accurate ConsumeShared analysis
for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        let referenced_exports = dependency.get_referenced_exports(
            module_graph,
            &rspack_core::ModuleGraphCacheArtifact::default(),
            None,
        );
        
        // Process referenced exports to extract used export names
        for export_ref in referenced_exports {
            match export_ref {
                ExtendedReferencedExport::Array(names) => {
                    // Multiple specific exports referenced
                },
                ExtendedReferencedExport::Export(export_info) => {
                    // Single export or namespace reference
                },
            }
        }
    }
}
```

## Research Validation and Implementation Status

This documentation now reflects the actual implementation status and investigation findings:

### ✅ Confirmed Implementation
- **ConsumeShared export metadata copying is fully implemented and working**
- ExportsInfo/ExportInfo usage patterns are correctly implemented
- Plugin hook timing and integration points work as documented
- Export specification copying mechanisms function correctly in production

### 🔍 Investigation Insights
- **API Usage Validation**: Correct usage of `ExportsInfoGetter` vs `ExportInfoGetter` confirmed
- **Prefetch Mode Usage**: `PrefetchExportsInfoMode::AllExports` used correctly for bulk operations
- **ProvidedExports Handling**: All three states (ProvidedNames, ProvidedAll, Unknown) handled properly
- **Usage State Analysis**: `ExportInfoGetter::get_used()` provides accurate usage information

### 📝 New Behavioral Understanding
- **ConsumeShared Empty Usage**: Confirmed as expected behavior, not a bug
- **Proxy Pattern Design**: ConsumeShared modules intentionally show empty usage due to proxy architecture
- **Data Source Differences**: Text analysis vs runtime analysis may show different results due to different data sources
- **Real Usage Analysis**: Actual usage must be determined through dependency analysis or fallback module inspection

### 🔄 Implementation Completeness
- **Core Functionality**: All essential features are implemented
- **Error Handling**: Proper error handling and diagnostic reporting in place
- **Performance Optimization**: Efficient bulk operations using prefetch modes
- **Integration**: Seamless integration with existing export analysis infrastructure

### Issues and Solutions Documented
1. **Empty Usage Arrays**: Not a bug - expected behavior for ConsumeShared proxy modules
2. **API Usage Patterns**: Proper distinction between ExportsInfoGetter and ExportInfoGetter clarified
3. **Data Source Understanding**: Different analysis methods (text vs runtime) explained
4. **Lifecycle Timing**: CompilationFinishModules hook usage validated as correct

This document now serves as both implementation reference and investigation findings, confirming that the ConsumeShared export metadata copying system is complete and functioning as designed.

## Flagging ConsumeShared Module Usage for Export Tracking

To properly track used exports for ConsumeShared modules, you need to flag their dependency usage similar to normal modules. Here's how to implement this pattern:

### 1. ConsumeShared Dependency Usage Flagging

```rust
// In your ConsumeShared dependency implementation
impl Dependency for ConsumeSharedDependency {
    fn get_referenced_exports(
        &self,
        module_graph: &ModuleGraph,
        _module_graph_cache: &ModuleGraphCacheArtifact,
        runtime: Option<&RuntimeSpec>,
    ) -> Vec<ExtendedReferencedExport> {
        // Get the ConsumeShared module this dependency points to
        if let Some(consume_shared_id) = module_graph.module_identifier_by_dependency_id(&self.id) {
            // Find what exports are actually being imported from ConsumeShared
            let mut referenced_exports = Vec::new();
            
            // Check if specific exports are being imported
            if let Some(imported_names) = &self.imported_names {
                for name in imported_names {
                    referenced_exports.push(ExtendedReferencedExport::Array(vec![name.clone()]));
                }
            } else if self.namespace_import {
                // Namespace import - references entire exports object
                referenced_exports.push(ExtendedReferencedExport::Array(vec![]));
            } else {
                // Default import or specific patterns
                referenced_exports.push(ExtendedReferencedExport::Array(vec!["default".into()]));
            }
            
            referenced_exports
        } else {
            create_no_exports_referenced()
        }
    }
}
```

### 2. Enhanced ConsumeShared Module Usage Tracking

```rust
// Flag ConsumeShared usage by analyzing incoming connections
pub fn flag_consume_shared_usage(
    module_graph: &mut ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
) -> Result<()> {
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);
    
    // Collect usage information from incoming dependencies
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;
    
    // Analyze incoming connections to determine usage
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
        if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
            let referenced_exports = dependency.get_referenced_exports(
                module_graph,
                &rspack_core::ModuleGraphCacheArtifact::default(),
                runtime,
            );
            
            for export_ref in referenced_exports {
                match export_ref {
                    ExtendedReferencedExport::Array(names) => {
                        if names.is_empty() {
                            // Namespace usage
                            uses_namespace = true;
                        } else {
                            // Specific exports
                            for name in names {
                                used_exports.push(name);
                            }
                        }
                    },
                    ExtendedReferencedExport::Export(export_info) => {
                        if export_info.name.is_empty() {
                            uses_namespace = true;
                        } else {
                            used_exports.extend(export_info.name);
                        }
                    },
                }
            }
        }
    }
    
    // Apply usage flags to ConsumeShared module's exports
    if uses_namespace {
        // Mark all exports as used in unknown way (namespace usage)
        consume_shared_exports_info.set_used_in_unknown_way(module_graph, runtime);
    } else {
        // Mark specific exports as used
        for export_name in used_exports {
            let export_info = consume_shared_exports_info.get_export_info(module_graph, &export_name);
            ExportInfoSetter::set_used(
                export_info.as_data_mut(module_graph),
                UsageState::Used,
                runtime,
            );
        }
    }
    
    Ok(())
}
```

### 3. Integration with FlagDependencyUsagePlugin

```rust
// In FlagDependencyUsagePlugin::process_referenced_module
fn process_consume_shared_module(
    &mut self,
    module_id: ModuleIdentifier,
    referenced_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    queue: &mut Queue<(ModuleIdentifier, Option<RuntimeSpec>)>,
) {
    let module_graph = &mut self.compilation.module_graph;
    let consume_shared_exports_info = module_graph.get_exports_info(&module_id);
    
    // Process referenced exports same as normal modules
    for export_ref in referenced_exports {
        match export_ref {
            ExtendedReferencedExport::Array(export_path) => {
                if export_path.is_empty() {
                    // Namespace usage
                    let changed = consume_shared_exports_info.set_used_in_unknown_way(
                        module_graph, 
                        runtime.as_ref()
                    );
                    if changed {
                        queue.enqueue((module_id, runtime.clone()));
                    }
                } else {
                    // Specific export usage
                    let mut current_exports_info = consume_shared_exports_info;
                    for (i, export_name) in export_path.iter().enumerate() {
                        let export_info = current_exports_info.get_export_info(module_graph, export_name);
                        
                        let usage_state = if i == export_path.len() - 1 {
                            UsageState::Used
                        } else {
                            UsageState::OnlyPropertiesUsed
                        };
                        
                        let changed = ExportInfoSetter::set_used_conditionally(
                            export_info.as_data_mut(module_graph),
                            Box::new(|current| current != &usage_state),
                            usage_state,
                            runtime.as_ref(),
                        );
                        
                        if changed {
                            queue.enqueue((module_id, runtime.clone()));
                        }
                        
                        // Continue to nested exports if not the last one
                        if i < export_path.len() - 1 {
                            if let Some(nested_info) = export_info.as_data(module_graph).exports_info() {
                                current_exports_info = nested_info;
                            }
                        }
                    }
                }
            },
            ExtendedReferencedExport::Export(export_info) => {
                // Handle with mangling and inlining constraints
                let export_path = export_info.name;
                if export_path.is_empty() {
                    let changed = consume_shared_exports_info.set_used_in_unknown_way(
                        module_graph, 
                        runtime.as_ref()
                    );
                    if changed {
                        queue.enqueue((module_id, runtime.clone()));
                    }
                } else {
                    // Process with constraints
                    for export_name in export_path {
                        let export_info_obj = consume_shared_exports_info.get_export_info(module_graph, &export_name);
                        let export_data = export_info_obj.as_data_mut(module_graph);
                        
                        // Apply constraints
                        if !export_info.can_mangle {
                            export_data.set_can_mangle_use(Some(false));
                        }
                        if !export_info.can_inline {
                            export_data.set_inlinable(Inlinable::NoByUse);
                        }
                        
                        let changed = ExportInfoSetter::set_used(
                            export_data,
                            UsageState::Used,
                            runtime.as_ref(),
                        );
                        
                        if changed {
                            queue.enqueue((module_id, runtime.clone()));
                        }
                    }
                }
            },
        }
    }
}
```

### 4. Complete ConsumeShared Usage Workflow

```rust
// Complete workflow for ConsumeShared usage tracking
pub fn track_consume_shared_usage(
    compilation: &mut Compilation,
    consume_shared_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
) -> Result<()> {
    let module_graph = compilation.get_module_graph_mut();
    
    // Step 1: Copy provided exports from fallback module
    if let Some(fallback_id) = find_fallback_module(module_graph, consume_shared_id) {
        copy_exports_from_fallback(module_graph, consume_shared_id, &fallback_id)?;
    }
    
    // Step 2: Flag usage based on incoming dependencies
    flag_consume_shared_usage(module_graph, consume_shared_id, runtime)?;
    
    // Step 3: Process through normal usage plugin flow
    // This happens automatically when FlagDependencyUsagePlugin processes the module
    
    Ok(())
}

// Helper: Copy exports from fallback to ConsumeShared
fn copy_exports_from_fallback(
    module_graph: &mut ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    fallback_id: &ModuleIdentifier,
) -> Result<()> {
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);
    
    let prefetched_fallback = ExportsInfoGetter::prefetch(
        &fallback_exports_info,
        module_graph,
        PrefetchExportsInfoMode::AllExports,
    );
    
    match prefetched_fallback.get_provided_exports() {
        ProvidedExports::ProvidedNames(export_names) => {
            for export_name in export_names {
                let consume_export_info = consume_shared_exports_info.get_export_info(module_graph, &export_name);
                let fallback_export_info = fallback_exports_info.get_export_info(module_graph, &export_name);
                
                // Copy provision status
                if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
                    consume_export_info.as_data_mut(module_graph).set_provided(Some(provided));
                }
                
                // Copy other metadata
                if let Some(can_mangle) = fallback_export_info.as_data(module_graph).can_mangle_provide() {
                    consume_export_info.as_data_mut(module_graph).set_can_mangle_provide(Some(can_mangle));
                }
            }
            
            // Mark as having complete provide info
            consume_shared_exports_info.set_has_provide_info(module_graph);
        },
        ProvidedExports::ProvidedAll => {
            consume_shared_exports_info.set_unknown_exports_provided(
                module_graph, true, None, None, None, None
            );
        },
        ProvidedExports::Unknown => {
            // Keep unknown status
        }
    }
    
    Ok(())
}
```

### Key Benefits of This Approach

1. **Normal Usage Tracking**: ConsumeShared modules get flagged for usage the same way as regular modules
2. **Accurate Export Data**: Provided exports come from fallback module, usage data from actual consumption
3. **Tree-Shaking Ready**: Unused exports in ConsumeShared modules can be properly eliminated
4. **Runtime Awareness**: Supports runtime-specific usage tracking for code splitting
5. **Constraint Handling**: Respects mangling and inlining constraints from dependencies

### Integration Timing

1. **Export Provision**: Copy exports from fallback module during `CompilationFinishModules`
2. **Usage Analysis**: Flag usage during `FlagDependencyUsagePlugin` execution
3. **Tree-Shaking**: Apply optimizations during code generation based on usage flags

## ✅ IMPLEMENTATION STATUS: COMPLETE

**Enhancement Applied**: The `FlagDependencyUsagePlugin` has been updated with special ConsumeShared module handling.

### What Was Implemented

1. **Enhanced FlagDependencyUsagePlugin**: Added `process_consume_shared_module()` method that processes ConsumeShared modules the same way as normal modules for usage tracking.

2. **Special Module Type Detection**: The plugin now detects ConsumeShared modules and routes them to enhanced processing:
   ```rust
   if module.module_type() == &rspack_core::ModuleType::ConsumeShared {
     self.process_consume_shared_module(module_id, used_exports, runtime, force_side_effects, queue);
     return;
   }
   ```

3. **Complete Usage State Management**: ConsumeShared modules now get proper usage state assignment:
   - **Specific Exports**: Individual exports marked as `Used` or `OnlyPropertiesUsed`
   - **Namespace Usage**: All exports marked as used in unknown way
   - **Side Effects**: Proper side-effect-only usage tracking
   - **Constraints**: Mangling and inlining constraints are applied

4. **Tree-Shaking Ready**: ConsumeShared modules can now have unused exports properly eliminated during tree-shaking.

### Integration with Existing System

- **Export Provision**: ConsumeShared modules get provided exports from fallback module (via `ConsumeSharedPlugin::finish_modules`)
- **Usage Tracking**: ConsumeShared modules now get proper usage flags (via enhanced `FlagDependencyUsagePlugin`)
- **Tree-Shaking**: Unused exports in ConsumeShared modules are eliminated during optimization

This implementation ensures that ConsumeShared modules participate fully in the export usage tracking system, enabling proper tree-shaking while maintaining the proxy pattern needed for module federation.

## Fallback Module Tree-Shaking Behavior

### ✅ **Correct Behavior: Fallback Modules Are NOT Tree-Shaken**

The implementation correctly preserves all exports in fallback modules, which is the desired behavior for module federation:

#### Why Fallback Modules Should Not Be Tree-Shaken

1. **Fallback Completeness**: Fallback modules must remain complete because they serve as the backup when shared modules are unavailable
2. **Runtime Uncertainty**: At build time, we don't know which shared module version will be available at runtime
3. **Safety First**: The fallback must be able to provide any export that might be needed

#### How The Implementation Ensures This

1. **ConsumeSharedFallbackDependency**: Does NOT implement `get_referenced_exports()`, so it uses the default that references the entire exports object:
   ```rust
   // Default implementation returns:
   create_exports_object_referenced() // References all exports
   ```

2. **Complete Export Preservation**: The fallback dependency includes all exports, preventing tree-shaking:
   ```rust
   // ConsumeSharedFallbackDependency inherits default behavior
   impl Dependency for ConsumeSharedFallbackDependency {
     // No custom get_referenced_exports - uses default (all exports)
   }
   ```

3. **Proxy Pattern Separation**: Tree-shaking works on the ConsumeShared proxy module, not the fallback:
   - **ConsumeShared Module**: Gets tree-shaken based on actual usage
   - **Fallback Module**: Remains complete and untouched by tree-shaking

#### Tree-Shaking Flow for Module Federation

```
Consumer Code
     ↓ (imports specific exports)
ConsumeShared Module (Proxy)
     ↓ (tree-shaken based on usage)
     ↓ (references ALL exports from fallback)
Fallback Module
     ↓ (NOT tree-shaken - remains complete)
```

### Benefits of This Approach

1. **Safety**: Fallback modules provide complete functionality when shared modules fail
2. **Reliability**: No risk of missing exports in fallback scenarios  
3. **Performance**: Tree-shaking still works on the ConsumeShared proxy layer
4. **Flexibility**: Fallback modules can handle any usage pattern at runtime

### Implementation Summary

The current implementation strikes the perfect balance:
- **ConsumeShared modules**: Participate in tree-shaking for optimal bundle size
- **Fallback modules**: Remain complete for maximum reliability
- **Module federation**: Works correctly with both shared and fallback scenarios

This architectural decision ensures that module federation remains robust while still benefiting from tree-shaking optimizations where appropriate.