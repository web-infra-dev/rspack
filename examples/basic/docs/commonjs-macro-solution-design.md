# Universal Macro Wrapping Solution Design: CommonJS & ESM

**Navigation**: [🏠 Docs Home](nav.md) | [📋 All Files](nav.md)

**Related Documents**:

- [🐛 Problem Analysis](commonjs-macro-wrapping-issue.md) - Detailed issue breakdown and symptoms
- [📊 CommonJS Flow](commonjs-parser-dependency-flow.md) - CommonJS system architecture
- [⚡ ESM Flow](esm-parser-dependency-flow.md) - ESM system architecture

## Table of Contents

- [Critical Assessment](#critical-assessment-wrong-changes-identified)
- [Problem Analysis Summary](#problem-analysis-summary)
- [Comprehensive Change Analysis](#comprehensive-change-analysis)
- [Revised Solution Architecture](#revised-solution-architecture-buildmeta-pattern)
- [Cleanup and Implementation Strategy](#cleanup-and-implementation-strategy)
- [Benefits and Risk Assessment](#benefits-and-risk-assessment)

---

## 🚨 Critical Assessment: Wrong Changes Identified

### **URGENT: Incorrect Changes Must Be Reverted**

Based on thorough analysis, several changes in this branch **violate fundamental ConsumeShared architecture**:

#### **❌ FlagDependencyUsagePlugin Changes (WRONG)**

```rust
// This change is ARCHITECTURALLY WRONG
if module.module_type() == &rspack_core::ModuleType::ConsumeShared {
  self.process_consume_shared_module(/* ... */); // ❌ Applies build-time tree-shaking
  return;
}
```

**Why This is Wrong**:
1. **ConsumeShared modules should NEVER be tree-shaken at build time**
2. **They must remain complete for runtime/server-time selection**
3. **Macro comments are the tree-shaking mechanism, not build-time removal**
4. **This breaks Module Federation's dynamic loading architecture**

#### **⚠️ Distinguish Runtime vs Build-Time Tree-Shaking**

Current branch adds extensive analysis infrastructure, but we need to **distinguish what's needed**:

**✅ KEEP (Runtime Tree-Shaking Support)**:
- PURE annotations in runtime templates - **Essential for webpack_require purity**
- Side effect detection for ConsumeShared modules - **Required for runtime optimization**
- Basic export metadata tracking - **Needed for shared module coordination**

**❌ REMOVE (Build-Time Tree-Shaking Violations)**:
- `FlagDependencyUsagePlugin` ConsumeShared handling - **Breaks Module Federation architecture**
- Complex export usage analysis systems - **Over-engineered for macro coordination**
- Build-time removal of ConsumeShared module exports - **Violates runtime selection principle**

## Problem Analysis Summary

> **🐛 Detailed Breakdown**: See [Root Cause Analysis](commonjs-macro-wrapping-issue.md#root-cause-analysis) and [System Architecture](commonjs-parser-dependency-flow.md) for complete technical details

### **Core Macro Issues (What We Actually Need to Fix)**

**CommonJS Issues:**
1. **Stacked endif tags** from bulk CommonJS exports  
2. **Wrong export values** (`module.exports.foo` vs `foo`)
3. **Shared range conflicts** in CommonJS bulk exports

**ESM Issues:**
4. **Redundant ConsumeShared detection** across multiple ESM export dependencies
5. **Template-time detection** in ESMExportSpecifierDependency, ESMExportExpressionDependency, ESMExportImportedSpecifierDependency  

**Universal Issues (Both Systems):**
6. **Template-time ConsumeShared detection** (expensive repeated operations)
7. **Module graph traversal performance** (O(n) operations per dependency)

### **What We DON'T Need to Fix**

1. ❌ ConsumeShared build-time tree-shaking (breaks architecture)
2. ❌ Over-complex export usage analysis systems  
3. ❌ Module Federation plugin ecosystem overhaul
4. ✅ **Keep PURE annotations** - Essential for runtime tree-shaking purity

## Comprehensive Change Analysis

### **📊 Current Branch Changes (64 files, 22,857 insertions)**

#### **✅ Useful Changes (Keep)**
- `common_js_exports_dependency.rs` - Enhanced macro generation logic
- `common_js_exports_parse_plugin.rs` - Bulk export detection improvements
- **ESM export dependency enhancements**:
  - `esm_export_expression_dependency.rs` - Default export ConsumeShared detection
  - `esm_export_specifier_dependency.rs` - Named export ConsumeShared detection
  - `esm_export_imported_specifier_dependency.rs` - Re-export ConsumeShared detection
- `runtime_template.rs` PURE annotation changes - **Essential for runtime tree-shaking**
- Side effect detection improvements - **Required for webpack_require purity**
- Build log analysis and test infrastructure

#### **❌ Problematic Changes (Remove/Revert)**
- `flag_dependency_usage_plugin.rs` - Wrong ConsumeShared **build-time** tree-shaking
- Over-complex export usage analysis systems - **Excessive for macro coordination**
- Build-time removal of ConsumeShared exports - **Violates Module Federation architecture**

#### **🤔 Neutral Changes (Documentation)**
- Test files and documentation - Keep for reference
- Example configurations - Keep for testing

### **🔍 What the Analysis Reveals**

1. **Total export count IS available** (`obj_lit.props.len()`) but not preserved
2. **ConsumeShared detection logic exists** but runs at wrong time (template vs parser)
3. **BuildMeta is the perfect infrastructure** for module-level metadata
4. **Current changes mix multiple unrelated problems**

## Revised Solution Architecture: NormalModuleFactory + BuildMeta Pattern

### **🎯 Focused Solution Scope**

Our solution should ONLY address:
1. ✅ Macro generation issues
2. ✅ ConsumeShared detection optimization  
3. ✅ Bulk export coordination
4. ❌ NOT tree-shaking behavior changes

### **🏗️ Three-Tier Architecture Implementation**

Based on codebase analysis, the optimal solution uses **existing ConsumeSharedPlugin infrastructure** with **NormalModuleFactory hooks** for early detection.

### **1. Extend ConsumeSharedPlugin (Tier 1: Early Detection)**

```rust
// ENHANCE: Existing ConsumeSharedPlugin with NormalModuleFactoryAfterResolve hook
impl ConsumeSharedPlugin {
  #[plugin_hook(NormalModuleFactoryAfterResolve)]
  async fn after_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
    // Use existing detection logic from lines 715-741 in consume_shared_plugin.rs
    if let Some(config) = self.find_consume_config(&data.request) {
      // Set BuildMeta BEFORE parsing begins (prevents all conflicts)
      data.build_info.build_meta.consume_shared_key = Some(config.share_key.clone());
      
      // Mark for enhanced macro coordination
      data.build_info.build_meta.export_coordination = Some(ExportCoordination::Pending);
    }
    Ok(None)
  }
  
  // LEVERAGE: Existing find_consume_config logic (lines 715-741)
  fn find_consume_config(&self, request: &str) -> Option<&ConsumeSharedConfig> {
    // Use existing MatchedConsumes logic - no changes needed
    // This already handles unresolved, prefixed, resolved patterns
    self.matched_consumes.find_consume_config(request)
  }
}
```

### **2. Minimal BuildMeta Enhancement (Tier 2: Metadata Storage)**

```rust
// SIMPLE: Extend existing BuildMeta structure (crates/rspack_core/src/build_meta.rs)
#[cacheable]
#[derive(Debug, Default, Clone, Hash, Serialize)]
pub struct BuildMeta {
  // ... existing fields unchanged (24 existing fields)
  pub esm: bool,
  pub exports_type: BuildMetaExportsType,
  pub default_object: BuildMetaDefaultObject,
  pub side_effect_free: Option<bool>,
  
  // NEW: ConsumeShared context (simple string, established pattern)
  pub consume_shared_key: Option<String>,
  
  // NEW: Export coordination (simple enum for range management)
  pub export_coordination: Option<ExportCoordination>,
}

// SIMPLE: Basic coordination info (cacheable for serialization)
#[cacheable]
#[derive(Debug, Clone, Hash, Serialize)]
pub enum ExportCoordination {
  Pending,  // Set by NormalModuleFactory hook
  CommonJS {
    total_exports: usize,           // Available from obj_lit.props.len()
    shared_range: DependencyRange,  // Available from assign_expr.right.span()
  },
  ESM {
    export_count: usize,
    fragment_group_id: String,
  },
}
```

### **3. Parser Enhancement (Tier 3: Coordination Logic)**

```rust
// ENHANCE: CommonJS parser plugin (minimal changes to existing bulk export handling)
impl CommonJsExportsParserPlugin {
  // In existing handle_assign_to_exports method around line 556
  fn handle_bulk_assignment(&mut self, parser: &mut JavascriptParser, assign_expr: &AssignExpr) {
    if let Expr::Object(obj_lit) = &*assign_expr.right {
      let total_exports = obj_lit.props.len(); // Already available!
      
      // CHECK: Use pre-computed ConsumeShared context from NormalModuleFactory
      if let Some(share_key) = &parser.build_meta.consume_shared_key {
        // Update coordination with parser-level details
        parser.build_meta.export_coordination = Some(ExportCoordination::CommonJS {
          total_exports,
          shared_range: assign_expr.right.span().into(),
        });
      }
      
      // EXISTING: Create dependencies as before (no changes to dependency creation)
      for prop in &obj_lit.props {
        // ... existing dependency creation logic unchanged
      }
    }
  }
}

// ENHANCE: ESM parser plugin (coordinate across multiple ESM export dependencies)
impl ESMExportDependencyParserPlugin {
  fn export_specifier(&mut self, parser: &mut JavascriptParser, statement: &ExportSpecifier) {
    // CHECK: Use pre-computed ConsumeShared context from NormalModuleFactory
    if let Some(share_key) = &parser.build_meta.consume_shared_key {
      // Update coordination for ESM exports
      parser.build_meta.export_coordination = Some(ExportCoordination::ESM {
        export_count: parser.exports_info.len(),
        fragment_group_id: format!("esm_exports_{}", parser.module_identifier),
      });
    }
    
    // EXISTING: Create ESM dependencies as before
    // All three ESM dependency types will use the shared BuildMeta context
  }
}
```

### **4. Template Simplification (Remove Module Graph Traversal)**

```rust
// SIMPLIFIED: Use BuildMeta instead of expensive module graph operations
impl DependencyTemplate for CommonJsExportsDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
    let build_meta = &context.module.build_meta;
    
    match &build_meta.consume_shared_key {
      Some(share_key) => {
        // Use pre-computed context - NO module graph traversal needed
        self.render_with_consume_shared_macro(dep, source, share_key, &build_meta.export_coordination)
      }
      None => {
        // EXISTING logic unchanged
        self.render_standard(dep, source, context)
      }
    }
  }
  
  fn render_with_consume_shared_macro(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, share_key: &str, coordination: &Option<ExportCoordination>) {
    match coordination {
      Some(ExportCoordination::CommonJS { total_exports, shared_range }) => {
        // Coordinated macro generation - only last dependency adds endif
        let is_last_dependency = self.is_last_in_bulk_export(dep, *total_exports);
        self.render_commonjs_macro(dep, source, share_key, is_last_dependency);
      }
      _ => {
        // Standard individual export macro
        self.render_individual_macro(dep, source, share_key);
      }
    }
  }
}

// SIMPLIFIED: ESM templates use BuildMeta for all three ESM export dependency types
impl ESMExportSpecifierDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
    let build_meta = &context.module.build_meta;
    
    match &build_meta.consume_shared_key {
      Some(share_key) => {
        // Use pre-computed context - NO get_consume_shared_info() module graph traversal
        let export_content = format!(
          "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
          share_key, 
          dep.export_name,
          dep.export_value
        );
        source.replace(dep.range.start, dep.range.end, &export_content, None);
      }
      None => {
        // EXISTING logic unchanged - standard ESM export
        self.render_standard_esm_export(dep, source, context)
      }
    }
  }
}

// APPLY SAME PATTERN: ESMExportExpressionDependencyTemplate, ESMExportImportedSpecifierDependencyTemplate
// All three ESM templates use identical BuildMeta approach instead of individual detection
```

## Cleanup and Implementation Strategy

### **Phase 1: Selective Cleanup (URGENT)**

```bash
# 1. Revert wrong FlagDependencyUsagePlugin build-time tree-shaking
git checkout main -- crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs

# 2. Remove over-engineered analysis systems (keep basic ones)
rm crates/rspack_plugin_mf/src/sharing/export_usage_analysis.rs  # If overly complex
rm crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs     # If overly complex

# 3. KEEP runtime template PURE annotation changes - ESSENTIAL for runtime tree-shaking
# ✅ DON'T revert: crates/rspack_core/src/dependency/runtime_template.rs

# 4. Keep all runtime tree-shaking support:
# ✅ PURE annotations in templates
# ✅ Side effect detection logic  
# ✅ Core macro generation improvements (CommonJS + ESM)
# ✅ ConsumeShared module metadata tracking
# ✅ ESM export dependency ConsumeShared detection (3 files)
# ✅ ESM fragment-based macro generation
```

### **Phase 2: Implement NormalModuleFactory + BuildMeta Solution**

```rust
// 1. Add BuildMeta fields (crates/rspack_core/src/build_meta.rs)
pub consume_shared_key: Option<String>,
pub export_coordination: Option<ExportCoordination>,

// 2. Extend ConsumeSharedPlugin with NormalModuleFactoryAfterResolve hook
#[plugin_hook(NormalModuleFactoryAfterResolve)]
async fn after_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  if let Some(config) = self.find_consume_config(&data.request) {
    data.build_info.build_meta.consume_shared_key = Some(config.share_key.clone());
  }
}

// 3. Enhance parser coordination (both CommonJS + ESM)
// CommonJS: Handle bulk export coordination
if let Some(share_key) = &parser.build_meta.consume_shared_key {
  parser.build_meta.export_coordination = Some(ExportCoordination::CommonJS { ... });
}
// ESM: Handle multiple export dependencies coordination  
if let Some(share_key) = &parser.build_meta.consume_shared_key {
  parser.build_meta.export_coordination = Some(ExportCoordination::ESM { ... });
}

// 4. Simplify template logic (remove module graph traversal for both systems)
// CommonJS + ESM: Read BuildMeta instead of doing expensive detection
match &build_meta.consume_shared_key {
  Some(share_key) => self.render_with_consume_shared_macro(...),
  None => self.render_standard(...),
}
```

### **Phase 3: Testing and Validation**

1. ✅ Verify macro generation works correctly
2. ✅ Ensure ConsumeShared modules remain untreeshaken  
3. ✅ Test bulk export coordination
4. ✅ Validate performance improvements

### **Phase 4: Documentation Update**

Update docs to reflect:
- ❌ Remove references to wrong tree-shaking changes
- ✅ Focus on macro generation improvements
- ✅ Clarify ConsumeShared architecture principles

## Benefits and Risk Assessment

### **✅ Benefits of Focused Approach**

1. **Solves actual problems** - Fixes macro generation issues
2. **Preserves architecture** - ConsumeShared modules work correctly
3. **Simple implementation** - Minimal code changes
4. **Performance improvement** - Eliminates redundant operations
5. **Easy to understand** - Clear problem→solution mapping

### **🚨 Risks of Current Approach**

1. **Breaks Module Federation** - Wrong tree-shaking behavior
2. **Over-complexity** - Thousands of lines for simple problem
3. **Maintenance burden** - Complex systems hard to maintain
4. **Architectural confusion** - Mixes unrelated concerns

### **✅ Risk Mitigation Strategy**

1. **Revert problematic changes** - Back to known-good state
2. **Focus on core issues** - Only fix macro generation
3. **Use established patterns** - BuildMeta is proven approach
4. **Incremental implementation** - Safe, testable phases

## Summary: Surgical Solution

### **What We Keep**
- ✅ **CommonJS macro generation improvements** - Enhanced dependency + template logic
- ✅ **ESM macro generation improvements** - All 3 export dependency types enhanced
- ✅ BuildMeta metadata enhancements
- ✅ Parser-phase ConsumeShared detection (both CommonJS + ESM)
- ✅ Bulk export coordination logic (CommonJS) + fragment coordination (ESM)
- ✅ **Runtime template PURE annotation changes** - Essential for webpack_require purity
- ✅ **Side effect detection** - Required for runtime tree-shaking
- ✅ Test infrastructure and documentation

### **What We Remove**
- ❌ FlagDependencyUsagePlugin ConsumeShared **build-time** tree-shaking
- ❌ Over-engineered Module Federation analysis systems  
- ❌ Complex build-time export usage tracking
- ❌ **Build-time** removal of ConsumeShared modules (runtime selection must be preserved)

### **Final Result**
- 🎯 **Focused solution** that fixes actual macro problems (CommonJS + ESM)
- 🏗️ **Preserves architecture** of ConsumeShared systems  
- ⚡ **Performance gains** from parser-phase detection (eliminates O(n) template-time traversals)
- 🔧 **Maintainable code** with minimal complexity
- ✅ **Universal approach** - Single BuildMeta pattern works for both CommonJS and ESM
- ✅ **Backward compatible** with existing systems

The solution uses the **perfect existing pattern** (BuildMeta) for **exactly the right purpose** (module-level parser→template metadata) while **removing architectural violations** and **focusing only on the actual problems** that need solving.

### **Next Steps**

1. **Implement cleanup phase** to remove wrong changes
2. **Add BuildMeta fields** with proper serialization
3. **Enhance parser detection** using existing infrastructure
4. **Simplify template logic** to read cached metadata
5. **Test comprehensive scenarios** to ensure correctness