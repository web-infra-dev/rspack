# Export Info Dependency Analysis

## Overview

The `ExportInfoDependency` is a critical component in rspack's export tracking system that provides runtime access to export metadata during code generation. It acts as a bridge between the analysis phase (where export usage information is collected) and the runtime phase (where this information is used for optimizations).

## Core Functionality

### Purpose
- **Runtime Export Information Access**: Provides a way to inject export metadata into the generated code at runtime
- **Tree-shaking Support**: Enables conditional code inclusion/exclusion based on export usage
- **Export State Querying**: Allows runtime queries about export states (used, unused, mangled, inlined)

### Key Components

#### 1. ExportInfoDependency Structure
```rust
pub struct ExportInfoDependency {
  start: u32,           // Start position in source code
  end: u32,             // End position in source code  
  export_name: Vec<Atom>, // Nested export path (e.g., ["default", "foo"])
  property: Atom,       // Property being queried ("used", "canMangle", etc.)
}
```

#### 2. Property Types
The dependency supports several property queries:

- **`usedExports`**: Returns array of used export names or boolean for namespace usage
- **`canMangle`**: Whether the export name can be mangled for minification
- **`inlinable`**: Whether the export can be inlined
- **`used`**: Boolean indicating if export is used
- **`useInfo`**: Detailed usage state (Used, Unused, OnlyPropertiesUsed, NoInfo, Unknown)
- **`provideInfo`**: Whether export is provided (Provided, NotProvided, Unknown)

## Implementation Details

### Template Rendering Process

#### 1. Export Name Resolution
```rust
fn get_property(&self, context: &TemplateContext) -> Option<String> {
    let export_name = &self.export_name;
    let prop = &self.property;
    let module_graph = compilation.get_module_graph();
    let module_identifier = module.identifier();
    
    // Special handling for usedExports query
    if export_name.is_empty() && prop == "usedExports" {
        let exports_info = module_graph.get_prefetched_exports_info(
            &module_identifier, 
            PrefetchExportsInfoMode::AllExports
        );
        let used_exports = exports_info.get_used_exports(*runtime);
        // Return serialized used exports data
    }
}
```

#### 2. Export Information Retrieval
The system uses prefetched export information for performance:

```rust
let exports_info = module_graph.get_prefetched_exports_info(
    &module_identifier,
    PrefetchExportsInfoMode::NamedNestedExports(export_name),
);
```

#### 3. Property-Specific Logic
Each property type has specialized handling:

**Can Mangle Query:**
```rust
"canMangle" => {
    let can_mangle = if let Some(export_info) = 
        exports_info.get_read_only_export_info_recursive(export_name) {
        ExportInfoGetter::can_mangle(export_info)
    } else {
        ExportInfoGetter::can_mangle(exports_info.other_exports_info())
    };
    can_mangle.map(|v| v.to_string())
}
```

**Usage State Query:**
```rust
"useInfo" => {
    let used_state = ExportsInfoGetter::get_used(&exports_info, export_name, *runtime);
    Some((match used_state {
        UsageState::Used => "true",
        UsageState::OnlyPropertiesUsed => "true", 
        UsageState::Unused => "false",
        UsageState::NoInfo => "undefined",
        UsageState::Unknown => "null",
    }).to_owned())
}
```

## Integration with Tree-Shaking

### Export Usage Tracking
The dependency integrates with rspack's export analysis plugins:

1. **FlagDependencyExportsPlugin**: Determines what exports are provided
2. **FlagDependencyUsagePlugin**: Tracks which exports are actually used
3. **ExportInfoDependency**: Provides runtime access to this information

### Runtime Conditional Logic
Generated code can use export information for conditional execution:

```javascript
// Example generated code
if (__webpack_exports_info__.canMangle) {
    // Use mangled export name
} else {
    // Use original export name
}

// Usage-based inclusion
if (__webpack_exports_info__.used) {
    // Include export-related code
}
```

## Usage Patterns

### 1. Basic Export Usage Query
```javascript
// Source code with export info dependency
const isUsed = __webpack_exports_info__.used;
if (isUsed) {
    // Conditionally include code
}
```

### 2. Nested Export Access
```javascript
// Query nested export property
const canMangleNested = __webpack_exports_info__.nested.property.canMangle;
```

### 3. All Used Exports
```javascript
// Get array of all used exports
const usedExports = __webpack_exports_info__.usedExports;
```

## Performance Considerations

### 1. Prefetching Strategy
The system uses prefetching to avoid repeated module graph queries:
- `PrefetchExportsInfoMode::AllExports` for general queries
- `PrefetchExportsInfoMode::NamedNestedExports` for specific export paths

### 2. Caching
Export information is cached in the compilation's module graph to avoid recomputation.

### 3. Runtime Efficiency
Properties are resolved at build time and injected as literals, avoiding runtime computation.

## Integration Points

### 1. Module Graph
- Uses `ModuleGraph::get_prefetched_exports_info()` for export data
- Integrates with `ExportsInfoGetter` for standardized access patterns

### 2. Template System
- Implements `DependencyTemplate` for code generation
- Uses `TemplateReplaceSource` for code replacement

### 3. Export Analysis Pipeline
- Consumes data from flag dependency plugins
- Provides bridge between analysis and runtime phases

## Error Handling

### 1. Missing Export Information
Returns `"undefined"` when export information is not available.

### 2. Invalid Property Access
Gracefully handles unknown property names by returning `None`.

### 3. Runtime Safety
All generated property accesses are safe and won't throw runtime errors.

## Export Analysis API Guidelines

Based on ShareUsagePlugin investigation findings, here are key guidelines for proper export analysis API usage:

### Correct API Usage Patterns

#### ExportsInfoGetter vs ExportInfoGetter
```rust
// Use ExportsInfoGetter::prefetch() for efficient bulk operations
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
);

// Use ExportInfoGetter::get_used() for individual export usage checking
let export_info_data = prefetched.get_read_only_export_info(&export_atom);
let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);

// NOT: ExportsInfoGetter::get_used() - this is incorrect API usage
```

#### Advanced Dependency Analysis for ConsumeShared Modules
```rust
// Use incoming connections to analyze ConsumeShared module usage
for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Extract specific export names using get_referenced_exports()
        let referenced_exports = dependency.get_referenced_exports(
            module_graph,
            &rspack_core::ModuleGraphCacheArtifact::default(),
            None,
        );
        
        // Process ExtendedReferencedExport patterns
        for export_ref in referenced_exports {
            match export_ref {
                ExtendedReferencedExport::Array(names) => {
                    // Handle multiple specific exports
                    for name in names {
                        let export_name = name.to_string();
                        // Process specific export usage
                    }
                },
                ExtendedReferencedExport::Export(export_info) => {
                    // Handle namespace or complex export patterns
                    if export_info.name.is_empty() {
                        // Namespace usage detected
                    } else {
                        // Specific named exports
                        for name in export_info.name {
                            // Process named export
                        }
                    }
                },
            }
        }
    }
}
```

#### Prefetch Mode Selection
```rust
// For comprehensive analysis (recommended for plugins)
PrefetchExportsInfoMode::AllExports

// For specific export analysis
PrefetchExportsInfoMode::NamedNestedExports(&export_names)

// For minimal analysis (performance-critical scenarios)
PrefetchExportsInfoMode::Default
```

### ProvidedExports Handling Best Practices

```rust
// Proper handling of all ProvidedExports variants
match provided_exports {
    ProvidedExports::ProvidedNames(names) => {
        // Iterate over provided exports - this is the correct approach
        for name in names {
            let export_atom = rspack_util::atom::Atom::from(name.as_str());
            // Process each specific export
        }
    },
    ProvidedExports::ProvidedAll => {
        // Module provides all exports dynamically
        // Handle with wildcard or comprehensive analysis
    },
    ProvidedExports::Unknown => {
        // Cannot determine exports statically
        // Preserve unknown status, don't assume empty
    }
}
```

### ConsumeShared Module Special Considerations

When dealing with ConsumeShared modules in export analysis:

```rust
// ConsumeShared modules are proxy modules - empty usage is expected
if module.module_type() == &ModuleType::ConsumeShared {
    // Real usage data comes from:
    // 1. Incoming dependencies (modules that import from ConsumeShared)
    // 2. Fallback module analysis
    // 3. Module connection analysis using get_referenced_exports()
    
    // Enhanced analysis using incoming connections
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;
    
    for connection in module_graph.get_incoming_connections(module_id) {
        if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
            let referenced_exports = dependency.get_referenced_exports(
                module_graph,
                &rspack_core::ModuleGraphCacheArtifact::default(),
                None,
            );
            
            for export_ref in referenced_exports {
                match export_ref {
                    ExtendedReferencedExport::Array(names) => {
                        for name in names {
                            used_exports.push(name.to_string());
                        }
                    },
                    ExtendedReferencedExport::Export(export_info) => {
                        if export_info.name.is_empty() {
                            uses_namespace = true;
                        } else {
                            for name in export_info.name {
                                used_exports.push(name.to_string());
                            }
                        }
                    },
                }
            }
        }
    }
    
    // Don't expect direct usage data from the proxy module itself
}
```

## Adding Exports and Setting Referenced Exports

### Creating and Adding Exports

#### 1. Via Dependency's `get_exports` Method

The primary way to declare exports is through the dependency's `get_exports` method:

```rust
fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
    Some(ExportsSpec {
        exports: ExportsOfExportsSpec::Names(vec![
            ExportNameOrSpec::ExportSpec(ExportSpec {
                name: "myExport".into(),
                can_mangle: Some(false),
                terminal_binding: Some(true),
                priority: Some(1),
                inlinable: Some(EvaluatedInlinableValue::String("inline_value".into())),
                ..Default::default()
            })
        ]),
        priority: Some(1),
        terminal_binding: Some(true),
        ..Default::default()
    })
}
```

#### 2. ExportsSpec Structure

```rust
pub struct ExportsSpec {
    pub exports: ExportsOfExportsSpec,
    pub priority: Option<u8>,
    pub can_mangle: Option<bool>,
    pub terminal_binding: Option<bool>,
    pub from: Option<ModuleGraphConnection>,  // For re-exports
    pub dependencies: Option<Vec<ModuleIdentifier>>,
    pub hide_export: Option<FxHashSet<Atom>>,
    pub exclude_exports: Option<FxHashSet<Atom>>,
}

pub enum ExportsOfExportsSpec {
    UnknownExports,               // For dynamic exports
    NoExports,                    // Module has no exports
    Names(Vec<ExportNameOrSpec>), // Specific named exports
}
```

#### 3. Programmatically Adding Exports at Runtime

```rust
// Access module's exports info
let exports_info = module_graph.get_exports_info(&module_id);

// Add a new export
let export_info = exports_info.get_export_info(&mut module_graph, &"newExport".into());
let export_data = export_info.as_data_mut(&mut module_graph);

// Configure the export
export_data.set_provided(Some(ExportProvided::Provided));
export_data.set_can_mangle_provide(Some(true));
export_data.set_terminal_binding(true);

// Set target if it's a re-export
ExportInfoSetter::set_target(
    export_data,
    Some(dependency_id),
    Some(target_module_id),
    Some(&Nullable::Value(vec!["targetExport".into()])),
    Some(1), // priority
);
```

### Setting Referenced Exports

#### 1. Via Dependency's `get_referenced_exports` Method

Dependencies specify which exports they reference:

```rust
fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _runtime: Option<&RuntimeSpec>,
) -> Vec<ExtendedReferencedExport> {
    vec![
        ExtendedReferencedExport::Array(vec!["specific".into(), "export".into()]), // obj.specific.export
        ExtendedReferencedExport::Export(ReferencedExport::new(
            vec!["another".into()],
            false, // can't mangle
            true,  // can inline
        )),
    ]
}
```

#### 2. Referenced Export Types

```rust
pub enum ExtendedReferencedExport {
    Array(Vec<Atom>),      // Path to specific export
    Export(ReferencedExport), // Export with additional metadata
}

pub struct ReferencedExport {
    pub name: Vec<Atom>,   // Export path
    pub can_mangle: bool,
    pub can_inline: bool,
}

// Helper functions
pub fn create_no_exports_referenced() -> Vec<ExtendedReferencedExport> {
    vec![]
}

pub fn create_exports_object_referenced() -> Vec<ExtendedReferencedExport> {
    vec![ExtendedReferencedExport::Array(vec![])]  // Empty array = entire exports object
}
```

#### 3. Marking Exports as Used

```rust
// Mark specific export as used
let export_info = exports_info.get_export_info(&mut module_graph, &"exportName".into());
ExportInfoSetter::set_used(
    export_info.as_data_mut(&mut module_graph),
    UsageState::Used,
    Some(&runtime),
);

// Mark entire exports object as used
exports_info.set_used_in_unknown_way(&mut module_graph, Some(&runtime));
```

### Integration Timing

1. **Export Declaration**: During dependency parsing via `get_exports()`
2. **Export Processing**: In `FlagDependencyExportsPlugin::finish_modules()`
3. **Usage Analysis**: During dependency analysis via `get_referenced_exports()`
4. **Usage Processing**: In `FlagDependencyUsagePlugin::optimize_dependencies()`

## Conclusion

The `ExportInfoDependency` is a sophisticated system that enables rspack to provide runtime access to compile-time export analysis results. It's essential for advanced tree-shaking scenarios where conditional code inclusion depends on export usage patterns. The system is designed for performance with prefetching and caching strategies while maintaining type safety and error resilience.

**Key Takeaways from ShareUsagePlugin Investigation:**
- Use the correct API patterns for export analysis (ExportsInfoGetter vs ExportInfoGetter)
- Handle ProvidedExports variants properly in pattern matching
- Understand ConsumeShared proxy module behavior (empty usage is expected)
- Use appropriate prefetch modes for different analysis scenarios
- Focus on dependency analysis for accurate ConsumeShared usage tracking
- **Latest Enhancement**: Use module_graph.get_incoming_connections() and dependency.get_referenced_exports() for advanced dependency analysis
- **Pattern Matching**: Handle ExtendedReferencedExport::Array and ExtendedReferencedExport::Export patterns for comprehensive export extraction
- **Cross-reference Analysis**: Compare extracted usage with provided exports for accurate filtering and optimization decisions