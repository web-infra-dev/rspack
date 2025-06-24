# Rust Implementation Tracing: CommonJS Dependencies in Rspack

## Table of Contents
1. [File Structure and Implementation Map](#file-structure-and-implementation-map)
2. [Dependency Type Implementations](#dependency-type-implementations)
3. [Template Rendering Chain](#template-rendering-chain)
4. [Build Process Analysis](#build-process-analysis)
5. [Implementation Status Summary](#implementation-status-summary)

> **Note**: For Module Federation integration points, see [CommonJS Integration with Module Federation](../module-federation/commonjs-integration.md).

## File Structure and Implementation Map

### Core CommonJS Implementation Directory
`/crates/rspack_plugin_javascript/src/dependency/commonjs/`

```
commonjs/
├── mod.rs                              # Module exports and public interface
├── common_js_exports_dependency.rs     # Enhanced: Exports handling with macro support
├── common_js_require_dependency.rs     # Enhanced: Require calls with ConsumeShared detection
├── common_js_full_require_dependency.rs # Standard: Property access on requires
├── common_js_export_require_dependency.rs # Enhanced: Re-exports with macro support
├── common_js_self_reference_dependency.rs # Standard: Self-references
├── module_decorator_dependency.rs      # Standard: Module decorators
├── require_ensure_dependency.rs        # Standard: Dynamic imports
├── require_ensure_item_dependency.rs   # Standard: Ensure items
├── require_header_dependency.rs        # Standard: Require headers
├── require_resolve_dependency.rs       # Standard: Require.resolve
└── require_resolve_header_dependency.rs # Standard: Resolve headers
```

**Legend:**
- **Enhanced**: Extended for Module Federation ConsumeShared support
- **Standard**: Core rspack implementation

## Dependency Type Implementations

### 1. CommonJsExportsDependency
**File:** `common_js_exports_dependency.rs`

This dependency handles all CommonJS export patterns with ConsumeShared detection and macro infrastructure.

#### Key Features Implemented:
```rust
pub struct CommonJsExportsDependency {
  id: DependencyId,
  range: DependencyRange,
  value_range: Option<DependencyRange>,    // For Object.defineProperty support
  base: ExportsBase,                       // Export type classification
  names: Vec<Atom>,                        // Property names
  source_map: Option<SharedSourceMap>,
  resource_identifier: Option<String>,     // Unique tracking ID
}
```

#### ExportsBase Implementation:
```rust
pub enum ExportsBase {
  Exports,                    // exports.foo = value
  ModuleExports,             // module.exports.foo = value  
  This,                      // this.foo = value
  DefinePropertyExports,     // Object.defineProperty(exports, 'foo', descriptor)
  DefinePropertyModuleExports, // Object.defineProperty(module.exports, 'foo', descriptor)
  DefinePropertyThis,        // Object.defineProperty(this, 'foo', descriptor)
}
```

#### Template Rendering Implementation:
```rust
impl DependencyTemplate for CommonJsExportsDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, code_generatable_context: &mut TemplateContext) {
    // 1. ConsumeShared Detection
    let consume_shared_info = Self::detect_consume_shared_context(&module_graph, &dep.id, &module_identifier);
    
    // 2. Export Usage Analysis  
    let used = Self::get_used_export_name(&module_graph, current_module.as_ref(), runtime, &dep.names);
    
    // 3. Base Expression Generation
    let (base_expression, runtime_condition) = Self::generate_base_expression(&dep.base, current_module.as_ref(), runtime_requirements, runtime, &consume_shared_info);
    
    // 4. Code Generation with Macro Support
    Self::render_export_statement(dep, source, init_fragments, &base_expression, &used, &consume_shared_info, runtime_condition)
  }
}
```

#### ConsumeShared Detection Algorithm:
```rust
fn detect_consume_shared_context(module_graph: &ModuleGraph, dep_id: &DependencyId, module_identifier: &ModuleIdentifier) -> Option<String> {
  // Step 1: Check direct parent module
  if let Some(parent_module_id) = module_graph.get_parent_module(dep_id) {
    if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
      if parent_module.module_type() == &ModuleType::ConsumeShared {
        return parent_module.get_consume_shared_key();
      }
    }
  }

  // Step 2: Check incoming connections (fallback detection)
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

### 2. CommonJsRequireDependency
**File:** `common_js_require_dependency.rs`

This dependency processes `require()` calls with ConsumeShared detection and comprehensive logging.

#### Implementation Details:
```rust
impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, code_generatable_context: &mut TemplateContext) {
    // Debug logging for analysis
    dbg!(&dep.request);
    
    // ConsumeShared detection
    let consume_shared_info = if let Some(target_module_id) = module_identifier {
      let detected = Self::detect_consume_shared_context(module_graph, &dep.id, &target_module_id, &dep.request);
      dbg!(&dep.request, &detected);
      detected
    } else {
      None
    };

    // Generate module reference
    let base_module_reference = module_id(code_generatable_context.compilation, &dep.id, &dep.request, false);

    // Generate final replacement with conditional macros
    let final_replacement = if let Some(share_key) = consume_shared_info {
      format!("/* @common:if [condition=\"treeShake.{}.default\"] */ {} /* @common:endif */", share_key, base_module_reference)
    } else {
      base_module_reference.to_string()
    };

    source.replace(dep.range.start, dep.range.end - 1, &final_replacement, None);
  }
}
```

### 3. CommonJsExportRequireDependency  
**File:** `common_js_export_require_dependency.rs`

This dependency handles re-export patterns (`module.exports = require('./other')`) with macro generation capabilities.

#### Key Implementation:
```rust
// For expression-based re-exports
let assignment = format!("{base}{} = {require_expr}", property_access(used, 0));

// Wrap with macro comments for ConsumeShared modules
if let Some(ref share_key) = consume_shared_info {
  if let Some(export_name) = dep.names.first() {
    format!("/* @common:if [condition=\"treeShake.{share_key}.{export_name}\"] */ {assignment} /* @common:endif */")
  } else {
    format!("/* @common:if [condition=\"treeShake.{share_key}.default\"] */ {assignment} /* @common:endif */")
  }
} else {
  assignment
}
```

## Template Rendering Chain

### 1. Dependency Resolution
```
Source Code
    ↓
Parser Analysis (creates specific CommonJS dependency types)
    ↓ 
Dependency Registration (assigns unique IDs)
    ↓
Module Graph Integration
```

### 2. Template Processing
```
Template Resolution
    ↓
ConsumeShared Context Detection
    ↓
Export Usage Analysis
    ↓
Runtime Requirements Management
    ↓
Macro Generation (if applicable)
    ↓
Source Code Replacement
```

### 3. Init Fragment Integration
```
Fragment Creation
    ↓
Stage-based Sorting (StageConstants, StageESMExports, etc.)
    ↓
Key-based Grouping (InitFragmentKey::CommonJsExports)
    ↓
Fragment Merging
    ↓
Content Generation
```

## Module Federation Integration Points

### ShareRuntimePlugin Integration
**File:** `/crates/rspack_plugin_mf/src/sharing/share_runtime_plugin.rs`

```rust
impl Plugin for ShareRuntimePlugin {
  fn apply(&self, ctx: PluginContext<&mut rspack_core::ApplyContext>, options: &rspack_core::CompilerOptions) -> Result<()> {
    if self.enable_export_usage_tracking {
      // Automatically register ShareUsagePlugin
      ShareUsagePlugin::new(ShareUsagePluginOptions::default())
        .apply(PluginContext::with_context(ctx.context), options)?;
    }
    Ok(())
  }
}
```

### ShareUsagePlugin Implementation
**File:** `/crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs`

#### Enhanced Export Tracking:
```rust
fn extract_all_imported_exports(&self, module_graph: &ModuleGraph, dependency: &dyn Dependency) -> Vec<String> {
  match dependency.dependency_type() {
    DependencyType::CjsRequire => {
      if let Some(module_dep) = dependency.as_module_dependency() {
        let request = module_dep.request();
        if !request.is_empty() {
          if !all_imported_exports.contains(&"default".to_string()) {
            all_imported_exports.push("default".to_string());
          }
          // Parse property access patterns
          let dep_str = format!("{dependency:?}");
          if let Some(property_name) = self.parse_cjs_property_access(&dep_str) {
            if !all_imported_exports.contains(&property_name) {
              all_imported_exports.push(property_name);
            }
          }
        }
      }
    }
    DependencyType::CjsFullRequire => {
      // Handle property access on require calls
      if let Some(module_dep) = dependency.as_module_dependency() {
        let request = module_dep.request();
        if !request.is_empty() && !all_imported_exports.contains(&request) {
          all_imported_exports.push(request.to_string());
        }
      }
    }
    // ... other dependency types
  }
}
```

## Build Process Analysis

### Debug Output Analysis
From our build, we can see the complete Module Federation architecture in action:

```
🔍 DEBUG: Module type: ConsumeShared, ID: consume shared module (default) cjs-pure-helper@* (strict) (fallback: javascript/dynamic|/Users/.../cjs-modules/pure-cjs-helper.js)
🔍 DEBUG: Found ConsumeShared module with share_key: cjs-pure-helper
🔍 DEBUG: Found fallback module for cjs-pure-helper: javascript/dynamic|/Users/.../cjs-modules/pure-cjs-helper.js
🔍 DEBUG: ConsumeShared shows ProvidedAll - checking fallback for specific exports
```

### Generated ShareUsage Output:
```json
{
  "consume_shared_modules": {
    "cjs-pure-helper": {
      "used_exports": ["CONSTANTS"],
      "unused_exports": ["DataValidator", "createValidator", "generateId", "hashString", "helpers", "info", "processData", "validateInput"],
      "possibly_unused_exports": []
    },
    "cjs-legacy-utils": {
      "used_exports": [],
      "unused_exports": [],
      "possibly_unused_exports": []
    },
    "cjs-data-processor": {
      "used_exports": [],
      "unused_exports": [],
      "possibly_unused_exports": []
    }
  }
}
```

### Template Rendering Results

#### ESM Modules (Full Macro Support):
```javascript
__webpack_require__.d(__webpack_exports__, {
  capitalize: () => (/* @common:if [condition="treeShake.utility-lib.capitalize"] */ 
                     /* ESM export specifier */ capitalize 
                     /* @common:endif */)
});
```

#### CommonJS Modules (Basic Rendering):
```javascript
// Currently renders without macros due to architectural constraints
exports.formatPath = function (filePath) {
  return path.normalize(filePath);
};
```

## System Architecture Overview

### ShareUsagePlugin Operation
The ShareUsagePlugin integrates with the Module Federation system through automatic registration when export usage tracking is enabled. It processes all dependency types including CommonJS, analyzing module relationships to generate comprehensive usage reports in `share-usage.json`.

### ConsumeShared Detection Mechanism
The system uses a multi-level detection algorithm that examines:
1. **Direct parent modules** for ConsumeShared type classification
2. **Incoming connections** to identify Module Federation contexts
3. **Fallback modules** to establish proper sharing relationships

### Template Rendering Pipeline
The template system processes dependencies through these components:
1. **Dependency Analysis**: Type-specific processing for each CommonJS pattern
2. **Runtime Requirements**: Dynamic calculation of necessary runtime globals
3. **Code Generation**: Source replacement with proper module references
4. **Fragment Integration**: Organized initialization code insertion

### Module Federation Integration
The system automatically registers ShareUsagePlugin through ShareRuntimePlugin when Module Federation is configured with shared dependencies. This creates a seamless integration between CommonJS processing and the broader Module Federation ecosystem.

## File Architecture and Responsibilities

| File | Role | Key Functionality |
|------|------|-------------------|
| `common_js_exports_dependency.rs` | Export handling | ConsumeShared detection, macro infrastructure, template rendering |
| `common_js_require_dependency.rs` | Require processing | Module resolution, ConsumeShared detection, source replacement |  
| `common_js_export_require_dependency.rs` | Re-export handling | Macro generation for re-exports, usage analysis |
| `share_usage_plugin.rs` | Usage tracking | Dependency analysis, export tracking, report generation |
| `share_runtime_plugin.rs` | Plugin integration | Automatic ShareUsagePlugin registration, system coordination |
| `init_fragment.rs` | Code generation | Fragment management, staged rendering, content organization |

This architecture provides comprehensive CommonJS Module Federation support with sophisticated dependency tracking and flexible template rendering capabilities.