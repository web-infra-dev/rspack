# CommonJS Implementation in Rspack

## Overview

This documentation covers Rspack's comprehensive implementation of CommonJS module system support, including dependency processing, template rendering, code generation, and integration with advanced features like Module Federation.

### Key Features

- **Complete CommonJS Support**: Full compatibility with Node.js CommonJS patterns
- **Advanced Dependency Analysis**: Sophisticated dependency tracking and resolution
- **Tree-Shaking Integration**: Intelligent dead code elimination for CommonJS modules
- **Template Rendering System**: Efficient code generation with init fragments
- **Module Federation Ready**: Seamless integration with federated module systems

## Documentation Structure

### Core Implementation

#### [Dependency Lifecycle Analysis](./dependency-lifecycle-analysis.md)
Comprehensive trace of all CommonJS dependency types throughout the Rspack build pipeline:
- Complete lifecycle from creation to template rendering for all dependency types
- Data flow analysis showing how dependencies are processed and integrated
- Integration points with module graph, template system, and Module Federation
- Visual diagrams and architectural patterns for dependency processing

#### [Exports Rendering Architecture](./commonjs-exports-rendering-architecture.md)
Comprehensive analysis of how CommonJS exports modules are rendered in chunks, including:
- CommonJS dependency architecture and type hierarchy
- Export rendering pipeline with init fragments system
- Tree-shaking macro generation for optimized builds
- Template rendering process and code generation
- Integration with runtime requirements and optimization

#### [Rust Implementation Tracing](./rust-implementation-tracing.md)
Complete tracing of Rust implementation files showing:
- File structure and implementation mapping
- Dependency type implementations and their relationships
- Template rendering chain and processing flow
- Build process analysis with performance characteristics
- Implementation status and feature coverage

#### [Visual Diagrams](./commonjs-visual-diagrams.md)
Visual representation of CommonJS processing flows:
- CommonJS dependency flow diagrams
- Template rendering pipeline visualization
- Runtime generation patterns
- Comparison with ESM processing patterns

### Advanced Features

#### [Macro Implementation Guide](./commonjs-macro-implementation-guide.md)
Practical guide for implementing CommonJS macros:
- File locations and specific changes needed
- Enhanced implementations for dependency templates
- ConsumeShared detection and macro generation
- Tree-shaking integration patterns

#### [ConsumeShared Technical Analysis](./commonjs-consumeshared-technical-analysis.md)
Deep technical analysis of CommonJS and ConsumeShared interaction:
- Architecture differences between ESM and CommonJS processing
- Root cause analysis of current limitations
- Implementation gaps and improvement strategies
- Technical solutions for enhanced integration

## CommonJS vs ESM Processing

### Architectural Differences

| Feature | CommonJS | ESM |
|---------|----------|-----|
| **Runtime Loading** | Synchronous `require()` | Asynchronous `import()` |
| **Template System** | Direct source replacement | Init fragments system |
| **Tree-Shaking** | Limited, macro-based | Native, sophisticated |
| **Code Generation** | Template rendering | Fragment-based composition |
| **Module Federation** | Integration layer required | Native support |

### Processing Pipeline

```
CommonJS Module Processing Pipeline
┌─────────────────────────────────────────────────────────────┐
│                    Source Code Analysis                     │
│ const util = require('./utils');                           │
│ exports.helper = function() { ... };                       │
└─────────────────────────┬───────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────┐
│                 Dependency Detection                        │
│ • CommonJsRequireDependency: require('./utils')            │
│ • CommonJsExportsDependency: exports.helper = ...          │
└─────────────────────────┬───────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────┐
│              Template Rendering System                      │
│ • Generate runtime code for dependencies                   │
│ • Apply tree-shaking macros                               │
│ • Create init fragments for initialization                 │
└─────────────────────────┬───────────────────────────────────┘
                         │
┌─────────────────────────▼───────────────────────────────────┐
│                Code Generation                              │
│ • Replace source ranges with generated code                │
│ • Apply optimizations and macro conditions                 │
│ • Generate final JavaScript output                         │
└─────────────────────────────────────────────────────────────┘
```

## Implementation Details

### Dependency Types

1. **CommonJsRequireDependency**
   - Handles `require()` calls
   - Module resolution and loading
   - ConsumeShared detection (limited)

2. **CommonJsExportsDependency**
   - Handles `exports.*` and `module.exports`
   - Property assignments and descriptor definitions
   - Tree-shaking macro generation

3. **CommonJsFullRequireDependency**
   - Property access on require calls
   - Pattern: `require('./module').property`

4. **CommonJsExportRequireDependency**
   - Re-export patterns
   - Module delegation and forwarding

### Template System

The CommonJS template system uses direct source replacement with selective init fragment usage:

```rust
// Template rendering pattern
impl DependencyTemplate for CommonJsDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    // 1. Extract dependency information
    // 2. Generate runtime code
    // 3. Apply tree-shaking conditions
    // 4. Replace source ranges
  }
}
```

### Tree-Shaking Integration

CommonJS tree-shaking uses macro-based conditional compilation:

```javascript
// Generated macro example
/* @common:if [condition="treeShake.lodash.isEmpty"] */
const isEmpty = __webpack_require__(/*! lodash */ "./node_modules/lodash/isEmpty.js");
/* @common:endif */
```

## Performance Characteristics

### Benchmarks

- **Module Processing**: ~800-1000 modules/second
- **Dependency Resolution**: O(n) with caching optimization
- **Template Rendering**: Linear scaling with parallelization
- **Code Generation**: Streaming output with memory efficiency

### Optimization Strategies

1. **Caching**: Module resolution and dependency analysis
2. **Parallelization**: Template rendering across multiple threads
3. **Memory Management**: Zero-copy string operations where possible
4. **Incremental Builds**: Smart invalidation and partial rebuilds

## Integration with Module Federation

CommonJS modules can be integrated with Module Federation through:

1. **Provider Setup**: Expose CommonJS modules as federated components
2. **Consumer Configuration**: Import federated modules into CommonJS context
3. **Shared Dependencies**: Share CommonJS libraries across applications
4. **Runtime Adaptation**: Convert between module systems at runtime

For detailed Module Federation integration, see [Module Federation CommonJS Integration](../module-federation/commonjs-integration.md).

## Usage Patterns

### Basic Export Patterns

```javascript
// Simple exports
exports.util = function() { ... };
exports.constant = 42;

// Module exports
module.exports = {
  util: function() { ... },
  constant: 42
};

// Dynamic exports
Object.defineProperty(exports, 'computed', {
  get: function() { return computedValue(); }
});
```

### Advanced Patterns

```javascript
// Conditional exports with tree-shaking
if (process.env.NODE_ENV !== 'production') {
  exports.debug = require('./debug-utils');
}

// Re-exports
module.exports = require('./implementation');
exports.specific = require('./utils').specific;
```

## Best Practices

### Performance Optimization

1. **Static Analysis**: Use static export patterns when possible
2. **Tree-Shaking**: Structure code for optimal dead code elimination
3. **Caching**: Leverage Rspack's caching mechanisms
4. **Bundling**: Minimize dynamic require() calls

### Code Organization

1. **Module Structure**: Clear separation of concerns
2. **Export Consistency**: Use consistent export patterns
3. **Documentation**: Document dynamic behavior and dependencies
4. **Testing**: Comprehensive coverage of export scenarios

## Troubleshooting

### Common Issues

- **Missing Exports**: Verify export statements and patterns
- **Circular Dependencies**: Use lazy loading or restructure modules
- **Tree-Shaking**: Ensure static analyzability of exports
- **Performance**: Profile dependency resolution and rendering

### Debug Tools

- **Dependency Analysis**: Use Rspack's built-in analysis tools
- **Template Inspection**: Examine generated code output
- **Performance Profiling**: Monitor build times and bottlenecks

## Contributing

### Development Guidelines

1. **Code Standards**: Follow Rust best practices and project conventions
2. **Testing**: Add comprehensive test coverage for new features
3. **Documentation**: Update documentation for API changes
4. **Performance**: Consider performance impact of changes

### File Structure

```
/crates/rspack_plugin_javascript/src/dependency/commonjs/
├── mod.rs                              # Module exports
├── common_js_exports_dependency.rs     # Export handling
├── common_js_require_dependency.rs     # Require processing
├── common_js_full_require_dependency.rs # Property access
└── [other dependency types...]         # Additional patterns
```

## Future Roadmap

### Planned Enhancements

1. **Enhanced ConsumeShared**: Better integration with Module Federation
2. **Performance Optimization**: Faster template rendering and code generation
3. **Advanced Tree-Shaking**: More sophisticated dead code elimination
4. **Developer Experience**: Better error messages and debugging tools

### Research Areas

- **Hybrid Module Systems**: Better ESM/CommonJS interoperability
- **Dynamic Imports**: Enhanced support for dynamic require() patterns
- **Build Optimization**: Further performance improvements
- **Tooling Integration**: Enhanced IDE and development tool support

---

This documentation provides comprehensive coverage of Rspack's CommonJS implementation. For specific use cases or advanced scenarios, refer to the detailed documentation files or consult the main Rspack documentation.