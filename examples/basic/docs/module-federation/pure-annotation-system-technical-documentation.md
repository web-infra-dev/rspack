# Rspack PURE Annotation System: Technical Documentation

## Overview

The PURE annotation system in Rspack's Module Federation is a sophisticated tree-shaking mechanism that applies `/* #__PURE__ */` annotations to import statements to enable dead code elimination. This system specifically targets modules that descend from ConsumeShared modules, helping bundlers identify and eliminate unused imports in federated module scenarios.

## System Architecture

### Core Components

1. **ConsumeShared Module Detection**: Identifies modules using the Module Federation sharing system
2. **Ancestry Analysis**: Traverses the module graph to detect ConsumeShared descendants
3. **Import Classification**: Distinguishes between different types of import statements
4. **PURE Annotation Application**: Conditionally applies PURE annotations based on module ancestry

### Key Data Structures

```rust
// Module type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleType {
    ConsumeShared,    // Shared modules in Module Federation
    JsAuto,          // Regular JavaScript modules  
    JsEsm,           // ES Module JavaScript
    // ... other types
}

// Dependency type classification for imports
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DependencyType {
    EsmImport,         // "esm import" - bare imports like `import './module'`
    EsmImportSpecifier, // "esm import specifier" - named imports like `import { func } from './module'`
    // ... other types
}
```

## PURE Annotation Implementation

### Location: `runtime_template.rs`

The core PURE annotation logic is implemented in the `import_statement` function:

```rust
pub fn import_statement(
  module: &dyn Module,
  compilation: &Compilation,
  runtime_requirements: &mut RuntimeGlobals,
  id: &DependencyId,
  request: &str,
  update: bool,
) -> (String, String) {
  // ... module resolution logic ...

  // Check if this import should be marked as pure
  // Only apply PURE annotations to modules that descend from ConsumeShared modules
  let is_pure = compilation
    .get_module_graph()
    .dependency_by_id(id)
    .is_some_and(|dep| {
      // Check the dependency type to distinguish between different import types
      let dep_type = dep.dependency_type();
      // Include both "esm import" (bare imports) and "esm import specifier" (named imports)
      // but exclude __webpack_require__ calls themselves
      let is_esm_import = matches!(dep_type.as_str(), "esm import" | "esm import specifier") 
        && import_var != "__webpack_require__";
      
      // Only apply PURE annotation if this is an ESM import AND descends from ConsumeShared
      if is_esm_import {
        // Check if the current module or any ancestor is ConsumeShared
        let module_graph = compilation.get_module_graph();
        is_consume_shared_descendant(&module_graph, &module.identifier())
      } else {
        false
      }
    });

  let pure_annotation = if is_pure { "/* #__PURE__ */ " } else { "" };

  let import_content = format!(
    "/* ESM import */{opt_declaration}{import_var} = {}{}({module_id_expr});\n",
    pure_annotation,
    RuntimeGlobals::REQUIRE
  );

  // ... rest of implementation
}
```

## ConsumeShared Ancestry Detection

### Primary Detection Function

```rust
/// Check if a module is a descendant of a ConsumeShared module
/// by traversing incoming connections recursively
fn is_consume_shared_descendant(
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
) -> bool {
  let mut visited = std::collections::HashSet::new();
  is_consume_shared_descendant_recursive(module_graph, module_identifier, &mut visited, 10)
}
```

### Recursive Ancestry Traversal

```rust
/// Recursively search for ConsumeShared modules in the module graph ancestry
fn is_consume_shared_descendant_recursive(
  module_graph: &ModuleGraph,
  current_module: &ModuleIdentifier,
  visited: &mut std::collections::HashSet<ModuleIdentifier>,
  max_depth: usize,
) -> bool {
  // Prevent infinite loops and excessive depth
  if max_depth == 0 || visited.contains(current_module) {
    return false;
  }
  visited.insert(current_module.clone());

  // Check if current module is ConsumeShared
  if let Some(module) = module_graph.module_by_identifier(current_module) {
    if module.module_type() == &ModuleType::ConsumeShared {
      return true;
    }
  }

  // Check all incoming connections for ConsumeShared ancestors
  for connection in module_graph.get_incoming_connections(current_module) {
    if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
      if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
        // Found a ConsumeShared module - this is a descendant
        if origin_module.module_type() == &ModuleType::ConsumeShared {
          return true;
        }
        
        // Recursively check this module's incoming connections
        if is_consume_shared_descendant_recursive(
          module_graph,
          origin_module_id,
          visited,
          max_depth - 1,
        ) {
          return true;
        }
      }
    }
  }

  false
}
```

### Key Detection Features

1. **Cycle Prevention**: Uses a `visited` HashSet to prevent infinite loops in circular dependencies
2. **Depth Limiting**: Limits recursion depth to 10 levels to prevent stack overflow
3. **Direct Detection**: Immediately returns true if the current module is ConsumeShared
4. **Transitive Detection**: Recursively checks all incoming connections for ConsumeShared ancestors

## Import Statement Classification

### ESM Import vs ESM Import Specifier

The system distinguishes between two types of ES module imports:

#### 1. ESM Import ("esm import")
- **Definition**: Bare import statements without specific specifiers
- **Examples**:
  ```javascript
  import './side-effects-module';
  import 'shared-library';
  ```
- **Dependency Type**: `DependencyType::EsmImport`
- **String Representation**: `"esm import"`

#### 2. ESM Import Specifier ("esm import specifier")  
- **Definition**: Named import statements with specific specifiers
- **Examples**:
  ```javascript
  import { func, Component } from 'shared-library';
  import defaultExport from 'shared-library';
  import * as namespace from 'shared-library';
  ```
- **Dependency Type**: `DependencyType::EsmImportSpecifier`
- **String Representation**: `"esm import specifier"`

### Classification Logic

```rust
let dep_type = dep.dependency_type();
let is_esm_import = matches!(dep_type.as_str(), "esm import" | "esm import specifier") 
  && import_var != "__webpack_require__";
```

**Key Points**:
- Both import types are eligible for PURE annotations
- `__webpack_require__` calls are explicitly excluded
- Only ES module imports are considered (CommonJS requires are not)

## Decision Tree for PURE Annotations

```mermaid
flowchart TD
    A[Import Statement] --> B{Is ESM Import?}
    B -->|No| C[No PURE Annotation]
    B -->|Yes| D{Is import_var == "__webpack_require__"?}
    D -->|Yes| C
    D -->|No| E{Is ConsumeShared Descendant?}
    E -->|No| C
    E -->|Yes| F[Apply PURE Annotation]
    
    E --> G[Check Current Module]
    G --> H{Module Type == ConsumeShared?}
    H -->|Yes| F
    H -->|No| I[Check Incoming Connections]
    I --> J{Has ConsumeShared Ancestor?}
    J -->|Yes| F
    J -->|No| K[Recursive Check with Depth Limit]
    K --> L{Found ConsumeShared in Ancestry?}
    L -->|Yes| F
    L -->|No| C
```

## Code Generation Examples

### Without PURE Annotation (Regular Module)

```javascript
/* ESM import */var module_a = __webpack_require__(/*! ./moduleA */ "./src/moduleA.js");
```

### With PURE Annotation (ConsumeShared Descendant)

```javascript
/* ESM import */var shared_lib = /* #__PURE__ */ __webpack_require__(/*! shared-lib */ "./node_modules/shared-lib/index.js");
```

### Dynamic Export Handling with PURE

For modules with dynamic exports, additional PURE annotations are applied:

```javascript
/* ESM import */var shared_lib = /* #__PURE__ */ __webpack_require__(/*! shared-lib */ "./node_modules/shared-lib/index.js");
/* ESM import */var shared_lib_default = /*#__PURE__*/__webpack_require__.n(shared_lib);
```

## ConsumeShared Module Structure

### Module Creation

```rust
impl ConsumeSharedModule {
  pub fn new(context: Context, options: ConsumeOptions) -> Self {
    let identifier = format!(
      "consume shared module ({}) {}@{}{}{}{}{}",
      &options.share_scope,
      &options.share_key,
      options.required_version.as_ref().map(|v| v.to_string()).unwrap_or_else(|| "*".to_string()),
      if options.strict_version { " (strict)" } else { Default::default() },
      if options.singleton { " (strict)" } else { Default::default() },
      options.import_resolved.as_ref().map(|f| format!(" (fallback: {f})")).unwrap_or_default(),
      if options.eager { " (eager)" } else { Default::default() },
    );
    // ... initialization
  }
}
```

### Fallback Module Discovery

```rust
/// Finds the fallback module identifier for this ConsumeShared module
pub fn find_fallback_module_id(&self, module_graph: &ModuleGraph) -> Option<ModuleIdentifier> {
  // Look through dependencies to find the fallback
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

## Integration with Module Federation

### Module Federation Context

The PURE annotation system is specifically designed for Module Federation scenarios where:

1. **Shared Dependencies**: Multiple micro-frontends share common dependencies
2. **Runtime Resolution**: Dependencies are resolved at runtime from shared scopes
3. **Tree Shaking**: Unused exports need to be eliminated to reduce bundle size
4. **Fallback Handling**: Local fallback modules are used when shared dependencies are unavailable

### Practical Example

```javascript
// Module Federation Configuration
const ModuleFederationPlugin = require('@module-federation/webpack');

module.exports = {
  plugins: [
    new ModuleFederationPlugin({
      name: 'host',
      remotes: {
        mfe1: 'mfe1@http://localhost:3001/remoteEntry.js'
      },
      shared: {
        'react': { singleton: true },
        'lodash': { singleton: true }
      }
    })
  ]
};

// Application Code
import { debounce } from 'lodash'; // This import gets PURE annotation
import { Component } from 'react'; // This import gets PURE annotation

// Generated Code (with PURE annotations)
/* ESM import */var lodash = /* #__PURE__ */ __webpack_require__(/*! lodash */ "webpack/sharing/consume/default/lodash");
/* ESM import */var react = /* #__PURE__ */ __webpack_require__(/*! react */ "webpack/sharing/consume/default/react");
```

## Performance Considerations

### Ancestry Detection Optimization

1. **Visited Set**: Prevents redundant traversal of already-checked modules
2. **Depth Limiting**: Prevents excessive recursion in deeply nested module graphs
3. **Early Exit**: Returns immediately upon finding a ConsumeShared module
4. **Connection Iteration**: Efficiently processes incoming connections without creating intermediate collections

### Memory Usage

- **HashSet Allocation**: Single allocation per ancestry check
- **Identifier Cloning**: Minimal cloning of module identifiers for visited tracking
- **Stack Usage**: Bounded by max_depth parameter (default: 10)

## Error Handling and Edge Cases

### Missing Modules

```rust
if compilation
  .get_module_graph()
  .module_identifier_by_dependency_id(id)
  .is_none()
{
  return (missing_module_statement(request), String::new());
};
```

### Circular Dependencies

The visited HashSet prevents infinite loops in circular dependency scenarios:

```rust
if max_depth == 0 || visited.contains(current_module) {
  return false;
}
```

### Invalid Connections

The system gracefully handles missing or invalid module connections:

```rust
if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
  if let Some(origin_module) = module_graph.module_by_identifier(origin_module_id) {
    // Process valid connection
  }
}
```

## Integration Points

### Runtime Template System

The PURE annotation system integrates with Rspack's runtime template system:

- **RuntimeGlobals::REQUIRE**: Standard webpack require function
- **TemplateContext**: Provides compilation context and module graph access
- **Code Generation**: Produces annotated JavaScript code for tree shaking

### Webpack Compatibility

The generated PURE annotations are compatible with:

- **Webpack's Tree Shaking**: Standard `/* #__PURE__ */` annotation format
- **Terser/UglifyJS**: Minifiers recognize and respect PURE annotations
- **Rollup**: Alternative bundlers can process the annotations

## Debugging and Diagnostics

### Module Identification

ConsumeShared modules can be identified by their module type:

```rust
if module.module_type() == &ModuleType::ConsumeShared {
  // This is a ConsumeShared module
}
```

### Dependency Analysis

Track dependency relationships for debugging:

```rust
let dep_type = dep.dependency_type();
println!("Dependency type: {}", dep_type.as_str());
// Outputs: "esm import" or "esm import specifier"
```

### Connection Traversal

Debug ancestry detection by logging connection traversal:

```rust
for connection in module_graph.get_incoming_connections(current_module) {
  if let Some(origin_module_id) = connection.original_module_identifier.as_ref() {
    println!("Checking connection from: {:?}", origin_module_id);
  }
}
```

## Future Enhancements

### Potential Improvements

1. **Caching**: Cache ancestry detection results to avoid repeated traversal
2. **Parallel Processing**: Process multiple ancestry checks concurrently
3. **Heuristic Optimization**: Use module naming patterns to optimize detection
4. **Statistical Analysis**: Track and report PURE annotation effectiveness

### Configuration Options

Potential configuration for fine-tuning:

```rust
pub struct PureAnnotationConfig {
  pub max_traversal_depth: usize,
  pub enable_caching: bool,
  pub include_dynamic_imports: bool,
  pub strict_consume_shared_detection: bool,
}
```

## Conclusion

The Rspack PURE annotation system provides sophisticated tree-shaking capabilities for Module Federation scenarios by:

1. **Precise Detection**: Accurately identifying ConsumeShared module descendants
2. **Import Classification**: Distinguishing between different types of ES module imports  
3. **Conditional Annotation**: Applying PURE annotations only where beneficial
4. **Performance Optimization**: Efficient graph traversal with cycle prevention
5. **Standards Compliance**: Generating standard PURE annotations for bundler compatibility

This system enables effective dead code elimination in complex Module Federation setups while maintaining correctness and performance.