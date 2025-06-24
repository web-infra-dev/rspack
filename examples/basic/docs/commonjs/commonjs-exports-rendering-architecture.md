# CommonJS Exports Module Rendering Architecture in Rspack

## Table of Contents
1. [Overview](#overview)
2. [CommonJS Dependency Architecture](#commonjs-dependency-architecture)
3. [Export Rendering Pipeline](#export-rendering-pipeline)
4. [Init Fragments System](#init-fragments-system)
5. [Tree-Shaking Macro Generation](#tree-shaking-macro-generation)
6. [Template Rendering Process](#template-rendering-process)
7. [ConsumeShared Integration](#consumeshared-integration)
8. [Implementation Details](#implementation-details)

## Overview

This document provides a comprehensive analysis of how CommonJS exports modules are rendered in chunks within the Rspack build system, focusing on the interaction between dependency analysis, init fragments, and template rendering. For Module Federation specific integration, see [CommonJS Integration with Module Federation](../module-federation/commonjs-integration.md).

## CommonJS Dependency Architecture

### Core Dependency Types

Rspack handles CommonJS dependencies through a sophisticated type hierarchy defined in `/crates/rspack_plugin_javascript/src/dependency/commonjs/`:

#### 1. **CommonJsExportsDependency**
Handles various CommonJS export patterns:
- `exports.*` - Direct property assignments
- `module.exports` - Module-level exports  
- `Object.defineProperty(exports, ...)` - Property descriptors
- Context-based exports (`this.*`)

```rust
#[derive(Debug, Clone)]
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: Option<DependencyRange>,
  base: ExportsBase,
  names: Vec<Atom>,
  source_map: Option<SharedSourceMap>,
  resource_identifier: Option<String>,
}
```

#### 2. **CommonJsRequireDependency**
Handles `require()` calls with ConsumeShared detection:
- Module resolution
- ConsumeShared context detection
- Macro generation for shared modules

#### 3. **CommonJsFullRequireDependency**
Handles property access on require calls (`require('./module').property`)

#### 4. **CommonJsExportRequireDependency**
Handles re-exports (`module.exports = require('./other')`)

### ExportsBase Enum

The `ExportsBase` enum defines the different export contexts:

```rust
pub enum ExportsBase {
  Exports,                    // exports.*
  ModuleExports,             // module.exports.*
  This,                      // this.*
  DefinePropertyExports,     // Object.defineProperty(exports, ...)
  DefinePropertyModuleExports, // Object.defineProperty(module.exports, ...)
  DefinePropertyThis,        // Object.defineProperty(this, ...)
}
```

## Export Rendering Pipeline

### 1. **Dependency Creation**
```
Source Code Analysis
    ↓
CommonJS Parser (creates specific dependency types)
    ↓
Resource Identifier Generation (unique tracking)
    ↓
Export Specification Creation
```

### 2. **Template Processing Flow**
```
Dependency Template Processing
    ↓
ConsumeShared Detection & Export Usage Analysis
    ↓
Base Expression & Runtime Requirements Generation
    ↓
Conditional Macro Generation (if ConsumeShared)
    ↓
Init Fragment Creation & Insertion
    ↓
Template Source Replacement
    ↓
Final Chunk Output
```

## Init Fragments System

### Core Architecture

Init fragments provide the infrastructure for adding initialization code to chunks. The system is defined in `/crates/rspack_core/src/init_fragment.rs`:

#### InitFragmentStage Enum
```rust
pub enum InitFragmentStage {
  StageConstants,        // Variable declarations
  StageAsyncBoundary,    // Async setup  
  StageESMExports,       // Export definitions
  StageESMImports,       // Import statements
  StageProvides,         // Module provision
  StageAsyncDependencies, // Async dependencies
  StageAsyncESMImports,   // Async ESM imports
}
```

#### InitFragmentKey Types
```rust
pub enum InitFragmentKey {
  Unique(u32),
  ESMImport(String),
  ESMExportStar(String),
  ESMExports,
  CommonJsExports(String), // ← Used for CommonJS exports
  ModuleExternal(String),
  ExternalModule(String),
  AwaitDependencies,
  ESMCompatibility,
  ModuleDecorator(String),
  ESMFakeNamespaceObjectFragment(String),
  Const(String),
}
```

### Fragment Rendering Process

The `render_init_fragments` function orchestrates the fragment rendering:

1. **Sorting**: Fragments are sorted by stage, then by position
2. **Grouping**: Fragments with the same key are grouped together
3. **Merging**: Key-specific merge logic is applied
4. **Rendering**: Contents are generated and concatenated

### CommonJS-Specific Fragment Usage

```rust
// Example: Adding a CommonJS export fragment
init_fragments.push(
  NormalInitFragment::new(
    format!("var __webpack_unused_export__;\n"),
    InitFragmentStage::StageConstants,
    0,
    InitFragmentKey::CommonJsExports("unused_export".to_owned()),
    None,
  ).boxed(),
);
```

## Tree-Shaking Macro Generation

### Macro Structure

CommonJS modules in ConsumeShared contexts receive conditional macros:

```javascript
/* @common:if [condition="treeShake.moduleName.exportName"] */
exports.exportName = value;
/* @common:endif */
```

### Implementation Details

#### ConsumeShared Detection Algorithm
```rust
fn detect_consume_shared_context(
  module_graph: &ModuleGraph,
  dep_id: &DependencyId,
  module_identifier: &ModuleIdentifier,
) -> Option<String> {
  // 1. Check direct parent module
  if let Some(parent_module_id) = module_graph.get_parent_module(dep_id) {
    if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
      if parent_module.module_type() == &ModuleType::ConsumeShared {
        return parent_module.get_consume_shared_key();
      }
    }
  }

  // 2. Check incoming connections for ConsumeShared modules
  for connection in module_graph.get_incoming_connections(module_identifier) {
    if let Some(origin_module) = connection.original_module_identifier.as_ref() {
      if let Some(origin_module_obj) = module_graph.module_by_identifier(origin_module) {
        if origin_module_obj.module_type() == &ModuleType::ConsumeShared {
          return origin_module_obj.get_consume_shared_key();
        }
      }
    }
  }

  None
}
```

#### Macro Generation Logic
```rust
// For expression-based exports
fn render_expression_export(
  dep: &CommonJsExportsDependency,
  source: &mut TemplateReplaceSource,
  // ... other params
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  match used {
    Some(UsedName::Normal(used_names)) => {
      let export_assignment = format!("{}{}", base_expression, property_access(used_names, 0));
      
      // Currently disabled macro generation for CommonJS
      // Need architectural changes for proper conditional rendering
      source.replace(dep.range.start, dep.range.end, &export_assignment, None);
    }
    Some(UsedName::Inlined(_)) => {
      Self::render_placeholder_export(dep, source, init_fragments, "inlined")?;
    }
    _ => {
      Self::render_placeholder_export(dep, source, init_fragments, "unused")?;
    }
  }
  Ok(())
}
```

## Template Rendering Process

### Expression-Based Exports

**Before Processing:**
```javascript
exports.myFunction = function() { /* implementation */ };
```

**After Template Rendering:**
```javascript
__webpack_exports__.myFunction = function() { /* implementation */ };
```

### Define Property Exports

**Before Processing:**
```javascript
Object.defineProperty(exports, 'prop', { value: val, enumerable: true });
```

**After Template Rendering:**
```javascript
Object.defineProperty(__webpack_exports__, "prop", { value: val, enumerable: true });
```

### ConsumeShared with Macros (ESM Example)

**ESM modules get full macro treatment:**
```javascript
__webpack_require__.d(__webpack_exports__, {
  capitalize: () => (/* @common:if [condition="treeShake.utility-lib.capitalize"] */ 
                     /* ESM export specifier */ capitalize 
                     /* @common:endif */)
});
```

## ConsumeShared Integration

### Module Federation Architecture

The debug output from our build shows the complete Module Federation architecture:

```
🔍 DEBUG: Module type: ConsumeShared, ID: consume shared module (default) cjs-pure-helper@* (strict) (fallback: javascript/dynamic|/Users/.../cjs-modules/pure-cjs-helper.js)
🔍 DEBUG: Found ConsumeShared module with share_key: cjs-pure-helper
🔍 DEBUG: Found fallback module for cjs-pure-helper: javascript/dynamic|/Users/.../cjs-modules/pure-cjs-helper.js
🔍 DEBUG: ConsumeShared shows ProvidedAll - checking fallback for specific exports
```

### ShareUsagePlugin Integration

The ShareUsagePlugin successfully tracks CommonJS modules and generates `share-usage.json`:

```json
{
  "consume_shared_modules": {
    "cjs-pure-helper": {
      "used_exports": ["CONSTANTS"],
      "unused_exports": [
        "DataValidator", "createValidator", "generateId", 
        "hashString", "helpers", "info", "processData", "validateInput"
      ],
      "possibly_unused_exports": []
    },
    "cjs-legacy-utils": {
      "used_exports": [],
      "unused_exports": [],
      "possibly_unused_exports": []
    }
  }
}
```

## Implementation Details

### Current Architectural Limitations

1. **CommonJS Macro Generation**: Currently disabled due to syntax compatibility issues
2. **ConsumeShared Detection**: Works but requires architectural changes for full macro support
3. **Template Rendering**: Basic replacement works, conditional macros need enhancement

### Runtime Requirements Management

```rust
fn generate_base_expression(
  base: &ExportsBase,
  module: &dyn rspack_core::Module,
  runtime_requirements: &mut RuntimeGlobals,
  // ... other params
) -> (String, Option<RuntimeCondition>) {
  let base_expr = match base {
    ExportsBase::Exports => {
      runtime_requirements.insert(RuntimeGlobals::EXPORTS);
      module.get_exports_argument().to_string()
    }
    ExportsBase::ModuleExports => {
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      format!("{}.exports", module.get_module_argument())
    }
    // ... other cases
  };
  // ...
}
```

### Error Handling and Fallbacks

```rust
fn render_fallback_export(
  dep: &CommonJsExportsDependency,
  source: &mut TemplateReplaceSource,
  init_fragments: &mut rspack_core::ModuleInitFragments,
) {
  let fallback_var = "__webpack_export_fallback__";
  source.replace(dep.range.start, dep.range.end, fallback_var, None);

  init_fragments.push(
    NormalInitFragment::new(
      format!("var {fallback_var};\n"),
      InitFragmentStage::StageConstants,
      0,
      InitFragmentKey::CommonJsExports(fallback_var.to_owned()),
      None,
    ).boxed(),
  );
}
```

## System Operation

### ShareUsagePlugin Integration
The ShareUsagePlugin tracks CommonJS modules by analyzing dependency relationships in the module graph. When a CommonJS module is consumed through Module Federation, the plugin records which exports are used and generates a comprehensive usage report in `share-usage.json`.

### ConsumeShared Detection Process
CommonJS dependencies are detected as ConsumeShared through a multi-level algorithm:
1. Direct parent module inspection for ConsumeShared type
2. Incoming connection analysis for Module Federation context
3. Fallback module identification for proper sharing

### Template Rendering Operation
The template rendering system processes CommonJS exports through these stages:
1. Dependency analysis and export specification generation
2. Runtime requirements calculation and base expression creation
3. Init fragment integration for code organization
4. Final source code replacement with proper module references

This architecture enables sophisticated CommonJS module federation with comprehensive export tracking and flexible rendering capabilities.