# CommonJS ConsumeShared Technical Analysis

## Executive Summary

This document provides a comprehensive technical analysis of why CommonJS dependencies don't receive ConsumeShared macro generation in Rspack's Module Federation implementation, based on detailed investigation of the dependency processing pipeline.

## Root Cause Analysis

### 1. Architecture Differences: ESM vs CommonJS Processing

**ESM Dependencies:**
- Use sophisticated init fragment system for runtime code generation
- Have built-in ConsumeShared detection infrastructure
- Support conditional macro generation through template rendering
- Generate runtime code through `ESMExportInitFragment`

**CommonJS Dependencies:**
- Use direct source replacement in templates
- Limited init fragment usage (only variable declarations)
- Missing ConsumeShared detection in most dependency types
- No runtime code generation infrastructure

### 2. Dependency Type Analysis

| Dependency Type | ConsumeShared Detection | Macro Generation | Template Complexity |
|----------------|------------------------|------------------|-------------------|
| `ESMImportSpecifierDependency` | ✅ Full | ✅ Full | High (init fragments) |
| `ESMExportSpecifierDependency` | ✅ Full | ✅ Full | High (init fragments) |
| `CommonJsRequireDependency` | ❌ None | ❌ None | Low (direct replacement) |
| `CommonJsExportsDependency` | ⚠️ Partial | ⚠️ Partial | Medium (limited macros) |
| `CommonJsFullRequireDependency` | ❌ None | ❌ None | Low (direct replacement) |

### 3. Template Rendering Pipeline Gaps

**Missing in CommonJS:**
- ConsumeShared ancestry detection during template rendering
- Macro wrapping infrastructure for conditional code generation
- Integration with Module Federation's sharing scope resolution
- Runtime code generation through init fragments

## Technical Investigation Findings

### File-by-File Analysis

#### `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`

**Current Implementation:**
```rust
// Lines 140-164: Simple module ID replacement
source.replace(
  dep.range.start,
  dep.range.end - 1,
  module_id(
    code_generatable_context.compilation,
    &dep.id,
    &dep.request,
    false,
  ).as_str(),
  None,
);
```

**Missing Capabilities:**
- No ConsumeShared detection
- No macro generation
- Direct module ID replacement without conditional logic

#### `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_exports_dependency.rs`

**Existing ConsumeShared Support (Lines 356-421):**
```rust
fn get_consume_shared_info(&self, module_graph: &ModuleGraph) -> Option<String> {
  // Partial implementation exists but incomplete
  if let Some(parent_module_id) = module_graph.get_parent_module(&self.id) {
    // Check ConsumeShared ancestry
  }
}
```

**Macro Generation (Lines 547-563):**
```rust
let export_content = if let Some(ref share_key) = consume_shared_info {
  format!(
    "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
    share_key, export_name, export_assignment
  )
} else {
  export_assignment
};
```

### Init Fragment System Analysis

#### `/Users/bytedance/RustroverProjects/rspack/crates/rspack_core/src/init_fragment.rs`

**ESM Integration:**
```rust
pub enum InitFragmentKey {
  ESMImport(String),
  ESMExports(String),
  CommonJsExports(String), // Limited to variable declarations
  // Missing: CommonJsConsumeShared, CommonJsRequire
}
```

**CommonJS Limitations:**
- Only `CommonJsExports` for variable declarations
- No runtime code generation support
- Missing ConsumeShared integration

## Implementation Strategy

### Phase 1: Core Infrastructure Enhancement

#### 1.1 Add ConsumeShared Detection Utility
```rust
// New utility function for all CommonJS dependencies
pub fn get_commonjs_consume_shared_info(
  dependency_id: &DependencyId,
  module_graph: &ModuleGraph,
) -> Option<String> {
  // Traverse module graph to find ConsumeShared ancestry
  // Return share_key if found
}
```

#### 1.2 Enhance Init Fragment System
```rust
// Add new init fragment types for CommonJS ConsumeShared
pub enum InitFragmentKey {
  // Existing...
  CommonJsConsumeShared(String),
  CommonJsRequireMacro(String),
}
```

#### 1.3 Create Macro Generation Utility
```rust
pub fn generate_commonjs_macro(
  share_key: &str,
  export_name: &str,
  original_code: &str,
) -> String {
  format!(
    "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
    share_key, export_name, original_code
  )
}
```

### Phase 2: Dependency-Specific Implementation

#### 2.1 Update CommonJsRequireDependency Template
```rust
impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    let dep = dep.as_any().downcast_ref::<CommonJsRequireDependency>().unwrap();
    
    // Add ConsumeShared detection
    let consume_shared_info = get_commonjs_consume_shared_info(&dep.id, &ctx.module_graph);
    
    let module_reference = module_id(ctx.compilation, &dep.id, &dep.request, false);
    
    let replacement = if let Some(share_key) = consume_shared_info {
      // Generate macro-wrapped replacement
      generate_commonjs_macro(&share_key, "default", module_reference.as_str())
    } else {
      module_reference.to_string()
    };
    
    source.replace(dep.range.start, dep.range.end - 1, &replacement, None);
  }
}
```

#### 2.2 Enhance CommonJsExportsDependency
- Expand existing ConsumeShared detection
- Improve macro generation consistency
- Add missing edge cases for property access patterns

### Phase 3: Pipeline Integration

#### 3.1 Runtime Template Enhancement
```rust
// In /Users/bytedance/RustroverProjects/rspack/crates/rspack_core/src/dependency/runtime_template.rs
pub fn module_id_with_consume_shared(
  &self,
  compilation: &Compilation,
  dependency_id: &DependencyId,
  request: &str,
  weak: bool,
  share_key: Option<&str>,
  export_name: Option<&str>,
) -> RuntimeCondition {
  let base_module_id = self.module_id(compilation, dependency_id, request, weak);
  
  if let (Some(key), Some(name)) = (share_key, export_name) {
    RuntimeCondition::Boolean(format!(
      "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
      key, name, base_module_id
    ))
  } else {
    base_module_id
  }
}
```

#### 3.2 Module Graph Integration
- Ensure ConsumeShared detection works with CommonJS dependency traversal
- Add support for CommonJS-specific sharing patterns
- Integrate with existing tree-shaking optimization passes

### Testing Strategy

#### 3.1 Unit Tests
- ConsumeShared detection for CommonJS requires
- Macro generation for different CommonJS patterns
- Init fragment integration for runtime code

#### 3.2 Integration Tests
- End-to-end CommonJS sharing scenarios
- Mixed ESM/CommonJS sharing validation
- Tree-shaking effectiveness with CommonJS macros

#### 3.3 Regression Tests
- Ensure existing CommonJS functionality preserved
- Validate ESM sharing still works correctly
- Performance impact assessment

## Risk Assessment

### Low Risk Changes
- Adding ConsumeShared detection utilities
- Enhancing existing macro generation in `CommonJsExportsDependency`
- Adding new init fragment types

### Medium Risk Changes
- Modifying `CommonJsRequireDependency` template rendering
- Integrating with runtime template system
- Extending module graph traversal logic

### High Risk Changes
- Fundamental changes to CommonJS dependency structure
- Modifying core compilation pipeline
- Breaking changes to existing APIs

## Implementation Priority

1. **High Priority**: ConsumeShared detection for `CommonJsRequireDependency`
2. **Medium Priority**: Macro generation infrastructure enhancement
3. **Low Priority**: Init fragment system extension for advanced runtime code generation

## Success Criteria

1. **Functional**: CommonJS modules accessed via `require()` generate `@common:if` macros when appropriate
2. **Compatible**: Existing CommonJS and ESM functionality preserved
3. **Performance**: No significant compilation time impact
4. **Testable**: Comprehensive test coverage for new functionality

## Next Steps

1. Implement ConsumeShared detection utility (Phase 1.1)
2. Update `CommonJsRequireDependency` template (Phase 2.1)
3. Create comprehensive test suite
4. Validate with existing Module Federation scenarios
5. Performance testing and optimization