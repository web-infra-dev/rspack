# Export Usage Tracking Integration and Workflow

## Overview

This document provides a comprehensive analysis of how rspack's export usage tracking system integrates all components to enable sophisticated tree-shaking and module optimization. The system combines multiple plugins, dependency types, and analysis phases to create a complete picture of export provision and usage across the entire module graph.

## System Architecture Overview

### Component Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│                    Compilation Process                       │
├─────────────────────────────────────────────────────────────┤
│  1. Module Parsing & Dependency Creation                    │
│     ├── ESMExportSpecifierDependency                       │
│     ├── ESMExportImportedSpecifierDependency               │
│     ├── ESMImportSpecifierDependency                       │
│     ├── CommonJsExportsDependency                          │
│     └── ExportInfoDependency                               │
│                                                             │
│  2. Export Provision Analysis (FlagDependencyExportsPlugin) │
│     ├── Collect ExportsSpec from all dependencies          │
│     ├── Populate ExportsInfo with provision data           │
│     ├── Track re-export relationships                      │
│     └── Handle nested and dynamic exports                  │
│                                                             │
│  3. Export Usage Analysis (FlagDependencyUsagePlugin)       │
│     ├── Start from entry points                            │
│     ├── Traverse dependency graph                          │
│     ├── Collect referenced exports                         │
│     └── Mark usage states in ExportsInfo                   │
│                                                             │
│  4. Code Generation & Optimization                          │
│     ├── Query ExportsInfo for used exports                 │
│     ├── Generate optimized export code                     │
│     ├── Apply tree-shaking decisions                       │
│     └── Handle module federation scenarios                 │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Workflow Analysis

### Phase 1: Module Parsing and Dependency Creation

#### 1.1 Dependency Creation During Parsing

When rspack parses a module, it creates specific dependency types based on the export/import patterns found:

**ESM Export Examples**:
```javascript
// Creates ESMExportSpecifierDependency
export const foo = 'value';
export { bar };

// Creates ESMExportImportedSpecifierDependency  
export { baz } from './module';
export * from './module';

// Creates ESMImportSpecifierDependency
import { used } from './module';
```

**CommonJS Export Examples**:
```javascript
// Creates CommonJsExportsDependency
exports.foo = 'value';
module.exports.bar = 'value';
Object.defineProperty(exports, 'baz', { value: 'value' });
```

**Export Info Access**:
```javascript
// Creates ExportInfoDependency
const isUsed = __webpack_exports_info__.used;
const canMangle = __webpack_exports_info__.canMangle;
```

#### 1.2 Dependency Registration

Each dependency implements the `get_exports()` method to describe what it provides:

```rust
// ESMExportSpecifierDependency example
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: self.name.clone(),
            inlinable: self.inline,
            ..Default::default()
        })]),
        priority: Some(1),
        can_mangle: None,
        terminal_binding: Some(true),
        from: None,
        dependencies: None,
        hide_export: None,
        exclude_exports: None,
    })
}
```

### Phase 2: Export Provision Analysis

#### 2.1 FlagDependencyExportsPlugin Execution

**Hook**: `CompilationFinishModules` - runs after all modules are parsed

**Process**:
```rust
// 1. Initialize exports info for all modules
for module_id in modules {
    let exports_info = mgm.exports;
    exports_info.reset_provide_info(self.mg);
    exports_info.set_has_provide_info(self.mg);
}

// 2. Collect export specifications from dependencies
for dep_id in module.get_dependencies() {
    let exports_spec = dep.get_exports(self.mg, module_graph_cache);
    exports_specs_from_dependencies.insert(dep_id, exports_spec);
}

// 3. Process and merge export specifications
for (dep_id, exports_spec) in exports_specs_from_dependencies {
    self.process_exports_spec(dep_id, exports_spec, exports_info);
}
```

#### 2.2 Export Information Population

**ExportsInfo Structure Population**:
```rust
// For each export found in ExportsSpec:
let export_info = exports_info.get_export_info(self.mg, &name);
let export_info_data = export_info.as_data_mut(self.mg);

// Set provision status
export_info_data.set_provided(Some(ExportProvided::Provided));

// Set mangling capabilities
if can_mangle == Some(false) {
    export_info_data.set_can_mangle_provide(Some(false));
}

// Set inlining potential
if let Some(inlined) = inlinable {
    export_info_data.set_inlinable(Inlinable::Inlined(inlined));
}

// Set terminal binding
if terminal_binding {
    export_info_data.set_terminal_binding(true);
}
```

#### 2.3 Re-export Target Tracking

For re-exports, the system establishes target relationships:

```rust
// For: export { foo } from './module'
if let Some(from) = from {
    ExportInfoSetter::set_target(
        export_info_data,
        Some(dep_id),                    // Dependency creating the re-export
        Some(from.dependency_id),        // Connection to target module
        export_name,                     // What export to target
        priority,                        // Priority for conflict resolution
    );
    
    // Track dependency for invalidation
    self.dependencies.entry(target.module)
        .or_insert_with(IdentifierSet::new)
        .insert(self.current_module_id);
}
```

### Phase 3: Export Usage Analysis

#### 3.1 FlagDependencyUsagePlugin Execution

**Hook**: `CompilationOptimizeDependencies` - runs during optimization phase

**Entry Point Analysis**:
```rust
// Start from application entry points
for (entry_name, entry) in entries.iter() {
    let runtime = get_entry_runtime(entry_name, &entry.options, &entries);
    
    for &dep in entry.dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut q);
    }
}

// Process global entries
for dep in self.compilation.global_entry.dependencies.clone() {
    self.process_entry_dependency(dep, global_runtime.clone(), &mut q);
}
```

#### 3.2 Dependency Graph Traversal

**Breadth-First Processing**:
```rust
while let Some((module_id, runtime)) = q.dequeue() {
    // Process the module and all its dependencies
    self.process_module(ModuleOrAsyncDependenciesBlock::Module(module_id), runtime, false, &mut q);
}

fn process_module(&mut self, block_id: ModuleOrAsyncDependenciesBlock, runtime: Option<RuntimeSpec>, force_side_effects: bool, q: &mut Queue<(ModuleIdentifier, Option<RuntimeSpec>)>) {
    // Collect all referenced exports from module dependencies
    let mut referenced_exports_map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();
    
    for dep_id in dependencies {
        let connection = module_graph.connection_by_dependency_id(&dep_id);
        let active_state = connection.active_state(&module_graph, runtime.as_ref(), module_graph_cache);
        
        // Get what exports this dependency references
        let referenced_exports = if let Some(md) = dep.as_module_dependency() {
            md.get_referenced_exports(&module_graph, module_graph_cache, runtime.as_ref())
        } else {
            continue;
        };
        
        // Merge with existing references
        self.merge_referenced_exports(connection.module_identifier(), referenced_exports, &mut referenced_exports_map);
    }
    
    // Apply usage information to all referenced modules
    for (module_id, referenced_exports) in referenced_exports_map {
        self.process_referenced_module(module_id, referenced_exports, runtime.clone(), force_side_effects, q);
    }
}
```

#### 3.3 Usage State Application

**Referenced Export Processing**:
```rust
fn process_referenced_module(&mut self, module_id: ModuleIdentifier, used_exports: Vec<ExtendedReferencedExport>, runtime: Option<RuntimeSpec>, force_side_effects: bool, queue: &mut Queue<(ModuleIdentifier, Option<RuntimeSpec>)>) {
    for used_export_info in used_exports {
        let (used_exports, can_mangle, can_inline) = extract_usage_info(used_export_info);
        
        if used_exports.is_empty() {
            // Unknown usage pattern
            mgm_exports_info.set_used_in_unknown_way(&mut module_graph, runtime.as_ref());
        } else {
            // Specific export usage
            let mut current_exports_info = mgm_exports_info;
            for (i, used_export) in used_exports.into_iter().enumerate() {
                let export_info = current_exports_info.get_export_info(&mut module_graph, &used_export);
                
                // Apply usage constraints
                if !can_mangle {
                    export_info.as_data_mut(&mut module_graph).set_can_mangle_use(Some(false));
                }
                if !can_inline {
                    export_info.as_data_mut(&mut module_graph).set_inlinable(Inlinable::NoByUse);
                }
                
                // Set usage state
                let usage_state = if i == used_exports.len() - 1 {
                    UsageState::Used
                } else {
                    UsageState::OnlyPropertiesUsed
                };
                
                ExportInfoSetter::set_used_conditionally(
                    export_info.as_data_mut(&mut module_graph),
                    Box::new(|used| used != &usage_state),
                    usage_state,
                    runtime.as_ref(),
                );
            }
        }
    }
}
```

### Phase 4: Code Generation and Optimization

#### 4.1 Export Code Generation

During code generation, dependencies query the populated ExportsInfo to make optimization decisions:

**ESM Export Generation**:
```rust
// ESMExportSpecifierDependencyTemplate
fn render(&self, dep: &dyn DependencyCodeGeneration, context: &mut TemplateContext) {
    // Query usage information
    let exports_info = module_graph.get_prefetched_exports_info(
        &module.identifier(),
        PrefetchExportsInfoMode::NamedExports(HashSet::from_iter([&dep.name])),
    );
    
    let used_name = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&exports_info),
        *runtime,
        std::slice::from_ref(&dep.name),
    );
    
    if let Some(UsedName::Normal(used)) = used_name {
        // Export is used - generate export code
        init_fragments.push(Box::new(ESMExportInitFragment::new(
            module.get_exports_argument(),
            vec![(used, dep.value.to_string().into())],
        )));
    } else {
        // Export is unused - generate placeholder or omit
        // This enables tree-shaking
    }
}
```

**CommonJS Export Generation**:
```rust
// CommonJsExportsDependencyTemplate
fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
    let used = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&exports_info),
        *runtime,
        &dep.names,
    );
    
    if let Some(UsedName::Normal(used)) = used {
        let export_assignment = format!("{}{}", base, property_access(&used, 0));
        source.replace(dep.range.start, dep.range.end, &export_assignment, None);
    } else {
        // Generate unused export placeholder
        let placeholder_var = "__webpack_unused_export__";
        source.replace(dep.range.start, dep.range.end, &placeholder_var, None);
    }
}
```

#### 4.2 Runtime Export Information Access

**ExportInfoDependency Integration**:
```rust
// ExportInfoDependencyTemplate
fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
    let value = match dep.property.as_str() {
        "used" => {
            let used = ExportsInfoGetter::get_used(&exports_info, export_name, *runtime);
            Some((!matches!(used, UsageState::Unused)).to_string())
        }
        "canMangle" => {
            let can_mangle = ExportInfoGetter::can_mangle(export_info);
            can_mangle.map(|v| v.to_string())
        }
        "usedExports" => {
            let used_exports = exports_info.get_used_exports(*runtime);
            // Serialize used exports array or boolean
            Some(serialize_used_exports(used_exports))
        }
        _ => None,
    };
    
    source.replace(dep.start, dep.end, value.unwrap_or("undefined".to_owned()).as_str(), None);
}
```

## Advanced Integration Scenarios

### 1. Module Federation Integration

The system includes special handling for module federation scenarios:

**ConsumeShared Module Detection**:
```rust
// Check if parent module is ConsumeShared
let consume_shared_info = if let Some(parent_module_id) = module_graph.get_parent_module(&dep.id) {
    if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
        if parent_module.module_type() == &ModuleType::ConsumeShared {
            parent_module.get_consume_shared_key()
        } else {
            None
        }
    } else {
        None
    }
} else {
    None
};
```

**Conditional Export Generation**:
```rust
// Generate tree-shaking macros for module federation
let export_content = if let Some(ref share_key) = consume_shared_info {
    format!(
        "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
        share_key, export_name, export_value
    )
} else {
    export_value.to_string()
};
```

### 2. Nested Export Handling

The system supports complex nested export structures:

**Nested ExportsInfo Creation**:
```rust
// For: export const obj = { prop: { nested: value } }
if let Some(exports) = exports {
    let nested_exports_info = ExportInfoSetter::create_nested_exports_info(&export_info, self.mg);
    self.merge_exports(nested_exports_info, exports, global_export_info.clone(), dep_id);
}
```

**Property Access Tracking**:
```rust
// Track usage of nested properties
for (i, used_export) in used_exports.into_iter().enumerate() {
    let export_info = current_exports_info.get_export_info(&mut module_graph, &used_export);
    
    if i < used_exports.len() - 1 {
        // Not the final property - mark as OnlyPropertiesUsed
        ExportInfoSetter::set_used_conditionally(
            export_info.as_data_mut(&mut module_graph),
            Box::new(|used| used == &UsageState::Unused),
            UsageState::OnlyPropertiesUsed,
            runtime.as_ref(),
        );
        
        // Continue to nested exports
        if let Some(nested_info) = export_info.as_data(&module_graph).exports_info() {
            current_exports_info = nested_info;
        }
    } else {
        // Final property - mark as Used
        ExportInfoSetter::set_used_conditionally(
            export_info.as_data_mut(&mut module_graph),
            Box::new(|v| v != &UsageState::Used),
            UsageState::Used,
            runtime.as_ref(),
        );
    }
}
```

### 3. Dynamic Export Handling

**Unknown Exports Processing**:
```rust
// Handle dynamic exports like require.context()
ExportsOfExportsSpec::UnknownExports => {
    if exports_info.set_unknown_exports_provided(
        self.mg,
        global_can_mangle.unwrap_or_default(),
        export_desc.exclude_exports.as_ref(),    // Known excluded exports
        global_from.map(|_| dep_id),
        global_from.map(|_| dep_id),
        *global_priority,
    ) {
        self.changed = true;
    }
}
```

**Unknown Usage Handling**:
```rust
// When usage pattern is unknown
if used_exports.is_empty() {
    let flag = mgm_exports_info.set_used_in_unknown_way(&mut module_graph, runtime.as_ref());
    if flag {
        queue.enqueue((module_id, runtime.clone()));
    }
}
```

## Performance and Optimization Strategies

### 1. Incremental Processing

**Affected Module Detection**:
```rust
let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::PROVIDED_EXPORTS) {
    // Only process modules affected by changes
    mutations.get_affected_modules_with_module_graph(&compilation.get_module_graph())
} else {
    // Full rebuild
    compilation.get_module_graph().modules().keys().copied().collect()
};
```

### 2. Caching and Prefetching

**Export Information Prefetching**:
```rust
// Prefetch commonly accessed export information
let exports_info = module_graph.get_prefetched_exports_info(
    &module.identifier(),
    match access_pattern {
        PrefetchExportsInfoMode::AllExports => all_exports,
        PrefetchExportsInfoMode::NamedExports(names) => specific_exports,
        PrefetchExportsInfoMode::NamedNestedExports(path) => nested_exports,
        PrefetchExportsInfoMode::Default => basic_info,
    },
);
```

### 3. Change Propagation

**Dependency Invalidation**:
```rust
// Track which modules depend on export information changes
if self.changed {
    self.notify_dependencies(&mut q);
}

fn notify_dependencies(&mut self, q: &mut Queue<ModuleIdentifier>) {
    if let Some(set) = self.dependencies.get(&self.current_module_id) {
        for mi in set.iter() {
            q.enqueue(*mi);  // Re-process dependent modules
        }
    }
}
```

## Debugging and Diagnostics

### 1. Comprehensive Logging

The system includes detailed debug logging for complex scenarios:

```rust
tracing::debug!(
    "[RSPACK_EXPORT_DEBUG:ESM_SPECIFIER_DETAILED] Module: {:?}, Type: {:?}, Layer: {:?}, Name: {:?}, Value: {:?}",
    module_identifier, module.module_type(), module.get_layer(), dep.name, dep.value
);
```

### 2. Export State Visualization

**Usage State Reporting**:
```rust
// Generate comprehensive export usage reports
fn generate_report(&self, compilation: &Compilation) -> Result<ModuleExportReport> {
    let mut modules = HashMap::new();
    
    for (module_id, _module) in module_graph.modules() {
        if let Some(usage_info) = self.analyze_module(&module_graph, &module_id, compilation, &runtimes) {
            modules.insert(module_id.to_string(), usage_info);
        }
    }
    
    Ok(ModuleExportReport {
        modules,
        summary: self.generate_summary(&modules),
        metadata: self.generate_metadata(&runtimes),
        timestamp: current_timestamp(),
    })
}
```

## Plugin Development Integration Points

### Key Integration Points for Plugin Developers

Based on ShareUsagePlugin implementation learnings including the latest enhancement for advanced dependency analysis, the following integration points are essential for plugin developers:

#### 1. Compilation Hook Selection
- **`CompilationFinishModules`**: Best for metadata copying and module information manipulation
- **`CompilerEmit`**: Ideal for asset generation and final analysis reporting
- **`CompilationOptimizeDependencies`**: Use when requiring optimization phase data

#### 2. Export Analysis API Usage
```rust
// Correct pattern for comprehensive export analysis
let exports_info = module_graph.get_exports_info(module_id);
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports, // Efficient bulk operations
);

// Individual export analysis
let export_info_data = prefetched.get_read_only_export_info(&export_atom);
let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);
```

#### 3. Advanced Dependency Analysis (Latest Enhancement)
```rust
// Use incoming connections for accurate ConsumeShared analysis
for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Extract specific export names using get_referenced_exports()
        let referenced_exports = dependency.get_referenced_exports(
            module_graph,
            &rspack_core::ModuleGraphCacheArtifact::default(),
            None,
        );
        
        // Handle ExtendedReferencedExport patterns
        for export_ref in referenced_exports {
            match export_ref {
                ExtendedReferencedExport::Array(names) => {
                    // Multiple specific exports referenced
                    for name in names {
                        let export_name = name.to_string();
                        // Process specific export usage
                    }
                },
                ExtendedReferencedExport::Export(export_info) => {
                    // Single export or namespace reference
                    if export_info.name.is_empty() {
                        // Namespace usage detected
                        uses_namespace = true;
                    } else {
                        // Specific named exports
                        for name in export_info.name {
                            let export_name = name.to_string();
                            // Process named export
                        }
                    }
                },
            }
        }
    }
}
```

#### 4. ConsumeShared Module Considerations
- Empty usage arrays on ConsumeShared modules are expected behavior (proxy pattern)
- Real usage data requires analyzing incoming dependencies using `get_referenced_exports()` or fallback modules
- Use dependency graph traversal with `ExtendedReferencedExport` pattern matching for accurate ConsumeShared analysis
- Cross-reference extracted usage with provided exports for accurate filtering

For comprehensive plugin development patterns incorporating these integration insights, see **[09_plugin_development_patterns.md](./09_plugin_development_patterns.md)**.

## Latest Enhancement: Advanced Dependency Analysis

The ShareUsagePlugin investigation revealed and implemented a significant enhancement to the export usage tracking system:

### Enhanced ConsumeShared Analysis

The latest enhancement introduces sophisticated dependency analysis using incoming connections:

```rust
// Enhanced analysis using module_graph.get_incoming_connections()
for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Use dependency.get_referenced_exports() to extract specific export names
        let referenced_exports = dependency.get_referenced_exports(
            module_graph,
            &rspack_core::ModuleGraphCacheArtifact::default(),
            None,
        );
        
        // Handle ExtendedReferencedExport patterns comprehensively
        for export_ref in referenced_exports {
            match export_ref {
                ExtendedReferencedExport::Array(names) => {
                    // Process multiple specific exports
                },
                ExtendedReferencedExport::Export(export_info) => {
                    // Process single export or namespace reference
                },
            }
        }
    }
}
```

**Key Enhancement Features:**
1. **Incoming Connection Analysis**: Uses `module_graph.get_incoming_connections()` to find all modules that import from ConsumeShared modules
2. **Referenced Export Extraction**: Calls `dependency.get_referenced_exports()` to extract specific export names being used
3. **Pattern Matching**: Handles both `ExtendedReferencedExport::Array` and `ExtendedReferencedExport::Export` patterns
4. **Cross-referencing**: Compares used exports with provided exports for accurate filtering

### Integration with Existing System

This enhancement seamlessly integrates with the existing four-phase compilation process:

1. **Phase 1-2**: Standard export discovery and provision analysis
2. **Phase 3**: Enhanced usage analysis with incoming connection analysis
3. **Phase 4**: Code generation with more accurate usage information

## Conclusion

The export usage tracking system in rspack represents a sophisticated integration of multiple components working together to enable precise tree-shaking and export optimization. Through the coordinated effort of dependency analysis, export provision tracking, usage analysis, and optimized code generation, the system can eliminate dead code while maintaining correctness across complex module relationships. The integration handles advanced scenarios including module federation, nested exports, dynamic patterns, and performance-critical optimizations, making it suitable for large-scale applications with complex dependency graphs.

The latest enhancement using `get_referenced_exports()` and incoming connection analysis further improves the accuracy of ConsumeShared module analysis, providing more precise usage information for tree-shaking decisions.

The workflow demonstrates how modern bundlers can achieve both comprehensive analysis and excellent performance through careful architecture design, incremental processing, and strategic caching. This system serves as a foundation for advanced optimizations while maintaining the flexibility needed for evolving JavaScript module patterns.

The ShareUsagePlugin implementation insights documented throughout this system, including the latest enhancement for advanced dependency analysis using `get_referenced_exports()`, provide essential patterns for plugin developers working with export analysis, module graph manipulation, and compilation hooks, ensuring robust and efficient plugin development within the rspack ecosystem.

**Latest Enhancement Summary:**
1. **Plugin Implementation**: Successfully created ShareUsagePlugin with proper API usage and advanced dependency analysis
2. **Export Analysis APIs**: Correct usage of ExportsInfoGetter, ExportInfoGetter, and dependency analysis with `get_referenced_exports()`
3. **ConsumeShared Behavior**: Confirmed that empty usage arrays are expected, enhanced analysis through incoming connections
4. **Advanced Dependency Analysis**: New pattern using `get_referenced_exports()` for extracting actual usage data from importing modules
5. **Module Federation Integration**: Proper integration with existing export metadata copying systems and enhanced usage tracking
6. **Pattern Matching**: Comprehensive handling of `ExtendedReferencedExport::Array` and `ExtendedReferencedExport::Export` patterns
7. **Cross-reference Analysis**: Implementation of usage-to-provided export filtering for accurate optimization decisions