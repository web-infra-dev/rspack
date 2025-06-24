# CommonJsRequireDependency Complete Lifecycle Analysis

## Overview

`CommonJsRequireDependency` is responsible for handling CommonJS `require()` calls in Rspack. This document traces its complete lifecycle from parsing to code generation.

## 1. Creation/Instantiation

### 1.1 Parser Plugin Registration

**File**: `/crates/rspack_plugin_javascript/src/visitors/dependency/parser/mod.rs`
```rust
// Line 339: CommonJsImportsParserPlugin is registered for JS modules
if module_type.is_js_auto() || module_type.is_js_dynamic() {
  plugins.push(Box::new(parser_plugin::CommonJsImportsParserPlugin));
  // ...
}
```

### 1.2 Parsing require() Calls

**File**: `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_imports_parse_plugin.rs`

#### Key Functions:

1. **`require_handler()`** (Lines 263-331)
   - Handles both `require()` and `new require()` expressions
   - Validates if the expression is a require call
   - Processes require with single argument

2. **`process_require_item()`** (Lines 229-247)
   ```rust
   fn process_require_item(
     &self,
     parser: &mut JavascriptParser,
     span: Span,
     param: &BasicEvaluatedExpression,
   ) -> Option<bool> {
     param.is_string().then(|| {
       let range_expr: DependencyRange = param.range().into();
       let dep = CommonJsRequireDependency::new(
         param.string().to_string(),
         range_expr,
         Some(span.into()),
         parser.in_try,
         Some(parser.source_map.clone()),
       );
       parser.dependencies.push(Box::new(dep));
       true
     })
   }
   ```

### 1.3 Dependency Construction

**File**: `/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`

```rust
pub fn new(
  request: String,
  range: DependencyRange,
  range_expr: Option<DependencyRange>,
  optional: bool,
  source_map: Option<SharedSourceMap>,
) -> Self {
  Self {
    id: DependencyId::new(),
    request,
    optional,
    range,
    range_expr,
    source_map,
    factorize_info: Default::default(),
  }
}
```

## 2. Data Flow

### 2.1 Trigger Points

1. **AST Walking**: When the parser encounters a `CallExpr` with `require` as callee
2. **Expression Evaluation**: The argument to `require()` is evaluated
3. **Dependency Creation**: If the argument is a string literal, a dependency is created

### 2.2 Data Passed During Construction

- **request**: The module specifier (e.g., "./module")
- **range**: The source code range of the entire require expression
- **range_expr**: The optional range for the expression part
- **optional**: Whether the require is in a try-catch block
- **source_map**: Reference to the source map for location tracking

### 2.3 Module Resolution Flow

1. **Dependency Factory Assignment**:
   ```rust
   // File: plugin/impl_plugin_for_js_plugin.rs, Line 78-81
   compilation.set_dependency_factory(
     DependencyType::CjsRequire,
     params.normal_module_factory.clone(),
   );
   ```

2. **Module Graph Integration**:
   - The dependency is added to the parser's dependency list
   - During compilation, the module factory resolves the request
   - A connection is established in the module graph

## 3. Integration Points

### 3.1 Module Graph Integration

**Key Methods**:
- `module_identifier_by_dependency_id()`: Maps dependency to resolved module
- `set_resolved_module()`: Creates connection between modules
- `add_connection()`: Adds the connection to the module graph

### 3.2 Template Rendering Pipeline

**File**: `/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`

1. **Template Registration** (Lines 260-263):
   ```rust
   compilation.set_dependency_template(
     CommonJsRequireDependencyTemplate::template_type(),
     Arc::new(CommonJsRequireDependencyTemplate::default()),
   );
   ```

2. **Rendering Process** (Lines 149-195):
   ```rust
   fn render(
     &self,
     dep: &dyn DependencyCodeGeneration,
     source: &mut TemplateReplaceSource,
     code_generatable_context: &mut TemplateContext,
   ) {
     // Get module identifier and check for ConsumeShared context
     let module_identifier = module_graph
       .module_identifier_by_dependency_id(&dep.id)
       .copied();
     
     // Generate module reference
     let base_module_reference = module_id(
       code_generatable_context.compilation,
       &dep.id,
       &dep.request,
       false,
     );
     
     // Apply ConsumeShared macro if applicable
     let final_replacement = if let Some(share_key) = consume_shared_info {
       format!(
         "/* @common:if [condition=\"treeShake.{}.default\"] */ {} /* @common:endif */",
         share_key, base_module_reference
       )
     } else {
       base_module_reference.to_string()
     };
     
     source.replace(dep.range.start, dep.range.end - 1, &final_replacement, None);
   }
   ```

### 3.3 Module Resolution System

**Key Function**: `module_id()` in `/crates/rspack_core/src/dependency/runtime_template.rs`

```rust
pub fn module_id(
  compilation: &Compilation,
  id: &DependencyId,
  request: &str,
  weak: bool,
) -> String {
  if let Some(module_identifier) = compilation
    .get_module_graph()
    .module_identifier_by_dependency_id(id)
    && let Some(module_id) =
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, *module_identifier)
  {
    module_id_expr(&compilation.options, request, module_id)
  } else if weak {
    "null /* weak dependency, without id */".to_string()
  } else {
    missing_module(request)
  }
}
```

## 4. Usage Patterns

### 4.1 require() Call Processing

1. **Detection**: Uses `is_require_call_start()` to identify require patterns
2. **Validation**: Checks for:
   - Single argument
   - String literal argument (for static requires)
   - Valid require context (not disabled in options)

### 4.2 Template Rendering Triggers

1. During code generation phase
2. When chunks are being rendered
3. After all modules have been resolved

### 4.3 Module Loading

1. **Static Analysis**: Extract module request from AST
2. **Resolution**: Use module factory to resolve the request
3. **Connection**: Establish module graph connection
4. **Code Gen**: Replace require() with runtime module reference

## 5. ConsumeShared Integration

### 5.1 Detection Logic (Lines 115-145)

The template includes special handling for ConsumeShared modules:

```rust
fn detect_consume_shared_context(
  module_graph: &ModuleGraph,
  dep_id: &DependencyId,
  module_identifier: &ModuleIdentifier,
  request: &str,
) -> Option<String> {
  // Check direct parent module for ConsumeShared context
  if let Some(parent_module_id) = module_graph.get_parent_module(dep_id) {
    if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
      if parent_module.module_type() == &ModuleType::ConsumeShared {
        return parent_module.get_consume_shared_key();
      }
    }
  }
  // ... fallback detection via incoming connections
}
```

### 5.2 Code Generation Enhancement

When a ConsumeShared context is detected, the generated code includes tree-shaking macros:

```javascript
/* @common:if [condition="treeShake.lodash.default"] */ __webpack_require__("./node_modules/lodash/lodash.js") /* @common:endif */
```

## 6. Complete Lifecycle Summary

1. **Parse Phase**: 
   - AST walker encounters `require()` call
   - `CommonJsImportsParserPlugin` processes it
   - `CommonJsRequireDependency` is created

2. **Build Phase**:
   - Module factory resolves the request
   - Module graph connection is established
   - Dependencies are linked to modules

3. **Code Generation Phase**:
   - Template renders the dependency
   - Checks for ConsumeShared context
   - Generates appropriate runtime code

4. **Runtime**:
   - Generated code uses `__webpack_require__()` to load modules
   - ConsumeShared macros control conditional loading

## Key Files Reference

1. **Parser Plugin**: `/crates/rspack_plugin_javascript/src/parser_plugin/common_js_imports_parse_plugin.rs`
2. **Dependency Definition**: `/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`
3. **Template Registration**: `/crates/rspack_plugin_javascript/src/plugin/impl_plugin_for_js_plugin.rs`
4. **Runtime Template**: `/crates/rspack_core/src/dependency/runtime_template.rs`
5. **Module Graph**: `/crates/rspack_core/src/module_graph/mod.rs`