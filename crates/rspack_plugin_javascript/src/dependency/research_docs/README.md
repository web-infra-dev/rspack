# Rspack Export Usage Tracking Research Documentation

## Overview

This collection of research documents provides a comprehensive analysis of rspack's export usage tracking and tree-shaking system. The documentation covers the entire pipeline from dependency creation through export optimization, offering detailed insights into one of the most sophisticated tree-shaking implementations in modern JavaScript bundlers.

## Document Structure

### [01_export_info_dependency_analysis.md](./01_export_info_dependency_analysis.md)
**Focus**: Runtime export information access system

**Key Topics**:
- `ExportInfoDependency` structure and functionality
- Runtime property queries (`used`, `canMangle`, `inlinable`, etc.)
- Template rendering and code generation
- Integration with tree-shaking decisions
- Performance optimizations and caching strategies

**Best For**: Understanding how rspack provides runtime access to compile-time export analysis results.

### [02_esm_export_tracking.md](./02_esm_export_tracking.md)
**Focus**: ECMAScript Module export dependency system

**Key Topics**:
- `ESMExportSpecifierDependency` for direct exports
- `ESMExportImportedSpecifierDependency` for re-exports with complex mode system
- `ESMImportSpecifierDependency` for import tracking
- Star re-export handling and conflict resolution
- Module federation integration with ConsumeShared modules
- Init fragment system for code generation

**Best For**: Understanding how ESM exports and re-exports are tracked and optimized.

### [03_commonjs_export_analysis.md](./03_commonjs_export_analysis.md)
**Focus**: CommonJS module export dependency system

**Key Topics**:
- `CommonJsExportsDependency` structure and base types
- Support for `exports`, `module.exports`, and `this` patterns
- `Object.defineProperty` handling
- Runtime requirement management
- Export usage analysis integration
- Template rendering for various CommonJS patterns

**Best For**: Understanding how CommonJS exports are handled while maintaining Node.js compatibility.

### [04_flag_dependency_plugins_analysis.md](./04_flag_dependency_plugins_analysis.md)
**Focus**: Core analysis plugins that drive the entire system

**Key Topics**:
- `FlagDependencyExportsPlugin` - export provision analysis
- `FlagDependencyUsagePlugin` - export usage tracking
- Two-phase analysis system architecture
- Queue-based processing algorithms
- Export specification merging and target relationship tracking
- Performance optimizations and incremental processing

**Best For**: Understanding the foundational plugins that collect and analyze export metadata.

### [05_integration_and_workflow.md](./05_integration_and_workflow.md)
**Focus**: Complete system integration and end-to-end workflow

**Key Topics**:
- Four-phase compilation process
- Component interaction and data flow
- Advanced integration scenarios (module federation, nested exports, dynamic patterns)
- Performance optimization strategies
- Debugging and diagnostic capabilities
- Real-world usage patterns and edge cases

**Best For**: Understanding how all components work together and the complete optimization pipeline.

### [06_comprehensive_tree_shaking_analysis.md](./06_comprehensive_tree_shaking_analysis.md)
**Focus**: Comprehensive overview of Rspack's tree-shaking system with fact-check results

**Key Topics**:
- Complete tree-shaking architecture analysis
- Core plugin locations and implementations
- Module format support (ESM, CommonJS, mixed)
- Advanced features (Module Federation, performance optimizations)
- Research documentation quality assessment
- Integration with broader ecosystem

**Best For**: Complete understanding of tree-shaking implementation and verification of documentation accuracy.

### [07_module_federation_export_patterns.md](./07_module_federation_export_patterns.md)
**Focus**: Advanced module federation patterns and proxy module implementation

**Key Topics**:
- ConsumeShared proxy module architecture
- Two-phase metadata copying (build meta + export info)
- Plugin hook integration patterns and borrow checker solutions
- Fallback module detection and context handling
- Performance considerations for bulk export analysis
- Integration with tree-shaking system
- **ShareUsagePlugin investigation findings and export analysis API patterns**
- **Correct API usage for export analysis (ExportsInfoGetter vs ExportInfoGetter)**
- **ConsumeShared module behavior analysis and proxy pattern insights**
- **Advanced dependency analysis using get_referenced_exports() for incoming connections**
- **ExtendedReferencedExport pattern handling for specific export extraction**

**Best For**: Understanding module federation implementation details, advanced export copying patterns, proper export analysis API usage, and dependency analysis techniques for plugin development.

### [08_module_metadata_copying_patterns.md](./08_module_metadata_copying_patterns.md)
**Focus**: Comprehensive guide to module information copying patterns and best practices

**Key Topics**:
- Core metadata types (build meta, export info, usage states)
- Established patterns (proxy modules, error recovery, export merging, template initialization)
- Plugin hook integration and borrow checker solutions
- Efficient export information access using prefetched analysis
- Usage state management and bulk operations
- Performance considerations and error handling

**Best For**: Plugin developers implementing module metadata copying, understanding rspack's module information management patterns, and building robust proxy modules or transformation plugins.

### [09_plugin_development_patterns.md](./09_plugin_development_patterns.md)
**Focus**: Comprehensive patterns and best practices for developing rspack plugins

**Key Topics**:
- Plugin structure with `#[plugin]` and `#[plugin_hook]` macros
- Compilation hooks and their proper usage (`CompilerEmit`, `CompilationFinishModules`, etc.)
- Module graph manipulation and borrow checker solutions
- Export analysis API usage patterns (`ExportsInfoGetter` vs `ExportInfoGetter`)
- ConsumeShared module analysis and proxy behavior understanding
- Asset generation patterns for reports and diagnostics
- Error handling and performance optimization strategies
- Integration testing patterns and common pitfalls to avoid
- **ShareUsagePlugin implementation learnings and API corrections**
- **Advanced dependency analysis using module_graph.get_incoming_connections()**
- **get_referenced_exports() usage for extracting specific export names**
- **ExtendedReferencedExport pattern matching and processing**

**Best For**: Plugin developers working with export analysis, module graph manipulation, compilation hooks, and advanced dependency analysis. Essential for understanding correct API usage patterns and avoiding common development pitfalls.

## System Architecture Summary

```
┌─────────────────────────────────────────────────────────────┐
│                    Complete Export Analysis System          │
├─────────────────────────────────────────────────────────────┤
│  Phase 1: Dependency Creation                               │
│  ├── Parse modules and create typed dependencies            │
│  ├── ESM: ExportSpecifier, ExportImportedSpecifier         │
│  ├── CJS: CommonJsExports                                  │
│  └── Runtime: ExportInfo                                   │
│                                                             │
│  Phase 2: Export Provision (FlagDependencyExportsPlugin)    │
│  ├── Collect ExportsSpec from all dependencies             │
│  ├── Populate ExportsInfo with provision metadata          │
│  ├── Handle re-export targets and nested structures        │
│  └── Queue-based processing with change propagation        │
│                                                             │
│  Phase 3: Usage Analysis (FlagDependencyUsagePlugin)        │
│  ├── Start from entry points and traverse dependencies     │
│  ├── Collect referenced exports from module dependencies   │
│  ├── Apply usage states (Used, Unused, OnlyPropertiesUsed) │
│  └── Handle side effects and transitive dependencies       │
│                                                             │
│  Phase 4: Code Generation & Optimization                    │
│  ├── Query ExportsInfo during template rendering           │
│  ├── Generate optimized export code                        │
│  ├── Apply tree-shaking decisions                          │
│  └── Handle module federation and advanced scenarios       │
└─────────────────────────────────────────────────────────────┘
```

## Key Concepts

### Export Information Flow
1. **Dependencies** describe what they provide via `get_exports()` → `ExportsSpec`
2. **FlagDependencyExportsPlugin** collects specs and populates `ExportsInfo`
3. **FlagDependencyUsagePlugin** tracks usage and updates `ExportsInfo`
4. **Template rendering** queries `ExportsInfo` for optimization decisions

### Advanced Features
- **Module Federation**: Special handling for ConsumeShared modules with conditional exports
- **Nested Exports**: Support for deep object property tracking
- **Dynamic Exports**: Handling of `require.context()` and other dynamic patterns
- **Re-export Chains**: Complex target relationship tracking
- **Performance**: Incremental processing, caching, and change propagation

### Data Structures
- **ExportsInfo**: Central repository of export metadata per module
- **ExportInfo**: Individual export tracking with usage states and capabilities
- **ExportsSpec**: Dependency-provided export specifications
- **UsageState**: Fine-grained usage classification (Used, Unused, OnlyPropertiesUsed, etc.)

## Usage Recommendations

### For Understanding Tree-Shaking
Start with **[04_flag_dependency_plugins_analysis.md](./04_flag_dependency_plugins_analysis.md)** to understand the core analysis system, then read **[05_integration_and_workflow.md](./05_integration_and_workflow.md)** for the complete picture.

### For Implementing Export Dependencies
Begin with **[02_esm_export_tracking.md](./02_esm_export_tracking.md)** or **[03_commonjs_export_analysis.md](./03_commonjs_export_analysis.md)** depending on the module system, then reference **[01_export_info_dependency_analysis.md](./01_export_info_dependency_analysis.md)** for runtime integration.

### For Export Analysis Plugin Development
Start with **[09_plugin_development_patterns.md](./09_plugin_development_patterns.md)** for comprehensive plugin development patterns and ShareUsagePlugin learnings, including advanced dependency analysis using get_referenced_exports(). Read **[07_module_federation_export_patterns.md](./07_module_federation_export_patterns.md)** for specific ShareUsagePlugin investigation findings and ConsumeShared module behavior insights. Also review **[01_export_info_dependency_analysis.md](./01_export_info_dependency_analysis.md)** and **[06_comprehensive_tree_shaking_analysis.md](./06_comprehensive_tree_shaking_analysis.md)** for export analysis API guidelines.

### For Performance Optimization
Focus on the performance sections in **[04_flag_dependency_plugins_analysis.md](./04_flag_dependency_plugins_analysis.md)** and **[05_integration_and_workflow.md](./05_integration_and_workflow.md)**.

### For Module Federation
Review the ConsumeShared sections in **[02_esm_export_tracking.md](./02_esm_export_tracking.md)** and **[03_commonjs_export_analysis.md](./03_commonjs_export_analysis.md)**, then read **[07_module_federation_export_patterns.md](./07_module_federation_export_patterns.md)** for comprehensive module federation insights and ShareUsagePlugin findings.

## Research Methodology

This documentation was created through:

1. **Static Code Analysis**: Comprehensive examination of dependency implementations
2. **Plugin Architecture Review**: Analysis of flag dependency plugin systems
3. **Integration Pattern Study**: Understanding component interactions and data flow
4. **Performance Analysis**: Investigation of optimization strategies and caching
5. **Edge Case Documentation**: Exploration of complex scenarios and error handling

The documentation provides both high-level architectural understanding and implementation-level details suitable for contributors, plugin developers, and bundler researchers.

## Contributing

When updating this documentation:

1. Maintain consistency with the existing structure and terminology
2. Include code examples from actual rspack implementations
3. Document both common use cases and edge cases
4. Update the integration workflow when adding new components
5. Keep performance implications in focus for all changes

This research documentation serves as both a learning resource and a foundation for future enhancements to rspack's tree-shaking capabilities.