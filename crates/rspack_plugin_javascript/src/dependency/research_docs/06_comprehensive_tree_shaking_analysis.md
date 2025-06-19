# Comprehensive Tree Shaking Analysis in Rspack

## Overview

This document provides a comprehensive analysis of Rspack's tree-shaking implementation, consolidating findings from fact-checking existing research docs and exploring the broader codebase. Rspack implements one of the most sophisticated tree-shaking systems in modern JavaScript bundlers.

## Core Tree Shaking Architecture

### Four-Phase Compilation Process

1. **Export Discovery Phase** - `FlagDependencyExportsPlugin`
2. **Usage Analysis Phase** - `FlagDependencyUsagePlugin` 
3. **Side Effect Analysis Phase** - `SideEffectsFlagPlugin`
4. **Optimization Phase** - `ModuleConcatenationPlugin` + `MangleExportsPlugin`

### Key Components and File Locations

#### Primary Tree Shaking Plugins

**FlagDependencyExportsPlugin**
- **Location**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_exports_plugin.rs`
- **Purpose**: Analyzes and flags which exports are provided by each module
- **Key Functions**:
  - `process_exports_spec()` - Processes export specifications from dependencies
  - `merge_exports()` - Merges export information from multiple sources
  - `set_unknown_exports_provided()` - Handles dynamic exports

**FlagDependencyUsagePlugin**
- **Location**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`
- **Purpose**: Tracks which exports are actually used across the dependency graph
- **Key Functions**:
  - `process_module()` - Analyzes module dependencies and usage
  - `process_referenced_module()` - Marks exports as used based on import patterns
  - `set_used_without_info()`, `set_used_in_unknown_way()` - Updates usage state

**SideEffectsFlagPlugin**
- **Location**: `crates/rspack_plugin_javascript/src/plugin/side_effects_flag_plugin.rs`
- **Purpose**: Identifies modules with side effects and optimizes import/export connections
- **Key Components**:
  - `SideEffectsFlagPluginVisitor` - AST visitor to detect side effects in code
  - `can_optimize_connection()` - Determines if connections can be optimized
  - `do_optimize_connection()` - Performs connection optimization

#### Core Data Structures

**ExportsInfo System**
- **Location**: `crates/rspack_core/src/exports/exports_info.rs`
- **Purpose**: Central data structure tracking export information for modules
- **Integration**: Links with module graph to provide comprehensive export tracking

**ExportInfo**
- **Location**: `crates/rspack_core/src/exports/export_info.rs`
- **Purpose**: Tracks individual export usage and provide information
- **Key Properties**: `provided`, `used_name`, `target`, `can_mangle_provide`, `can_mangle_use`

### Usage State System

```rust
pub enum UsageState {
    Unused = 0,                // Export is not used - can be eliminated
    OnlyPropertiesUsed = 1,    // Only properties of export are used
    NoInfo = 2,                // No usage information available
    Unknown = 3,               // Usage is unknown - assume used
    Used = 4,                  // Export is definitely used
}

pub enum ExportProvided {
    Provided,     // Export is statically confirmed
    NotProvided,  // Export is confirmed to not exist
    Unknown,      // Export status is unknown (e.g., CommonJS)
}
```

## Module Format Support

### ESM (ES Modules)

**Export Dependencies**:
- `esm_export_specifier_dependency.rs` - Handles `export { name }`
- `esm_export_imported_specifier_dependency.rs` - Handles `export { name } from 'module'`
- `esm_export_expression_dependency.rs` - Handles `export default` and expressions

**Import Dependencies**:
- `esm_import_dependency.rs` - Handles `import` statements
- `esm_import_specifier_dependency.rs` - Tracks specific import specifiers

### CommonJS

**Export Dependencies**:
- `common_js_exports_dependency.rs` - Handles `module.exports` assignments
- `common_js_export_require_dependency.rs` - Handles `module.exports = require()`

**Import Dependencies**:
- `common_js_require_dependency.rs` - Tracks `require()` calls
- `require_resolve_dependency.rs` - Handles `require.resolve()`

### Advanced Pattern Detection

**Star Re-exports**: Complex mode system for `export * from 'module'` with sophisticated namespace handling
**Dynamic Exports**: Unknown export type handling for `module.exports = dynamicValue`
**Mixed Formats**: ESM/CommonJS interop with compatibility dependencies

## Advanced Features

### Module Federation Integration

**Location**: `crates/rspack_plugin_mf/src/sharing/`

- **Export Usage Analysis**: `export_usage_analysis.rs` - Advanced usage tracking for federated modules
- **Export Usage Plugin**: `export_usage_plugin.rs` - Generates detailed usage reports
- **Usage Types**: `export_usage_types.rs` - Comprehensive data structures

### Performance Optimizations

1. **Prefetched Exports**: Bulk analysis using `PrefetchExportsInfoMode`
2. **Incremental Processing**: Queue-based algorithms for dependency processing
3. **Caching Systems**: 
   - Export info caching
   - Mode caching for complex dependencies
   - Module graph cache artifacts
4. **Change Tracking**: Only re-analyze modified modules during rebuilds

### Configuration Integration

**Optimization Options** (`crates/rspack_core/src/options/optimizations.rs`):
- `side_effects: bool` - Controls side effect analysis
- `used_exports: bool | "global"` - Enables usage analysis
- `provided_exports: bool` - Enables export discovery
- `mangle_exports: bool` - Enables export name mangling

## Tree Shaking Decision Process

### 1. Export Discovery
- Parse module AST to identify all exports
- Create `ExportInfo` entries with provision status
- Handle different export patterns (named, default, re-exports)

### 2. Usage Analysis
- Start from entry points and traverse dependency graph
- Analyze import statements to determine referenced exports
- Propagate usage information through module connections
- Handle namespace imports vs named imports differently

### 3. Side Effect Evaluation
- Detect modules with side effects using AST analysis
- Optimize connections by skipping side-effect-free modules
- Preserve modules with side effects even if exports aren't used

### 4. Dead Code Elimination
- Mark unused exports for elimination
- Generate optimized code without unused exports
- Maintain source maps and debugging information

## Integration with Broader Ecosystem

### Webpack Compatibility
- Maintains compatibility with webpack's tree-shaking semantics
- Supports similar configuration options and behavior
- Handles edge cases consistently with webpack

### Development Tools
- Provides detailed usage reports for debugging
- Supports development mode with preserved export information
- Integrates with source maps for accurate debugging

### Build Performance
- Implements incremental compilation for fast rebuilds
- Uses efficient data structures for large codebases
- Provides parallel processing capabilities

## Export Analysis Plugin Development Guidelines

Based on ShareUsagePlugin investigation findings, here are key guidelines for developing export analysis plugins:

### API Usage Best Practices

#### Export Information Access
```rust
// Correct: Use ExportsInfoGetter::prefetch() for efficient bulk operations
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports, // Comprehensive analysis
);

// Correct: Use ExportInfoGetter::get_used() for usage state checking
let export_info_data = prefetched.get_read_only_export_info(&export_atom);
let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);

// Incorrect: Don't use ExportsInfoGetter for individual export usage checking
// let usage_state = ExportsInfoGetter::get_used(&exports_info, ...); // Wrong API
```

#### Advanced Dependency Analysis (Latest Enhancement)
```rust
// Use module_graph.get_incoming_connections() to analyze how ConsumeShared modules are imported
for connection in module_graph.get_incoming_connections(consume_shared_id) {
    if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Call dependency.get_referenced_exports() to extract specific export names
        let referenced_exports = dependency.get_referenced_exports(
            module_graph,
            &rspack_core::ModuleGraphCacheArtifact::default(),
            None,
        );
        
        // Handle ExtendedReferencedExport patterns
        for export_ref in referenced_exports {
            match export_ref {
                ExtendedReferencedExport::Array(names) => {
                    // Multiple specific exports are referenced
                    for name in names {
                        let export_name = name.to_string();
                        // Process specific export usage
                    }
                },
                ExtendedReferencedExport::Export(export_info) => {
                    // Single export or namespace reference
                    if export_info.name.is_empty() {
                        // No specific name indicates namespace usage
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

#### ProvidedExports Pattern Matching
```rust
// Proper handling of all ProvidedExports variants
match provided_exports {
    ProvidedExports::ProvidedNames(names) => {
        // Iterate over specific exports, not try to enumerate all exports
        for name in names {
            // Process each specific named export
        }
    },
    ProvidedExports::ProvidedAll => {
        // Module provides all possible exports dynamically
        // Handle appropriately for bulk export scenarios
    },
    ProvidedExports::Unknown => {
        // Cannot determine exports statically
        // Preserve unknown status, don't assume empty
    }
}
```

### ConsumeShared Module Considerations

#### Expected Behavior Patterns
- **Empty Usage Arrays**: ConsumeShared modules showing empty usage is correct behavior
- **Proxy Pattern**: These modules act as proxies, real usage data is elsewhere
- **Analysis Strategy**: Focus on incoming dependencies and fallback modules

#### Proper Analysis Approach
```rust
// For ConsumeShared modules, analyze usage from consumers using get_referenced_exports()
if module.module_type() == &ModuleType::ConsumeShared {
    // 1. Enhanced analysis using incoming connections and get_referenced_exports()
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;
    
    for connection in module_graph.get_incoming_connections(module_id) {
        if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
            // Use get_referenced_exports() to extract specific export names
            let referenced_exports = dependency.get_referenced_exports(
                module_graph,
                &rspack_core::ModuleGraphCacheArtifact::default(),
                None,
            );
            
            // Process ExtendedReferencedExport patterns
            for export_ref in referenced_exports {
                match export_ref {
                    ExtendedReferencedExport::Array(names) => {
                        // Multiple specific exports referenced
                        for name in names {
                            used_exports.push(name.to_string());
                        }
                    },
                    ExtendedReferencedExport::Export(export_info) => {
                        // Single export or namespace reference
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
    
    // 2. Find and analyze fallback module
    if let Some(fallback_id) = find_fallback_module(module_graph, module_id) {
        // Use fallback module's export information
    }
    
    // 3. Cross-reference extracted usage with provided exports for accurate filtering
    let filtered_usage = cross_reference_usage_with_provided(used_exports, provided_exports);
}
```

### Plugin Implementation Patterns

#### Hook Usage for Export Analysis
```rust
#[plugin_hook(CompilerEmit for YourExportAnalysisPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
    // CompilerEmit hook provides access to final module graph
    // All export analysis and usage information is available
    
    let module_graph = compilation.get_module_graph();
    let runtimes: Vec<RuntimeSpec> = compilation
        .chunk_by_ukey
        .values()
        .map(|chunk| chunk.runtime())
        .cloned()
        .collect();
    
    // Generate analysis reports
}
```

## Research Documentation Quality Assessment

Based on comprehensive fact-checking and ShareUsagePlugin investigation, the existing research documentation in `research_docs/` is **highly accurate and comprehensive**:

- ✅ **Technical Accuracy**: All core concepts, data structures, and implementation details are correct
- ✅ **Code Examples**: Rust code examples follow proper conventions and show realistic patterns
- ✅ **Architecture Coverage**: Complete coverage of the tree-shaking pipeline
- ✅ **Performance Awareness**: Consistently addresses optimization strategies
- ✅ **Edge Case Handling**: Documents complex scenarios like module federation and dynamic exports
- ✅ **API Usage Patterns**: ShareUsagePlugin investigation confirmed correct API usage throughout
- ✅ **Latest Enhancements**: Advanced dependency analysis using get_referenced_exports() documented and validated
- ✅ **Pattern Matching**: ExtendedReferencedExport handling patterns correctly documented
- ✅ **ConsumeShared Analysis**: Proxy module behavior and incoming connection analysis properly covered

The documentation represents one of the most thorough analyses of a modern bundler's tree-shaking implementation available.

## Future Enhancements

### Potential Improvements
1. **Metrics Collection**: Add performance benchmarks and detailed timing data
2. **Advanced Diagnostics**: Enhanced debugging tools for complex usage patterns
3. **Optimization Heuristics**: Machine learning-based optimization suggestions
4. **Cross-Module Analysis**: Even more sophisticated inter-module optimization

### Areas for Research
1. **Dynamic Import Patterns**: Enhanced analysis of dynamic import usage
2. **Web Workers**: Tree-shaking optimization for worker contexts
3. **Micro-frontend Architecture**: Advanced federation scenarios
4. **Bundle Splitting**: Integration with code splitting strategies

This comprehensive analysis demonstrates that Rspack's tree-shaking implementation is among the most advanced in the JavaScript ecosystem, providing sophisticated optimization capabilities while maintaining compatibility and performance.