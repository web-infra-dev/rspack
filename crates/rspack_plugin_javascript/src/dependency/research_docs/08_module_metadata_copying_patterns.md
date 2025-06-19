# Module Metadata Copying Patterns and Best Practices

## Overview

This document consolidates comprehensive research into module information copying patterns in the rspack codebase, providing definitive guidance on when and how to copy, merge, and transfer metadata between modules.

## Core Metadata Types

### 1. Build Metadata
- **`build_meta`**: Build-time metadata including ESM info, async module state, side effects
- **`build_info`**: Build information including asset dependencies, file dependencies, context dependencies
- **`factory_meta`**: Factory creation metadata preserved during module transformations

### 2. Export Information
- **`ExportsInfo`**: Per-module export collection with bulk operations
- **`ExportInfo`**: Individual export metadata including usage state, mangling capabilities, targets
- **`ProvidedExports`**: What exports a module provides (`ProvidedNames`, `ProvidedAll`, `Unknown`)

### 3. Usage Information
- **`UsageState`**: Export usage state (`Unused`, `OnlyPropertiesUsed`, `NoInfo`, `Unknown`, `Used`)
- **Runtime-specific usage**: Per-runtime tracking for code splitting scenarios
- **Referenced exports**: Dependency-driven usage information

## Established Patterns in Rspack Codebase

### 1. **Proxy Module Pattern** (ConsumeShared)

**Location**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_mf/src/sharing/consume_shared_module.rs`

**Purpose**: Module Federation proxy modules that inherit all metadata from fallback modules

**Implementation**:
```rust
/// Copies metadata from fallback module to make ConsumeShared act as true proxy
pub fn copy_metadata_from_fallback(&mut self, module_graph: &mut ModuleGraph) -> Result<()> {
    if let Some(fallback_id) = self.find_fallback_module_id(module_graph) {
        if let Some(fallback_module) = module_graph.module_by_identifier(&fallback_id) {
            // Phase 1: Copy build metadata
            self.build_meta = fallback_module.build_meta().clone();
            self.build_info = fallback_module.build_info().clone();

            // Phase 2: Copy export information
            self.copy_exports_from_fallback(module_graph, &fallback_id)?;
        }
    }
    Ok(())
}

/// Comprehensive export information copying
fn copy_exports_from_fallback(&self, module_graph: &mut ModuleGraph, fallback_id: &ModuleIdentifier) -> Result<()> {
    let fallback_exports_info = module_graph.get_exports_info(fallback_id);
    let consume_shared_exports_info = module_graph.get_exports_info(&self.identifier());

    // Use prefetched analysis for efficiency
    let prefetched_fallback = ExportsInfoGetter::prefetch(
        &fallback_exports_info,
        module_graph,
        PrefetchExportsInfoMode::AllExports,
    );

    match prefetched_fallback.get_provided_exports() {
        ProvidedExports::ProvidedNames(export_names) => {
            // Copy each specific export with full metadata
            for export_name in export_names {
                let consume_shared_export_info = consume_shared_exports_info.get_export_info(module_graph, &export_name);
                let fallback_export_info = fallback_exports_info.get_export_info(module_graph, &export_name);

                // Copy provided status
                if let Some(provided) = fallback_export_info.as_data(module_graph).provided() {
                    consume_shared_export_info.as_data_mut(module_graph).set_provided(Some(provided));
                }

                // Copy mangling capabilities
                if let Some(can_mangle) = fallback_export_info.as_data(module_graph).can_mangle_provide() {
                    consume_shared_export_info.as_data_mut(module_graph).set_can_mangle_provide(Some(can_mangle));
                }

                // Copy nested export structures
                if let Some(nested_exports_info) = fallback_export_info.as_data(module_graph).exports_info() {
                    consume_shared_export_info.as_data_mut(module_graph).set_exports_info(Some(nested_exports_info));
                }
            }

            // Mark as having complete provide info
            consume_shared_exports_info.set_has_provide_info(module_graph);
            
            // Set unknown exports to not provided
            consume_shared_exports_info.set_unknown_exports_provided(
                module_graph,
                false, // not provided
                None, None, None, None,
            );
        }
        ProvidedExports::ProvidedAll => {
            // Inherit dynamic export capability
            consume_shared_exports_info.set_unknown_exports_provided(
                module_graph,
                true, // provided
                None, None, None, None,
            );
            consume_shared_exports_info.set_has_provide_info(module_graph);
        }
        ProvidedExports::Unknown => {
            // Preserve unknown status - no copying needed
        }
    }

    Ok(())
}
```

**When to Use**: Module Federation scenarios where modules must act as transparent proxies

### 2. **Error Recovery Pattern** (FixBuildMeta)

**Location**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_core/src/compiler/make/cutout/fix_build_meta.rs`

**Purpose**: Preserve build metadata when module builds fail

**Implementation**:
```rust
// Save original metadata before rebuild
pub fn analyze_force_build_module(&mut self, artifact: &MakeArtifact, module_identifier: &ModuleIdentifier) {
    let module = module_graph.module_by_identifier(module_identifier).expect("should have module");
    self.origin_module_build_meta.insert(*module_identifier, module.build_meta().clone());
}

// Restore metadata if build fails
pub fn fix_artifact(&mut self, artifact: &mut MakeArtifact, failed_module: &ModuleIdentifier) {
    if let Some(build_meta) = self.origin_module_build_meta.get(failed_module) {
        if let Some(mut module) = module_graph.module_by_identifier_mut(failed_module) {
            if module.first_error().is_some() {
                *module.build_meta_mut() = build_meta.clone();
            }
        }
    }
}
```

**When to Use**: Error recovery scenarios, incremental compilation with rollback needs

### 3. **Export Merging Pattern** (FlagDependencyExportsPlugin)

**Location**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`

**Purpose**: Merge export specifications from dependencies into module export info

**Implementation**:
```rust
pub fn merge_exports(&mut self, exports_info: ExportsInfo, exports: &Vec<ExportNameOrSpec>, global_export_info: DefaultExportInfo, dep_id: DependencyId) {
    for export_name_or_spec in exports {
        let export_info = exports_info.get_export_info(self.mg, &name);
        let export_info_data = export_info.as_data_mut(self.mg);
        
        // Merge provided status
        if matches!(export_info_data.provided(), Some(ExportProvided::NotProvided | ExportProvided::Unknown)) {
            export_info_data.set_provided(Some(ExportProvided::Provided));
            self.changed = true;
        }

        // Merge mangling capabilities
        if Some(false) != export_info_data.can_mangle_provide() && can_mangle == Some(false) {
            export_info_data.set_can_mangle_provide(Some(false));
            self.changed = true;
        }

        // Set target for re-exports
        if let Some(from) = from {
            let changed = ExportInfoSetter::set_target(
                export_info_data,
                Some(dep_id),
                Some(from.dependency_id),
                export_name,
                priority,
            );
            self.changed |= changed;
        }

        // Recursive merge for nested exports
        if let Some(exports) = exports {
            let nested_exports_info = ExportInfoSetter::create_nested_exports_info(&export_info, self.mg);
            self.merge_exports(nested_exports_info, exports, global_export_info.clone(), dep_id);
        }
    }
}
```

**When to Use**: Dependency analysis plugins that need to accumulate export information

### 4. **Template Initialization Pattern** (ExportInfo)

**Location**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_core/src/exports/export_info.rs`

**Purpose**: Initialize new ExportInfo from existing template with property inheritance

**Implementation**:
```rust
pub fn new(name: Option<Atom>, init_from: Option<&ExportInfoData>) -> Self {
    let used_name = init_from.and_then(|init_from| init_from.used_name.clone());
    let global_used = init_from.and_then(|init_from| init_from.global_used);
    let used_in_runtime = init_from.and_then(|init_from| init_from.used_in_runtime.clone());
    let has_use_in_runtime_info = init_from.is_some_and(|init_from| init_from.has_use_in_runtime_info);

    let provided = init_from.and_then(|init_from| init_from.provided);
    let terminal_binding = init_from.is_some_and(|init_from| init_from.terminal_binding);
    let can_mangle_provide = init_from.and_then(|init_from| init_from.can_mangle_provide);
    let can_mangle_use = init_from.and_then(|init_from| init_from.can_mangle_use);

    // Target copying with name transformation
    let target = init_from.and_then(|item| {
        if item.target_is_set {
            Some(/* transform targets with new name */)
        } else {
            None
        }
    }).unwrap_or_default();

    ExportInfoData {
        name,
        used_name,
        global_used,
        used_in_runtime,
        has_use_in_runtime_info,
        provided,
        terminal_binding,
        can_mangle_provide,
        can_mangle_use,
        target,
        target_is_set: target.is_set(),
        /* ... other fields */
    }
}
```

**When to Use**: Creating new exports based on existing export templates

### 5. **DLL Delegation Pattern** (DelegatedModule)

**Location**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_dll/src/dll_reference/delegated_module.rs`

**Purpose**: Inherit metadata from DLL manifest

**Implementation**:
```rust
async fn build(&mut self, _build_context: BuildContext, _compilation: Option<&Compilation>) -> Result<BuildResult> {
    // Copy build meta from DLL manifest
    self.build_meta = self.delegate_data.build_meta.clone();
    
    let dependencies = vec![
        Box::new(DelegatedSourceDependency::new(self.source_request.clone())),
        Box::new(StaticExportsDependency::new(/* DLL exports */)) as BoxDependency,
    ];
    
    Ok(BuildResult { dependencies, ..Default::default() })
}
```

**When to Use**: DLL reference scenarios where delegated modules inherit DLL characteristics

## Best Practices

### 1. **Two-Phase Copying Approach**

**Phase 1: Build Metadata**
```rust
// Copy build-time characteristics
proxy_module.build_meta = source_module.build_meta().clone();
proxy_module.build_info = source_module.build_info().clone();
```

**Phase 2: Export Information**
```rust
// Copy export metadata using prefetched analysis
let prefetched = ExportsInfoGetter::prefetch(&source_exports_info, module_graph, PrefetchExportsInfoMode::AllExports);
// ... detailed export copying
```

### 2. **Plugin Hook Integration**

**Recommended Hook**: `CompilationFinishModules`
- **Timing**: After all modules are built and analyzed, before optimization
- **Access**: Full module graph with build metadata available
- **Safety**: Avoids borrow checker issues with sequential processing

```rust
#[plugin_hook(CompilationFinishModules for YourPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    // Find modules requiring metadata copying
    let target_modules: Vec<ModuleIdentifier> = /* collect */;
    
    // Process each individually to avoid borrow checker issues
    for module_id in target_modules {
        Self::copy_metadata_between_modules(compilation, &module_id)?;
    }
    
    Ok(())
}
```

### 3. **Efficient Export Information Access**

**Use Prefetched Analysis**:
```rust
// Efficient: Bulk prefetch for multiple operations
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports
);

// Efficient: Individual export access through prefetched wrapper
let export_info_data = prefetched.get_read_only_export_info(&export_atom);
let usage_state = ExportInfoGetter::get_used(export_info_data, runtime);

// Avoid: Repeated individual module graph access
// let export_info = exports_info.get_export_info(module_graph, &export_name); // Less efficient
```

### 4. **Usage State Management**

**Setting Usage States**:
```rust
// Basic usage setting
ExportInfoSetter::set_used(export_info_data, UsageState::Used, runtime);

// Conditional usage setting (only if condition met)
ExportInfoSetter::set_used_conditionally(
    export_info_data,
    Box::new(|current| current != &UsageState::Used),
    UsageState::Used,
    runtime
);

// Disable optimizations when usage unclear
ExportInfoSetter::set_used_without_info(export_info_data, runtime);
```

**Bulk Operations**:
```rust
// Set all exports used
exports_info.set_all_known_exports_used(module_graph, runtime);

// Mark as having complete usage info
exports_info.set_has_use_info(module_graph);
```

### 5. **Borrow Checker Patterns**

**Separate Scope Approach**:
```rust
// Problematic: Multiple mutable borrows
let mut module_graph = compilation.get_module_graph_mut();
let source_module = module_graph.module_by_identifier(&source_id); // Borrows mutably
let target_module = module_graph.module_by_identifier_mut(&target_id); // Second mutable borrow

// Solution: Separate scopes
let source_metadata = {
    let module_graph = compilation.get_module_graph(); // Immutable borrow
    let source_module = module_graph.module_by_identifier(&source_id)?;
    (source_module.build_meta().clone(), source_module.build_info().clone())
};

{
    let mut module_graph = compilation.get_module_graph_mut(); // New mutable borrow
    let mut target_module = module_graph.module_by_identifier_mut(&target_id)?;
    target_module.build_meta = source_metadata.0;
    target_module.build_info = source_metadata.1;
}
```

### 6. **Error Handling and Diagnostics**

**Graceful Failure**:
```rust
if let Err(e) = module.copy_metadata_from_source(&mut module_graph) {
    compilation.push_diagnostic(
        rspack_error::Diagnostic::warn(
            "ModuleMetadataCopyPlugin".into(),
            format!("Failed to copy metadata: {}", e),
        )
    );
}
```

## Common Use Cases

### 1. **Proxy Modules**
- **ConsumeShared**: Module Federation proxy modules
- **LazyCompilation**: Proxy modules for lazy-loaded content
- **DelegatedModule**: DLL reference proxies

### 2. **Module Transformation**
- **Concatenation**: Preserving metadata during module concatenation
- **Code Splitting**: Maintaining export information across chunk boundaries
- **Tree Shaking**: Copying usage information for optimization decisions

### 3. **Error Recovery**
- **Incremental Compilation**: Restoring metadata after failed rebuilds
- **Hot Module Replacement**: Preserving module state during updates
- **Build Rollback**: Reverting to previous module states

### 4. **Plugin Development**
- **Export Analysis**: Plugins that analyze and modify export information
- **Module Federation**: Sharing modules across application boundaries
- **Custom Transformations**: Plugins that create new modules based on existing ones

## Performance Considerations

1. **Use Prefetched Analysis**: Avoid repeated module graph traversals
2. **Batch Operations**: Process multiple modules in single hook invocation
3. **Separate Scopes**: Avoid borrow checker conflicts with scope separation
4. **Change Detection**: Only propagate changes when necessary
5. **Runtime Specificity**: Consider per-runtime vs global usage tracking

This comprehensive guide provides the foundation for implementing robust module metadata copying in rspack plugins and extensions.