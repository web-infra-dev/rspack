# ESM Export Tracking and Dependencies

## Overview

Rspack implements comprehensive ESM (ECMAScript Module) export tracking through a sophisticated dependency system that handles both direct exports and re-exports. This system enables precise tree-shaking and export optimization by tracking how exports are defined, used, and transformed throughout the build process.

## Core ESM Export Dependencies

### 1. ESMExportSpecifierDependency

**Purpose**: Handles direct named exports within a module
```rust
pub struct ESMExportSpecifierDependency {
    name: Atom,        // Export name (e.g., "foo" in `export { foo }`)
    value: Atom,       // Local variable/value being exported
    inline: Option<EvaluatedInlinableValue>, // Inlining information
    range: DependencyRange, // Source code position
}
```

**Code Generation Process**:
```rust
impl DependencyTemplate for ESMExportSpecifierDependencyTemplate {
    fn render(&self, dep: &dyn DependencyCodeGeneration, context: &mut TemplateContext) {
        // 1. Get used name after mangling/optimization
        let used_name = ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            *runtime,
            std::slice::from_ref(&dep.name),
        );
        
        // 2. Generate export definition
        if let Some(UsedName::Normal(used)) = used_name {
            init_fragments.push(Box::new(ESMExportInitFragment::new(
                module.get_exports_argument(),
                vec![(used, dep.value.to_string().into())],
            )));
        }
    }
}
```

**Module Federation Integration**:
The system includes special handling for ConsumeShared modules:
```rust
// Check if this dependency is related to a ConsumeShared module
let consume_shared_info = {
    if let Some(parent_module_id) = module_graph.get_parent_module(&dep.id) {
        if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
            if parent_module.module_type() == &ModuleType::ConsumeShared {
                parent_module.get_consume_shared_key()
            }
        }
    }
};

// Generate conditional exports for tree-shaking
let export_content = if let Some(ref share_key) = consume_shared_info {
    format!(
        "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
        share_key, dep.name, dep.value
    )
} else {
    dep.value.to_string()
};
```

### 2. ESMExportImportedSpecifierDependency

**Purpose**: Handles re-exports from other modules (`export { foo } from './module'`)

**Complex Mode System**:
The dependency implements a sophisticated mode system to handle different re-export scenarios:

```rust
pub enum ExportMode {
    Missing,                           // Target module not found
    Unused(ExportModeUnused),         // Export is not used
    EmptyStar(ExportModeEmptyStar),   // Star re-export with no exports
    ReexportDynamicDefault(ExportModeReexportDynamicDefault), // Dynamic default re-export
    ReexportNamedDefault(ExportModeReexportNamedDefault),     // Named default re-export
    ReexportNamespaceObject(ExportModeReexportNamespaceObject), // Namespace re-export
    ReexportFakeNamespaceObject(ExportModeFakeNamespaceObject), // Fake namespace
    ReexportUndefined(ExportModeReexportUndefined),           // Undefined re-export
    NormalReexport(ExportModeNormalReexport),                 // Normal re-export
    DynamicReexport(ExportModeDynamicReexport),               // Dynamic re-export
}
```

**Mode Determination Logic**:
```rust
fn get_mode(&self, module_graph: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> ExportMode {
    // 1. Check if export is unused
    let is_name_unused = if let Some(ref name) = name {
        ExportsInfoGetter::get_used(&exports_info_data, std::slice::from_ref(name), runtime)
            == UsageState::Unused
    };
    if is_name_unused {
        return ExportMode::Unused(ExportModeUnused { name: "*".into() });
    }
    
    // 2. Handle special default export cases
    if let Some(name) = name.as_ref() && ids.first() == Some("default") {
        match imported_exports_type {
            ExportsType::Dynamic => {
                return ExportMode::ReexportDynamicDefault(ExportModeReexportDynamicDefault {
                    name: name.clone(),
                });
            }
            ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
                return ExportMode::ReexportNamedDefault(ExportModeReexportNamedDefault {
                    name: name.clone(),
                    partial_namespace_export_info: exports_info_data.get_read_only_export_info(name).id(),
                });
            }
        }
    }
    
    // 3. Determine star re-export behavior
    let StarReexportsInfo { exports, checked, ignored_exports, hidden } = 
        self.get_star_reexports(module_graph, runtime, Some(exports_info), imported_module_identifier);
    
    // 4. Return appropriate mode based on analysis
}
```

### 3. ESMImportSpecifierDependency

**Purpose**: Handles import statements that may be used in exports

**Referenced Exports Analysis**:
```rust
fn get_referenced_exports(&self, module_graph: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> Vec<ExtendedReferencedExport> {
    let mut ids = self.get_ids(module_graph);
    
    // Handle namespace imports
    if ids.is_empty() {
        return self.get_referenced_exports_in_destructuring(None);
    }
    
    // Handle default export special cases
    if let Some(id) = ids.first() && id == "default" {
        let exports_type = get_exports_type(module_graph, &self.id, parent_module);
        match exports_type {
            ExportsType::DefaultOnly | ExportsType::DefaultWithNamed => {
                if ids.len() == 1 {
                    return self.get_referenced_exports_in_destructuring(None);
                }
                ids = &ids[1..];
                namespace_object_as_context = true;
            }
            ExportsType::Dynamic => {
                return create_exports_object_referenced();
            }
        }
    }
    
    // Handle property access and destructuring
    if self.call && !self.direct_import && (namespace_object_as_context || ids.len() > 1) {
        if ids.len() == 1 {
            return create_exports_object_referenced();
        }
        ids = &ids[..ids.len() - 1];
    }
    
    self.get_referenced_exports_in_destructuring(Some(ids))
}
```

## Export Fragment Generation

### Init Fragment System
ESM exports use an init fragment system for code generation:

```rust
pub struct ESMExportInitFragment {
    exports_argument: String,        // __webpack_exports__
    export_map: Vec<(Atom, Atom)>, // [(export_name, export_value)]
}

impl InitFragment for ESMExportInitFragment {
    fn generate(&self, context: &mut GenerateContext) -> String {
        let exports = self.export_map
            .iter()
            .map(|(name, value)| format!("{}: {}", property_name(name), value))
            .collect::<Vec<_>>()
            .join(", ");
            
        format!(
            "/* ESM exports */ {}({}, {{ {} }});\n",
            RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
            self.exports_argument,
            exports
        )
    }
}
```

### Conditional Export Generation
For module federation and tree-shaking:

```rust
fn get_reexport_fragment(&self, ctxt: &mut TemplateContext, comment: &str, key: String, name: &str, value_key: ValueKey) -> ESMExportInitFragment {
    // Check for ConsumeShared module context
    let consume_shared_info = self.get_consume_shared_context(module_graph);
    
    // Generate conditional export content
    let export_content = if let Some(ref share_key) = consume_shared_info {
        format!(
            "/* @common:if [condition=\"treeShake.{}.{}\"] */ /* {comment} */ {return_value} /* @common:endif */",
            share_key, key
        )
    } else {
        format!(
            "/* EXPORT_BEGIN:{} */ /* {comment} */ {return_value} /* EXPORT_END:{} */",
            key, key
        )
    };
    
    ESMExportInitFragment::new(module.get_exports_argument(), vec![(key.into(), export_content.into())])
}
```

## Star Re-export Handling

### Star Re-export Information Collection
```rust
pub struct StarReexportsInfo {
    exports: Option<HashSet<Atom>>,    // Available exports
    checked: Option<HashSet<Atom>>,    // Exports that need runtime checks
    ignored_exports: HashSet<Atom>,    // Exports to ignore
    hidden: Option<HashSet<Atom>>,     // Hidden exports
}

fn get_star_reexports(&self, module_graph: &ModuleGraph, runtime: Option<&RuntimeSpec>) -> StarReexportsInfo {
    let exports_info = module_graph.get_exports_info(parent_module).as_data(module_graph);
    let imported_exports_info = module_graph.get_exports_info(imported_module_identifier).as_data(module_graph);
    
    // Check if we can determine exports statically
    let no_extra_exports = matches!(
        imported_exports_info.other_exports_info().as_data(module_graph).provided(),
        Some(ExportProvided::NotProvided)
    );
    let no_extra_imports = matches!(
        ExportInfoGetter::get_used(exports_info.other_exports_info().as_data(module_graph), runtime),
        UsageState::Unused
    );
    
    if !no_extra_exports && !no_extra_imports {
        return StarReexportsInfo {
            ignored_exports: self.active_exports(module_graph).clone(),
            hidden: self.discover_active_exports_from_other_star_exports(module_graph),
            ..Default::default()
        };
    }
    
    // Collect static export information
    let mut exports = HashSet::default();
    let mut checked = HashSet::default();
    
    // Process each export from the imported module
    for imported_export_info in imported_exports_info.exports() {
        let imported_export_info_data = imported_export_info.as_data(module_graph);
        let imported_export_info_name = imported_export_info_data.name().cloned().unwrap_or_default();
        
        // Skip ignored exports
        if ignored_exports.contains(&imported_export_info_name) ||
           matches!(imported_export_info_data.provided(), Some(ExportProvided::NotProvided)) {
            continue;
        }
        
        // Check if export is used
        let export_info = exports_info.id().get_read_only_export_info(module_graph, &imported_export_info_name);
        if matches!(ExportInfoGetter::get_used(export_info.as_data(module_graph), runtime), UsageState::Unused) {
            continue;
        }
        
        exports.insert(imported_export_info_name.clone());
        
        // Mark for runtime checking if provision is uncertain
        if !matches!(imported_export_info_data.provided(), Some(ExportProvided::Provided)) {
            checked.insert(imported_export_info_name);
        }
    }
    
    StarReexportsInfo {
        ignored_exports,
        exports: Some(exports),
        checked: Some(checked),
        hidden: None,
    }
}
```

## Performance Optimizations

### 1. Export Information Prefetching
```rust
// Prefetch export information to avoid repeated queries
let exports_info = module_graph.get_prefetched_exports_info(
    &module.identifier(),
    PrefetchExportsInfoMode::NamedExports(HashSet::from_iter([&dep.name])),
);
```

### 2. Used Name Caching
```rust
// Cache used names to avoid recomputation
let used_name = ExportsInfoGetter::get_used_name(
    GetUsedNameParam::WithNames(&exports_info),
    *runtime,
    std::slice::from_ref(&dep.name),
);
```

### 3. Mode Caching
```rust
// Cache export modes for re-export dependencies
let key = (self.id, runtime.map(|runtime| get_runtime_key(runtime).to_owned()));
module_graph_cache.cached_get_mode(key, || {
    self.get_mode_inner(module_graph, module_graph_cache, runtime)
})
```

## Debug and Tracing

### Comprehensive Logging
The system includes detailed logging for module federation scenarios:

```rust
tracing::debug!(
    "[RSPACK_EXPORT_DEBUG:ESM_SPECIFIER_DETAILED] Module: {:?}, Type: {:?}, Layer: {:?}, Name: {:?}, Value: {:?}, DependencyId: {:?}",
    module_identifier,
    module.module_type(),
    module.get_layer(),
    dep.name,
    dep.value,
    dep.id()
);
```

## Integration Points

### 1. Flag Dependency Plugins
- **FlagDependencyExportsPlugin**: Populates export provision information
- **FlagDependencyUsagePlugin**: Tracks export usage patterns
- **ESM Dependencies**: Consume and act on this information

### 2. Module Graph
- Central storage for export metadata
- Provides caching and prefetching mechanisms
- Maintains relationships between modules and dependencies

### 3. Template System
- Generates optimized export code
- Handles conditional exports for tree-shaking
- Integrates with init fragment system for code organization

## Conclusion

The ESM export tracking system in rspack provides comprehensive analysis and optimization of ES module exports and re-exports. Through sophisticated dependency types, mode analysis, and conditional code generation, it enables precise tree-shaking while maintaining correctness across complex re-export scenarios. The system is designed for performance with extensive caching and prefetching, while providing detailed debugging capabilities for complex module federation use cases.