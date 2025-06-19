# Flag Dependency Plugins Analysis

## Overview

The Flag Dependency plugins are the cornerstone of rspack's tree-shaking system. These plugins work in tandem to collect comprehensive export information and track usage patterns across the module graph, enabling precise dead code elimination and export optimization.

## Plugin Architecture

### 1. Two-Phase Analysis System

```rust
// Phase 1: Export Provision Analysis
FlagDependencyExportsPlugin -> Identifies what exports are provided
// Phase 2: Export Usage Analysis  
FlagDependencyUsagePlugin -> Tracks which exports are actually used
```

**Execution Order**:
1. **FlagDependencyExportsPlugin** runs during `CompilationFinishModules`
2. **FlagDependencyUsagePlugin** runs during `CompilationOptimizeDependencies`

This ensures that all export information is collected before usage analysis begins.

## FlagDependencyExportsPlugin

### Core Responsibilities

1. **Export Provision Tracking**: Determines what exports each module provides
2. **Export Metadata Collection**: Gathers information about mangling capabilities, inlining potential, and terminal bindings
3. **Dependency Chain Analysis**: Tracks how exports flow through re-exports and module connections

### Implementation Architecture

#### 1. FlagDependencyExportsState

```rust
struct FlagDependencyExportsState<'a> {
    mg: &'a mut ModuleGraph<'a>,              // Module graph for export information
    mg_cache: &'a ModuleGraphCacheArtifact,   // Caching for performance
    changed: bool,                            // Tracks if any changes occurred
    current_module_id: ModuleIdentifier,      // Currently processing module
    dependencies: IdentifierMap<IdentifierSet>, // Dependency tracking for invalidation
}
```

#### 2. Processing Algorithm

**Queue-Based Processing**:
```rust
pub fn apply(&mut self, modules: IdentifierSet) {
    let mut q = Queue::new();
    
    // 1. Initialize all modules and reset export information
    for module_id in modules {
        let exports_info = mgm.exports;
        exports_info.reset_provide_info(self.mg);
        
        // Handle modules without exports
        let is_module_without_exports = 
            module.build_meta().exports_type == BuildMetaExportsType::Unset;
        if is_module_without_exports {
            exports_info.set_unknown_exports_provided(self.mg, false, None, None, None, None);
            continue;
        }
        
        exports_info.set_has_provide_info(self.mg);
        q.enqueue(module_id);
    }
    
    // 2. Process modules until no more changes occur
    while let Some(module_id) = q.dequeue() {
        self.changed = false;
        self.current_module_id = module_id;
        
        // Process all dependencies to collect export specifications
        self.process_dependencies_block(&module_id, &mut exports_specs_from_dependencies);
        
        // Apply collected export specifications
        let exports_info = self.mg.get_exports_info(&module_id);
        for (dep_id, exports_spec) in exports_specs_from_dependencies.iter() {
            self.process_exports_spec(*dep_id, exports_spec, exports_info);
        }
        
        // If changes occurred, notify dependent modules
        if self.changed {
            self.notify_dependencies(&mut q);
        }
    }
}
```

#### 3. Export Specification Processing

**ExportsSpec Analysis**:
```rust
pub fn process_exports_spec(&mut self, dep_id: DependencyId, export_desc: &ExportsSpec, exports_info: ExportsInfo) {
    match &export_desc.exports {
        ExportsOfExportsSpec::UnknownExports => {
            // Handle dynamic exports (require.context, etc.)
            if exports_info.set_unknown_exports_provided(
                self.mg,
                global_can_mangle.unwrap_or_default(),
                export_desc.exclude_exports.as_ref(),
                global_from.map(|_| dep_id),
                global_from.map(|_| dep_id),
                *global_priority,
            ) {
                self.changed = true;
            }
        }
        ExportsOfExportsSpec::NoExports => {
            // Module provides no exports
        }
        ExportsOfExportsSpec::Names(ele) => {
            // Named exports - most common case
            self.merge_exports(exports_info, ele, DefaultExportInfo {
                can_mangle: *global_can_mangle,
                terminal_binding: global_terminal_binding,
                from: global_from,
                priority: *global_priority,
            }, dep_id);
        }
    }
    
    // Track dependency relationships for invalidation
    if let Some(export_dependencies) = export_dependencies {
        for export_dep in export_dependencies {
            self.dependencies.entry(*export_dep)
                .or_insert_with(IdentifierSet::new)
                .insert(self.current_module_id);
        }
    }
}
```

#### 4. Export Merging Logic

**Complex Export Handling**:
```rust
pub fn merge_exports(&mut self, exports_info: ExportsInfo, exports: &Vec<ExportNameOrSpec>, global_export_info: DefaultExportInfo, dep_id: DependencyId) {
    for export_name_or_spec in exports {
        let (name, can_mangle, terminal_binding, exports, from, from_export, priority, hidden, inlinable) = 
            self.extract_export_properties(export_name_or_spec, &global_export_info);
        
        let export_info = exports_info.get_export_info(self.mg, &name);
        let export_info_data = export_info.as_data_mut(self.mg);
        
        // Update provision status
        if let Some(provided) = export_info_data.provided() 
            && matches!(provided, ExportProvided::NotProvided | ExportProvided::Unknown) {
            export_info_data.set_provided(Some(ExportProvided::Provided));
            self.changed = true;
        }
        
        // Update mangling capabilities
        if Some(false) != export_info_data.can_mangle_provide() && can_mangle == Some(false) {
            export_info_data.set_can_mangle_provide(Some(false));
            self.changed = true;
        }
        
        // Update inlining capabilities
        if let Some(inlined) = inlinable && !export_info_data.inlinable().can_inline() {
            export_info_data.set_inlinable(Inlinable::Inlined(inlined));
            self.changed = true;
        }
        
        // Handle nested exports (object properties)
        if let Some(exports) = exports {
            let nested_exports_info = ExportInfoSetter::create_nested_exports_info(&export_info, self.mg);
            self.merge_exports(nested_exports_info, exports, global_export_info.clone(), dep_id);
        }
        
        // Set up target relationships for re-exports
        if let Some(from) = from {
            let changed = if hidden {
                ExportInfoSetter::unset_target(export_info_data, &dep_id)
            } else {
                ExportInfoSetter::set_target(
                    export_info_data,
                    Some(dep_id),
                    Some(from.dependency_id),
                    export_name,
                    priority,
                )
            };
            self.changed |= changed;
        }
    }
}
```

### How Dependencies Add Exports

Dependencies declare their exports through the `get_exports()` method, which returns an `ExportsSpec`:

#### 1. Simple Named Export
```rust
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![
            ExportNameOrSpec::String("myExport".into())
        ]),
        terminal_binding: Some(true),
        ..Default::default()
    })
}
```

#### 2. Export with Metadata
```rust
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![
            ExportNameOrSpec::ExportSpec(ExportSpec {
                name: "complexExport".into(),
                can_mangle: Some(false),
                terminal_binding: Some(true),
                priority: Some(1),
                inlinable: Some(EvaluatedInlinableValue::String("value".into())),
                ..Default::default()
            })
        ]),
        ..Default::default()
    })
}
```

#### 3. Re-export from Another Module
```rust
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![
            ExportNameOrSpec::ExportSpec(ExportSpec {
                name: "reexported".into(),
                export: Some(Nullable::Value(vec!["originalName".into()])),
                from: Some(target_connection),
                ..Default::default()
            })
        ]),
        from: Some(target_connection),
        ..Default::default()
    })
}
```

#### 4. Dynamic Exports
```rust
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::UnknownExports, // For require.context(), etc.
        can_mangle: Some(false),
        ..Default::default()
    })
}
```
```

## FlagDependencyUsagePlugin

### Core Responsibilities

1. **Usage State Tracking**: Marks exports as Used, Unused, OnlyPropertiesUsed, etc.
2. **Entry Point Analysis**: Starts usage analysis from application entry points
3. **Transitive Usage**: Follows module dependencies to track usage propagation
4. **Side Effects Handling**: Accounts for modules that must be executed for side effects

### Implementation Architecture

#### 1. FlagDependencyUsagePluginProxy

```rust
pub struct FlagDependencyUsagePluginProxy<'a> {
    global: bool,                             // Global vs per-runtime analysis
    compilation: &'a mut Compilation,         // Compilation context
    exports_info_module_map: UkeyMap<ExportsInfo, ModuleIdentifier>, // Reverse mapping
}
```

#### 2. Entry Point Processing

**Starting from Entry Dependencies**:
```rust
fn apply(&mut self) {
    // Initialize all exports info with usage tracking
    for exports_info in self.exports_info_module_map.keys() {
        exports_info.set_has_use_info(mg);
    }
    
    // Process entry points
    for (entry_name, entry) in entries.iter() {
        let runtime = if self.global {
            None
        } else {
            Some(get_entry_runtime(entry_name, &entry.options, &entries))
        };
        
        for &dep in entry.dependencies.iter() {
            self.process_entry_dependency(dep, runtime.clone(), &mut q);
        }
    }
    
    // Process all modules reachable from entries
    while let Some((module_id, runtime)) = q.dequeue() {
        self.process_module(ModuleOrAsyncDependenciesBlock::Module(module_id), runtime, false, &mut q);
    }
}
```

#### 3. Module Processing Algorithm

**Comprehensive Dependency Analysis**:
```rust
fn process_module(&mut self, block_id: ModuleOrAsyncDependenciesBlock, runtime: Option<RuntimeSpec>, force_side_effects: bool, q: &mut Queue<(ModuleIdentifier, Option<RuntimeSpec>)>) {
    let mut map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(block_id);
    
    // Traverse all dependencies in the module
    while let Some(module_id) = queue.pop_front() {
        let (blocks, dependencies) = self.get_module_dependencies(module_id);
        
        // Process async dependency blocks
        for block_id in blocks {
            if !self.global && has_entrypoint_options(&block_id) {
                let runtime = RuntimeSpec::from_entry_options(options);
                self.process_module(AsyncDependenciesBlock(block_id), runtime, true, q);
            } else {
                queue.push_back(AsyncDependenciesBlock(block_id));
            }
        }
        
        // Process each dependency
        for dep_id in dependencies {
            let connection = module_graph.connection_by_dependency_id(&dep_id);
            let active_state = connection.active_state(&module_graph, runtime.as_ref(), module_graph_cache);
            
            match active_state {
                ConnectionState::Active(false) => continue,
                ConnectionState::TransitiveOnly => {
                    // Module is needed but exports aren't used
                    self.process_module(Module(*connection.module_identifier()), runtime.clone(), false, q);
                    continue;
                }
                _ => {}
            }
            
            // Get referenced exports from dependency
            let referenced_exports = if let Some(md) = dep.as_module_dependency() {
                md.get_referenced_exports(&module_graph, module_graph_cache, runtime.as_ref())
            } else if dep.as_context_dependency().is_some() {
                vec![ExtendedReferencedExport::Array(vec![])]
            } else {
                continue;
            };
            
            // Merge with existing references
            self.merge_referenced_exports(connection.module_identifier(), referenced_exports, &mut map);
        }
    }
    
    // Process all referenced modules
    for (module_id, referenced_exports) in map {
        self.process_referenced_module(module_id, referenced_exports, runtime.clone(), force_side_effects, q);
    }
}
```

#### 4. Referenced Export Processing

**Usage State Application**:
```rust
fn process_referenced_module(&mut self, module_id: ModuleIdentifier, used_exports: Vec<ExtendedReferencedExport>, runtime: Option<RuntimeSpec>, force_side_effects: bool, queue: &mut Queue<(ModuleIdentifier, Option<RuntimeSpec>)>) {
    let mgm_exports_info = mgm.exports;
    
    if !used_exports.is_empty() {
        // Handle modules without export information
        if matches!(module.build_meta().exports_type, BuildMetaExportsType::Unset) {
            let flag = mgm_exports_info.set_used_without_info(&mut module_graph, runtime.as_ref());
            if flag {
                queue.enqueue((module_id, None));
            }
            return;
        }
        
        // Process each used export
        for used_export_info in used_exports {
            let (used_exports, can_mangle, can_inline) = match used_export_info {
                ExtendedReferencedExport::Array(used_exports) => (used_exports, true, true),
                ExtendedReferencedExport::Export(export) => (export.name, export.can_mangle, export.can_inline),
            };
            
            if used_exports.is_empty() {
                // Unknown usage pattern - mark as used in unknown way
                let flag = mgm_exports_info.set_used_in_unknown_way(&mut module_graph, runtime.as_ref());
                if flag {
                    queue.enqueue((module_id, runtime.clone()));
                }
            } else {
                // Track specific export usage
                let mut current_exports_info = mgm_exports_info;
                for (i, used_export) in used_exports.into_iter().enumerate() {
                    let export_info = current_exports_info.get_export_info(&mut module_graph, &used_export);
                    
                    // Apply mangling and inlining constraints
                    if !can_mangle {
                        export_info.as_data_mut(&mut module_graph).set_can_mangle_use(Some(false));
                    }
                    if !can_inline {
                        export_info.as_data_mut(&mut module_graph).set_inlinable(Inlinable::NoByUse);
                    }
                    
                    let last_one = i == used_exports.len() - 1;
                    let usage_state = if last_one {
                        UsageState::Used
                    } else {
                        UsageState::OnlyPropertiesUsed
                    };
                    
                    // Set usage state conditionally
                    let changed_flag = ExportInfoSetter::set_used_conditionally(
                        export_info.as_data_mut(&mut module_graph),
                        Box::new(|used| used != &usage_state),
                        usage_state,
                        runtime.as_ref(),
                    );
                    
                    if changed_flag {
                        queue.enqueue((module_id, runtime.clone()));
                    }
                    
                    // Continue to nested exports if not the last one
                    if !last_one {
                        if let Some(nested_info) = export_info.as_data(&module_graph).exports_info() {
                            current_exports_info = nested_info;
                        }
                    }
                }
            }
        }
    } else {
        // Module is used for side effects only
        if !force_side_effects && is_side_effect_free(&module) {
            return;
        }
        
        let changed_flag = mgm_exports_info.set_used_for_side_effects_only(&mut module_graph, runtime.as_ref());
        if changed_flag {
            queue.enqueue((module_id, runtime));
        }
    }
}
```

### How Dependencies Set Referenced Exports

Dependencies specify which exports they use through the `get_referenced_exports()` method:

#### 1. Specific Export References
```rust
fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
) -> Vec<ExtendedReferencedExport> {
    vec![
        // Reference to obj.specific.export
        ExtendedReferencedExport::Array(vec!["specific".into(), "export".into()]),
        // Reference with usage constraints
        ExtendedReferencedExport::Export(ReferencedExport {
            name: vec!["another".into()],
            can_mangle: false, // Cannot be mangled
            can_inline: true,  // Can be inlined
        }),
    ]
}
```

#### 2. Namespace References
```rust
fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
) -> Vec<ExtendedReferencedExport> {
    // Empty array means reference to entire exports object
    create_exports_object_referenced()
}
```

#### 3. No References
```rust
fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
) -> Vec<ExtendedReferencedExport> {
    // Module doesn't reference any exports
    create_no_exports_referenced()
}
```

#### 4. Conditional References
```rust
fn get_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    runtime: Option<&RuntimeSpec>,
) -> Vec<ExtendedReferencedExport> {
    let mut references = Vec::new();
    
    // Conditional logic based on module analysis
    if self.should_reference_default(module_graph) {
        references.push(ExtendedReferencedExport::Array(vec!["default".into()]));
    }
    
    if self.should_reference_named(module_graph) {
        references.push(ExtendedReferencedExport::Array(vec!["namedExport".into()]));
    }
    
    references
}
```

### Referenced Export Types and Helpers

```rust
pub enum ExtendedReferencedExport {
    Array(Vec<Atom>),           // Path to specific export
    Export(ReferencedExport),   // Export with metadata
}

pub struct ReferencedExport {
    pub name: Vec<Atom>,   // Export path
    pub can_mangle: bool,  // Mangling constraint
    pub can_inline: bool,  // Inlining hint
}

// Utility functions
pub fn create_no_exports_referenced() -> Vec<ExtendedReferencedExport> {
    vec![]
}

pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![])]  // Empty = entire exports object
}
```

## Performance Optimizations

### 1. Incremental Processing

**FlagDependencyExportsPlugin**:
```rust
let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::PROVIDED_EXPORTS) {
    // Only process affected modules
    mutations.get_affected_modules_with_module_graph(&compilation.get_module_graph())
} else {
    // Full rebuild - process all modules
    compilation.get_module_graph().modules().keys().copied().collect()
};
```

### 2. Caching Strategy

**Module Graph Caching**:
```rust
// Freeze cache during processing for consistency
self.mg_cache.freeze();
self.process_dependencies_block(&module_id, &mut exports_specs_from_dependencies, self.mg_cache);
self.mg_cache.unfreeze();
```

### 3. Change Tracking

**Efficient Invalidation**:
```rust
// Track dependency relationships for targeted invalidation
if let Some(export_dependencies) = export_dependencies {
    for export_dep in export_dependencies {
        match self.dependencies.entry(*export_dep) {
            Entry::Occupied(mut occ) => {
                occ.get_mut().insert(self.current_module_id);
            }
            Entry::Vacant(vac) => {
                vac.insert(IdentifierSet::from_iter([self.current_module_id]));
            }
        }
    }
}
```

## Export State Management

### 1. ExportProvided States

```rust
pub enum ExportProvided {
    Provided,      // Export is definitely provided
    NotProvided,   // Export is definitely not provided
    Unknown,       // Export provision is unknown (dynamic)
}
```

### 2. UsageState States

```rust
pub enum UsageState {
    Used,                 // Export is used
    OnlyPropertiesUsed,   // Only properties of export are used
    Unused,               // Export is not used
    NoInfo,               // No usage information available
    Unknown,              // Usage is unknown (dynamic)
}
```

### 3. Inlinable States

```rust
pub enum Inlinable {
    Inlined(EvaluatedInlinableValue), // Can be inlined with specific value
    NoByUse,                          // Cannot inline due to usage pattern
    NoByProvide,                      // Cannot inline due to provision pattern
}
```

## Integration with Module Graph

### 1. ExportsInfo Structure

The plugins work with the central `ExportsInfo` data structure:

```rust
// Each module has an ExportsInfo that tracks:
// - Individual export information (ExportInfo)
// - Overall export state
// - Nested export structures
// - Usage and provision metadata
```

### 2. Target Relationships

For re-exports, the system tracks target relationships:

```rust
// Re-export: export { foo } from './module'
// Creates target relationship: current_module.foo -> target_module.foo
let target = get_target(export_info_data, self.mg);
if let Some(target) = target {
    // Track dependency for invalidation
    self.dependencies.entry(target.module)
        .or_insert_with(IdentifierSet::new)
        .insert(self.current_module_id);
}
```

## Error Handling and Edge Cases

### 1. Modules Without Exports

```rust
let is_module_without_exports = module.build_meta().exports_type == BuildMetaExportsType::Unset;
if is_module_without_exports {
    exports_info.set_unknown_exports_provided(self.mg, false, None, None, None, None);
    continue;
}
```

### 2. Dynamic Exports

```rust
// Handle require.context and other dynamic patterns
ExportsOfExportsSpec::UnknownExports => {
    exports_info.set_unknown_exports_provided(
        self.mg,
        global_can_mangle.unwrap_or_default(),
        export_desc.exclude_exports.as_ref(),
        global_from.map(|_| dep_id),
        global_from.map(|_| dep_id),
        *global_priority,
    );
}
```

### 3. Side Effect Handling

```rust
// Modules used only for side effects
if !force_side_effects && is_side_effect_free(&module) {
    return;
}
let changed_flag = mgm_exports_info.set_used_for_side_effects_only(&mut module_graph, runtime.as_ref());
```

## Conclusion

The Flag Dependency plugins implement a sophisticated two-phase analysis system that forms the foundation of rspack's tree-shaking capabilities. Through careful export provision tracking and comprehensive usage analysis, these plugins enable precise dead code elimination while handling complex scenarios including re-exports, dynamic imports, side effects, and module federation patterns. The system is optimized for performance with incremental processing, caching strategies, and efficient invalidation mechanisms, making it suitable for large-scale applications with complex dependency graphs.