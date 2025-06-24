# ConsumeShared Macro Solution Implementation Checklist

## Phase 1: Critical Cleanup (URGENT)

### 1.1 Revert Wrong Changes

- [x] **Check FlagDependencyUsagePlugin changes**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`
  - [x] Look for ConsumeShared build-time tree-shaking logic
  - [x] ✅ **No ConsumeShared logic found** - File contains only general dependency usage flagging, no revert needed

### 1.2 Remove Over-Engineered Systems (If Overly Complex)

- [x] **Check export_usage_analysis.rs**: `crates/rspack_plugin_mf/src/sharing/export_usage_analysis.rs`
  - [x] Evaluate if file is overly complex (>1000 lines)
  - [x] ✅ **1098 lines - OVERLY COMPLEX but kept** - Should be removed per solution design
  - [x] ✅ **Files identified but NOT removing** - Per user instruction, keeping complex files
- [x] **Check share_usage_plugin.rs**: `crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs`
  - [x] Evaluate if file is overly complex (>1000 lines)
  - [x] ✅ **1036 lines - OVERLY COMPLEX but kept** - Per user instruction, not removing
  - [x] ✅ **Files identified but NOT removing** - Per user instruction, keeping complex files

### 1.3 Preserve Essential Changes

- [x] **KEEP runtime template PURE annotations**: `crates/rspack_core/src/dependency/runtime_template.rs`
  - [x] ✅ **ConsumeShared PURE annotations verified** - Found essential runtime tree-shaking logic:
    - Line 419: `"/* #__PURE__ */ "` annotations for ConsumeShared descendant imports
    - Line 433: `"/*#__PURE__*/"` annotations for ESM default exports
    - Line 698: `"/* #__PURE__ */"` annotations for module factories
    - ConsumeShared descendant detection functions (lines 720-768) are **essential**
- [x] **KEEP CommonJS macro enhancements**: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_exports_dependency.rs`
  - [x] ✅ **Enhanced ConsumeShared detection** - `detect_consume_shared_context()` function (lines 425-453)
  - [x] ✅ **Macro generation logic** - Tree-shaking macro wrapping in `render_expression_export()` (lines 574-610)
  - [x] ✅ **Value range handling** - Comprehensive macro wrapping with proper endif placement
  - [x] ✅ **Export coordination** - Enhanced export statement rendering with error handling
- [x] **KEEP ESM macro enhancements**:
  - [x] ✅ `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_expression_dependency.rs`
    - [x] **ConsumeShared detection**: `get_consume_shared_info()` function (lines 17-40)
    - [x] **Default export macros**: Tree-shaking macro generation (lines 260-265)
  - [x] ✅ `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_specifier_dependency.rs`
    - [x] **ConsumeShared detection**: `get_consume_shared_info()` method (line 52)
    - [x] **Named export macros**: ConsumeShared macro usage (line 241)
  - [x] ✅ `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_imported_specifier_dependency.rs`
    - [x] **ConsumeShared detection**: `get_consume_shared_info()` method (line 132)
    - [x] **Re-export macros**: ConsumeShared macro usage (lines 948, 990)

## Phase 2: Extend BuildMeta Structure

### 2.1 Add ConsumeShared Fields

- [ ] **Edit BuildMeta**: `crates/rspack_core/src/build_meta.rs`
  - [ ] Add `pub consume_shared_key: Option<String>,`
  - [ ] Add `pub export_coordination: Option<ExportCoordination>,`

### 2.2 Create ExportCoordination Enum

- [ ] **Add ExportCoordination enum** in same file:

```rust
#[cacheable]
#[derive(Debug, Clone, Hash, Serialize)]
pub enum ExportCoordination {
  Pending,  // Set by NormalModuleFactory hook
  CommonJS {
    total_exports: usize,
    shared_range: DependencyRange,
  },
  ESM {
    export_count: usize,
    fragment_group_id: String,
  },
}
```

### 2.3 Update Imports

- [ ] Add necessary imports for `DependencyRange` and other types
- [ ] Ensure `#[cacheable]` and serialization attributes are correct
- [ ] Add `serde::Serialize` import if not already present
- [ ] Add `rspack_cacheable::cacheable` import if not already present

## Phase 3: Extend ConsumeSharedPlugin

### 3.1 Add NormalModuleFactoryAfterResolve Hook

- [ ] **Edit ConsumeSharedPlugin**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- [ ] **Add hook registration** in `apply()` method around line 769-796:

```rust
compilation
  .normal_module_factory()
  .add_hook(|normal_module_factory| {
    normal_module_factory.after_resolve.tap(self.after_resolve_tap())
  });
```

### 3.2 Implement Hook Handler

- [ ] **Add after_resolve method**:

```rust
#[plugin_hook(NormalModuleFactoryAfterResolve)]
async fn after_resolve(&self, data: &mut ModuleFactoryCreateData) -> Result<Option<bool>> {
  if let Some(config) = self.find_consume_config(&data.request) {
    data.build_info.build_meta.consume_shared_key = Some(config.share_key.clone());
    data.build_info.build_meta.export_coordination = Some(ExportCoordination::Pending);
  }
  Ok(None)
}
```

### 3.3 Leverage Existing Detection Logic

- [ ] **Use existing find_consume_config logic** from lines 715-741
- [ ] **Reuse MatchedConsumes** pattern matching (unresolved, prefixed, resolved)
- [ ] **Add necessary imports** for hook registration and BuildMeta types
- [ ] **Create find_consume_config helper method** that wraps existing MatchedConsumes logic

## Phase 4: Enhance CommonJS Parser

### 4.1 Update Bulk Export Handling

- [ ] **Edit CommonJsExportsParserPlugin**: `crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs`
- [ ] **Find handle_assign_to_exports method** around line 556
- [ ] **Add BuildMeta coordination**:

```rust
if let Expr::Object(obj_lit) = &*assign_expr.right {
  let total_exports = obj_lit.props.len();

  if let Some(share_key) = &parser.build_meta.consume_shared_key {
    parser.build_meta.export_coordination = Some(ExportCoordination::CommonJS {
      total_exports,
      shared_range: assign_expr.right.span().into(),
    });
  }

  // EXISTING: Create dependencies as before (no changes)
}
```

## Phase 5: Enhance ESM Parser

### 5.1 Update ESM Export Handling

- [ ] **Edit ESMExportDependencyParserPlugin**: `crates/rspack_plugin_javascript/src/parser_plugin/esm_export_dependency_parser_plugin.rs`
- [ ] **Add BuildMeta coordination** in export handlers:

```rust
if let Some(share_key) = &parser.build_meta.consume_shared_key {
  parser.build_meta.export_coordination = Some(ExportCoordination::ESM {
    export_count: parser.exports_info.len(),
    fragment_group_id: format!("esm_exports_{}", parser.module_identifier),
  });
}
```

## Phase 6: Update CommonJS Templates

### 6.1 Simplify Template Detection

- [ ] **Edit CommonJsExportsDependencyTemplate**: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_exports_dependency.rs`
- [ ] **Replace template-time detection** with BuildMeta reading:

```rust
fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
  let build_meta = &context.module.build_meta;

  match &build_meta.consume_shared_key {
    Some(share_key) => {
      self.render_with_consume_shared_macro(dep, source, share_key, &build_meta.export_coordination)
    }
    None => {
      self.render_standard(dep, source, context)
    }
  }
}
```

### 6.2 Add Coordinated Macro Generation

- [ ] **Implement range coordination** for bulk exports to prevent stacked endif tags
- [ ] **Fix export value generation** (use `foo` not `module.exports.foo` in object literals)
- [ ] **Add render_with_consume_shared_macro method** to handle coordinated generation
- [ ] **Add is_last_in_bulk_export logic** to determine when to add endif tag

## Phase 7: Update ESM Templates

### 7.1 Update All Three ESM Templates

- [ ] **ESMExportExpressionDependencyTemplate** (default exports)
- [ ] **ESMExportSpecifierDependencyTemplate** (named exports)
- [ ] **ESMExportImportedSpecifierDependencyTemplate** (re-exports)

### 7.2 Replace get_consume_shared_info() Calls

- [ ] **Remove module graph traversal** in each template
- [ ] **Remove find_consume_shared_recursive() methods**
- [ ] **Use BuildMeta context** instead:

```rust
match &build_meta.consume_shared_key {
  Some(share_key) => {
    let export_content = format!(
      "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
      share_key, dep.export_name, dep.export_value
    );
    source.replace(dep.range.start, dep.range.end, &export_content, None);
  }
  None => {
    self.render_standard_esm_export(dep, source, context)
  }
}
```

- [ ] **Update ESMExportInitFragment generation** to use BuildMeta context

## Testing & Validation

### Test CommonJS Scenarios

- [ ] **Bulk exports**: `module.exports = {a, b, c}`
- [ ] **Individual exports**: `module.exports.foo = bar`
- [ ] **ConsumeShared modules**: Verify macro generation
- [ ] **Regular modules**: Verify no macro generation

### Test ESM Scenarios

- [ ] **Named exports**: `export {a, b, c}`
- [ ] **Default exports**: `export default foo`
- [ ] **Re-exports**: `export {a} from './module'`
- [ ] **ConsumeShared modules**: Verify macro generation

### Performance Validation

- [ ] **Verify BuildMeta caching**: No repeated module graph traversals
- [ ] **Check macro output**: Proper `@common:if` / `@common:endif` structure
- [ ] **Validate coordination**: No stacked endif tags in CommonJS
- [ ] **Validate performance**: Eliminate O(n) template-time operations
- [ ] **Test serialization**: Ensure BuildMeta fields serialize correctly with #[cacheable]

### Integration Testing

- [ ] **Test mixed scenarios**: Modules with both ConsumeShared and regular exports
- [ ] **Test nested scenarios**: ConsumeShared modules importing other ConsumeShared modules
- [ ] **Test edge cases**: Empty exports, dynamic exports, conditional exports
- [ ] **Test build pipeline**: Ensure no compilation errors after changes

## Commit Strategy

### Individual Commits

- [ ] **Commit Phase 1**: "cleanup: revert architectural violations in ConsumeShared handling"
- [ ] **Commit Phase 2**: "feat: extend BuildMeta with ConsumeShared coordination fields"
- [ ] **Commit Phase 3**: "feat: add early ConsumeShared detection in NormalModuleFactory hook"
- [ ] **Commit Phase 4-5**: "feat: enhance parsers to use BuildMeta ConsumeShared context"
- [ ] **Commit Phase 6-7**: "feat: optimize templates to use cached BuildMeta instead of detection"

### Final Integration Commit

- [ ] **Integration testing**: Verify all systems work together
- [ ] **Final commit**: "feat: complete ConsumeShared macro optimization with BuildMeta pattern"

---

**Implementation Notes:**

- Keep PURE annotations for runtime tree-shaking
- Preserve all existing ConsumeShared functionality
- Only optimize detection and coordination
- Maintain backward compatibility
- Follow established Rspack patterns

## Missing Critical Implementation Details

### Hook Registration Pattern

- [ ] **Use #[plugin_hook] attribute macro** for proper hook registration
- [ ] **Follow existing hook patterns** in ConsumeSharedPlugin apply() method
- [ ] **Ensure proper async coordination** with existing hooks

### BuildMeta Access Patterns

- [ ] **Access via context.module.build_meta** in templates
- [ ] **Access via parser.build_meta** in parser plugins
- [ ] **Access via data.build_info.build_meta** in NormalModuleFactory hooks

### Error Handling

- [ ] **Add proper Result<> return types** for all async methods
- [ ] **Handle Option<> unwrapping safely** for BuildMeta fields
- [ ] **Add validation** for ExportCoordination enum variants

### Import Requirements

- [ ] **Add BuildMeta imports** to ConsumeSharedPlugin
- [ ] **Add ExportCoordination imports** to parser plugins
- [ ] **Add DependencyRange imports** for coordination struct

## Pre-Implementation Analysis (DO FIRST)

### Codebase Analysis Required

- [ ] **Examine existing FlagDependencyUsagePlugin** to understand what needs reverting
- [ ] **Check current export_usage_analysis.rs complexity** (count lines, assess if >1000 lines)
- [ ] **Check current share_usage_plugin.rs complexity** (count lines, assess if >1000 lines)
- [ ] **Review existing ConsumeSharedPlugin structure** to understand hook integration points
- [ ] **Find actual method names** in CommonJsExportsParserPlugin (may not be handle_assign_to_exports)
- [ ] **Find actual method names** in ESMExportDependencyParserPlugin
- [ ] **Verify BuildMeta structure location** and existing fields
- [ ] **Check existing template method signatures** for proper context parameter access

### Missing Solution Components from Design Document

- [ ] **Add render_with_consume_shared_macro implementation** with proper coordination logic
- [ ] **Add is_last_in_bulk_export method** to determine endif placement
- [ ] **Add render_standard fallback methods** for non-ConsumeShared modules
- [ ] **Add template context access helpers** for BuildMeta retrieval
- [ ] **Add proper dependency ordering logic** for coordinated macro generation

### Template Method Implementations Missing

- [ ] **CommonJS render_with_consume_shared_macro**:
  ```rust
  fn render_with_consume_shared_macro(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, share_key: &str, coordination: &Option<ExportCoordination>) {
    match coordination {
      Some(ExportCoordination::CommonJS { total_exports, shared_range }) => {
        let is_last_dependency = self.is_last_in_bulk_export(dep, *total_exports);
        self.render_commonjs_macro(dep, source, share_key, is_last_dependency);
      }
      _ => {
        self.render_individual_macro(dep, source, share_key);
      }
    }
  }
  ```
- [ ] **Add render_individual_macro method** for single exports
- [ ] **Add render_commonjs_macro method** with coordination
- [ ] **Add render_standard_esm_export methods** for all three ESM templates

## Missing Implementation Items (Add to Phases Above)

### Phase 2 Additional Items
- [ ] **Verify BuildMeta struct location** - Confirm file structure and existing fields
- [ ] **Check existing serialization** - Ensure compatibility with current BuildMeta serialization
- [ ] **Validate DependencyRange type** - Confirm import path and usage patterns

### Phase 3 Additional Items  
- [ ] **Study existing hook registration patterns** in ConsumeSharedPlugin apply() method
- [ ] **Verify ModuleFactoryCreateData structure** - Confirm data parameter access patterns
- [ ] **Check plugin trait implementation** - Ensure proper async trait bounds

### Phase 4-5 Additional Items
- [ ] **Verify parser method names** - Find actual method names in CommonJS/ESM parser plugins
- [ ] **Check parser.build_meta access** - Confirm BuildMeta is accessible in parser context
- [ ] **Validate span conversion** - Ensure assign_expr.right.span().into() works with DependencyRange

### Phase 6-7 Additional Items
- [ ] **Implement dependency ordering detection** - Logic to determine is_last_in_bulk_export
- [ ] **Add proper export value extraction** - Correct variable references in object literal context
- [ ] **Create template method helpers**:
  - [ ] render_commonjs_macro(dep, source, share_key, is_last)
  - [ ] render_individual_macro(dep, source, share_key)  
  - [ ] render_standard(dep, source, context) - fallback for non-ConsumeShared
  - [ ] render_standard_esm_export(dep, source, context) - ESM fallback

### Compilation and Error Handling
- [ ] **Add comprehensive error handling** for all Option<> unwrapping
- [ ] **Verify compilation after each phase** - Ensure no build errors introduced
- [ ] **Add proper import statements** for all new types and traits
- [ ] **Test serialization compatibility** - Ensure #[cacheable] works correctly
