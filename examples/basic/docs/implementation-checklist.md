# ConsumeShared Macro Solution Implementation Checklist

## Phase 1: Critical Cleanup (URGENT)

### 1.1 Revert Wrong Changes
- [ ] **Check FlagDependencyUsagePlugin changes**: `crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`
  - [ ] Look for ConsumeShared build-time tree-shaking logic
  - [ ] Revert if found: `git checkout main -- crates/rspack_plugin_javascript/src/plugin/flag_dependency_usage_plugin.rs`

### 1.2 Remove Over-Engineered Systems (If Overly Complex)
- [ ] **Check export_usage_analysis.rs**: `crates/rspack_plugin_mf/src/sharing/export_usage_analysis.rs`
  - [ ] Evaluate if file is overly complex (>1000 lines)
  - [ ] Remove if excessive: `rm crates/rspack_plugin_mf/src/sharing/export_usage_analysis.rs`
- [ ] **Check share_usage_plugin.rs**: `crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs`  
  - [ ] Evaluate if file is overly complex (>1000 lines)
  - [ ] Remove if excessive: `rm crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs`

### 1.3 Preserve Essential Changes
- [ ] **KEEP runtime template PURE annotations**: `crates/rspack_core/src/dependency/runtime_template.rs`
- [ ] **KEEP CommonJS macro enhancements**: `crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_exports_dependency.rs`
- [ ] **KEEP ESM macro enhancements**: 
  - [ ] `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_expression_dependency.rs`
  - [ ] `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_specifier_dependency.rs`
  - [ ] `crates/rspack_plugin_javascript/src/dependency/esm/esm_export_imported_specifier_dependency.rs`

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

## Phase 3: Extend ConsumeSharedPlugin

### 3.1 Add NormalModuleFactoryAfterResolve Hook
- [ ] **Edit ConsumeSharedPlugin**: `crates/rspack_plugin_mf/src/sharing/consume_shared_plugin.rs`
- [ ] **Add hook registration** in `apply()` method around line 769-796:
```rust
compilation.add_runtime_requirement_dependency(RuntimeGlobals::SHARE_SCOPE_MAP);
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

## Phase 7: Update ESM Templates

### 7.1 Update All Three ESM Templates
- [ ] **ESMExportExpressionDependencyTemplate** (default exports)
- [ ] **ESMExportSpecifierDependencyTemplate** (named exports)  
- [ ] **ESMExportImportedSpecifierDependencyTemplate** (re-exports)

### 7.2 Replace get_consume_shared_info() Calls
- [ ] **Remove module graph traversal** in each template
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