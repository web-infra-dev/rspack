# CommonJS Export Analysis and Dependencies

## Overview

Rspack's CommonJS export system handles the complex semantics of CommonJS modules, including `module.exports`, `exports`, and `this` assignment patterns. The system provides comprehensive tracking and optimization capabilities while maintaining compatibility with Node.js module semantics.

## Core CommonJS Export Components

### 1. CommonJsExportsDependency Structure

```rust
pub struct CommonJsExportsDependency {
    id: DependencyId,
    range: DependencyRange,              // Source code position of export assignment
    value_range: Option<DependencyRange>, // Position of assigned value
    base: ExportsBase,                   // Type of export base (exports, module.exports, this)
    names: Vec<Atom>,                    // Property path (e.g., ["foo", "bar"] for exports.foo.bar)
}
```

### 2. Export Base Types

The system handles multiple CommonJS export patterns:

```rust
#[derive(Debug, Clone, Copy)]
pub enum ExportsBase {
    Exports,                    // exports.foo = value
    ModuleExports,             // module.exports.foo = value  
    This,                      // this.foo = value
    DefinePropertyExports,     // Object.defineProperty(exports, ...)
    DefinePropertyModuleExports, // Object.defineProperty(module.exports, ...)
    DefinePropertyThis,        // Object.defineProperty(this, ...)
}
```

**Base Type Classification Methods**:
```rust
impl ExportsBase {
    pub const fn is_exports(&self) -> bool {
        matches!(self, Self::Exports | Self::DefinePropertyExports)
    }
    
    pub const fn is_module_exports(&self) -> bool {
        matches!(self, Self::ModuleExports | Self::DefinePropertyModuleExports)
    }
    
    pub const fn is_this(&self) -> bool {
        matches!(self, Self::This | Self::DefinePropertyThis)
    }
    
    pub const fn is_expression(&self) -> bool {
        matches!(self, Self::Exports | Self::ModuleExports | Self::This)
    }
    
    pub const fn is_define_property(&self) -> bool {
        matches!(self, Self::DefinePropertyExports | Self::DefinePropertyModuleExports | Self::DefinePropertyThis)
    }
}
```

## Export Information Generation

### 1. Export Specification Creation

```rust
impl Dependency for CommonJsExportsDependency {
    fn get_exports(&self, _mg: &ModuleGraph, _mg_cache: &ModuleGraphCacheArtifact) -> Option<ExportsSpec> {
        let vec = vec![ExportNameOrSpec::ExportSpec(ExportSpec {
            name: self.names[0].clone(),
            can_mangle: Some(false), // CommonJS object properties may not be mangled
            ..Default::default()
        })];
        
        Some(ExportsSpec {
            exports: ExportsOfExportsSpec::Names(vec),
            ..Default::default()
        })
    }
}
```

**Key Characteristics**:
- **Non-mangleable**: CommonJS exports typically cannot be mangled due to dynamic property access patterns
- **Property-based**: Each export is treated as an object property assignment
- **Runtime semantics**: Maintains Node.js-compatible behavior

## Template Rendering and Code Generation

### 1. Base Expression Handling

The rendering process varies based on the export base type:

```rust
impl DependencyTemplate for CommonJsExportsDependencyTemplate {
    fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, context: &mut TemplateContext) {
        // 1. Determine runtime base expression
        let base = if dep.base.is_exports() {
            runtime_requirements.insert(RuntimeGlobals::EXPORTS);
            exports_argument.to_string()
        } else if dep.base.is_module_exports() {
            runtime_requirements.insert(RuntimeGlobals::MODULE);
            format!("{module_argument}.exports")
        } else if dep.base.is_this() {
            runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
            "this".to_string()
        };
        
        // 2. Handle expression vs defineProperty patterns
        if dep.base.is_expression() {
            self.render_expression_assignment(dep, source, context, base);
        } else if dep.base.is_define_property() {
            self.render_define_property(dep, source, context, base);
        }
    }
}
```

### 2. Expression Assignment Rendering

For direct property assignments (`exports.foo = value`):

```rust
fn render_expression_assignment(&self, dep: &CommonJsExportsDependency, source: &mut TemplateReplaceSource, context: &mut TemplateContext, base: String) {
    // Get used name after tree-shaking analysis
    let used = ExportsInfoGetter::get_used_name(
        GetUsedNameParam::WithNames(&exports_info),
        *runtime,
        &dep.names,
    );
    
    if let Some(UsedName::Normal(used)) = used {
        let export_assignment = format!("{}{}", base, property_access(&used, 0));
        let export_name = used.iter().map(|a| a.as_str()).collect::<Vec<_>>().join(".");
        
        // Generate conditional exports for module federation
        let export_content = if let Some(ref share_key) = consume_shared_info {
            format!(
                "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
                share_key, export_name, export_assignment
            )
        } else {
            format!(
                "/* EXPORT_BEGIN:{} */ {} /* EXPORT_END:{} */",
                export_name, export_assignment, export_name
            )
        };
        
        source.replace(dep.range.start, dep.range.end, &export_content, None);
    } else {
        // Handle unused exports
        let placeholder_var = format!("__webpack_{}_export__", if is_inlined { "inlined" } else { "unused" });
        source.replace(dep.range.start, dep.range.end, &placeholder_var, None);
        
        // Add placeholder variable declaration
        init_fragments.push(NormalInitFragment::new(
            format!("var {placeholder_var};\n"),
            InitFragmentStage::StageConstants,
            0,
            InitFragmentKey::CommonJsExports(placeholder_var),
            None,
        ).boxed());
    }
}
```

### 3. DefineProperty Handling

For `Object.defineProperty` patterns:

```rust
fn render_define_property(&self, dep: &CommonJsExportsDependency, source: &mut TemplateReplaceSource, context: &mut TemplateContext, base: String) {
    if let Some(value_range) = &dep.value_range {
        if let Some(UsedName::Normal(used)) = used {
            if !used.is_empty() {
                let export_name = used.last().unwrap();
                
                // Generate defineProperty call with conditional wrapping
                let define_property_start = if let Some(ref share_key) = consume_shared_info {
                    format!(
                        "/* @common:if [condition=\"treeShake.{}.{}\"] */ Object.defineProperty({}{}, {}, (",
                        share_key, export_name, base, property_access(used[0..used.len() - 1].iter(), 0),
                        serde_json::to_string(&used.last()).expect("Unexpected render define property base")
                    )
                } else {
                    format!(
                        "/* EXPORT_BEGIN:{} */ Object.defineProperty({}{}, {}, (",
                        export_name, base, property_access(used[0..used.len() - 1].iter(), 0),
                        serde_json::to_string(&used.last()).expect("Unexpected render define property base")
                    )
                };
                
                source.replace(dep.range.start, value_range.start, &define_property_start, None);
                
                let define_property_end = if consume_shared_info.is_some() {
                    ")) /* @common:endif */"
                } else {
                    &format!(")) /* EXPORT_END:{} */", export_name)
                };
                
                source.replace(value_range.end, dep.range.end, define_property_end, None);
            }
        } else {
            // Handle unused defineProperty exports
            init_fragments.push(NormalInitFragment::new(
                "var __webpack_unused_export__;\n".to_string(),
                InitFragmentStage::StageConstants,
                0,
                InitFragmentKey::CommonJsExports("__webpack_unused_export__".to_owned()),
                None,
            ).boxed());
            
            source.replace(dep.range.start, value_range.start, "__webpack_unused_export__ = (", None);
            source.replace(value_range.end, dep.range.end, ")", None);
        }
    }
}
```

## Module Federation Integration

### 1. ConsumeShared Module Detection

The system includes sophisticated detection for module federation scenarios:

```rust
fn get_consume_shared_context(&self, module_graph: &ModuleGraph, dep_id: &DependencyId) -> Option<String> {
    if let Some(parent_module_id) = module_graph.get_parent_module(dep_id) {
        if let Some(parent_module) = module_graph.module_by_identifier(parent_module_id) {
            if parent_module.module_type() == &ModuleType::ConsumeShared {
                let trait_result = parent_module.get_consume_shared_key();
                
                // Enhanced debugging for ConsumeShared modules
                tracing::debug!(
                    "[RSPACK_EXPORT_DEBUG:CJS_RENDER_DETAILED] Module: {:?}, Type: {:?}, Layer: {:?}, Names: {:?}, Base: {:?}",
                    module_identifier, module.module_type(), module.get_layer(), dep.names, dep.base
                );
                
                return trait_result;
            }
        }
    }
    None
}
```

### 2. Conditional Export Generation

For tree-shaking in module federation contexts:

```rust
fn generate_conditional_export(&self, share_key: &str, export_name: &str, assignment: &str) -> String {
    format!(
        "/* @common:if [condition=\"treeShake.{}.{}\"] */ {} /* @common:endif */",
        share_key, export_name, assignment
    )
}
```

## Runtime Requirements Management

### 1. Runtime Globals Insertion

The system manages runtime requirements based on export patterns:

```rust
fn insert_runtime_requirements(&self, base: &ExportsBase, runtime_requirements: &mut RuntimeRequirements) {
    match base {
        ExportsBase::Exports | ExportsBase::DefinePropertyExports => {
            runtime_requirements.insert(RuntimeGlobals::EXPORTS);
        }
        ExportsBase::ModuleExports | ExportsBase::DefinePropertyModuleExports => {
            runtime_requirements.insert(RuntimeGlobals::MODULE);
        }
        ExportsBase::This | ExportsBase::DefinePropertyThis => {
            runtime_requirements.insert(RuntimeGlobals::THIS_AS_EXPORTS);
        }
    }
}
```

### 2. Property Access Generation

Safe property access generation for nested exports:

```rust
fn generate_property_access(names: &[Atom], start_index: usize) -> String {
    names[start_index..]
        .iter()
        .map(|name| {
            if is_valid_identifier(name) {
                format!(".{}", name)
            } else {
                format!("[{}]", serde_json::to_string(name).unwrap())
            }
        })
        .collect::<String>()
}
```

## Export Usage Analysis Integration

### 1. Used Name Resolution

```rust
fn resolve_used_names(&self, module_graph: &ModuleGraph, names: &[Atom], runtime: Option<&RuntimeSpec>) -> Option<UsedName> {
    if names.is_empty() {
        let exports_info = ExportsInfoGetter::prefetch_used_info_without_name(
            &module_graph.get_exports_info(&module.identifier()),
            module_graph,
            runtime,
            false,
        );
        ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithoutNames(&exports_info),
            runtime,
            names,
        )
    } else {
        let exports_info = module_graph.get_prefetched_exports_info(
            &module.identifier(),
            PrefetchExportsInfoMode::NamedNestedExports(names),
        );
        ExportsInfoGetter::get_used_name(
            GetUsedNameParam::WithNames(&exports_info),
            runtime,
            names,
        )
    }
}
```

### 2. Tree-Shaking Integration

The system integrates with flag dependency plugins:

```rust
// Export information flows from analysis to code generation:
// 1. FlagDependencyExportsPlugin -> identifies provided exports
// 2. FlagDependencyUsagePlugin -> tracks export usage
// 3. CommonJsExportsDependency -> generates optimized code based on usage
```

## Performance Optimizations

### 1. Lazy Fragment Creation

```rust
// Only create init fragments when needed
if unused_exports.is_empty() {
    // No fragments needed
} else {
    init_fragments.push(create_unused_export_fragment(unused_exports));
}
```

### 2. Efficient Property Access

```rust
// Use property_access utility for optimized property chain generation
let property_chain = property_access(&used_names, 0);
// Generates: .foo.bar or ["foo"]["bar"] based on identifier validity
```

### 3. Runtime Requirement Optimization

```rust
// Only insert required runtime globals
match base_type {
    ExportsBase::Exports => runtime_requirements.insert(RuntimeGlobals::EXPORTS),
    ExportsBase::ModuleExports => runtime_requirements.insert(RuntimeGlobals::MODULE),
    // ... other cases
}
```

## Error Handling and Edge Cases

### 1. Missing Value Range

```rust
if let Some(value_range) = &dep.value_range {
    // Handle defineProperty with value
} else {
    panic!("Define property need value range");
}
```

### 2. Invalid Export Names

```rust
// Handle special characters in export names
let property_name = if is_valid_identifier(&name) {
    name.to_string()
} else {
    serde_json::to_string(&name).expect("Invalid export name")
};
```

### 3. Unused Export Handling

```rust
// Generate placeholder for unused exports to maintain side effects
let placeholder = if is_inlined {
    "__webpack_inlined_export__"
} else {
    "__webpack_unused_export__"
};
```

## Debugging and Diagnostics

### 1. Comprehensive Logging

```rust
tracing::debug!(
    "[RSPACK_EXPORT_DEBUG:CJS_ASSIGNMENT] Module: {:?}, Export: {}, Assignment: {}, Used: {:?}, Range: {:?}",
    module_identifier, export_name, export_assignment, used, dep.range
);
```

### 2. Module Build Information

```rust
// Log module context for debugging
if let Some(normal_module) = module.as_normal_module() {
    tracing::debug!(
        "[RSPACK_EXPORT_DEBUG:CJS_MODULE_BUILD_INFO] Request: {:?}, UserRequest: {:?}, RawRequest: {:?}",
        normal_module.request(), normal_module.user_request(), normal_module.raw_request()
    );
}
```

## Conclusion

The CommonJS export system in rspack provides comprehensive support for all CommonJS export patterns while maintaining optimization capabilities through integration with the flag dependency plugin system. The system handles complex scenarios including module federation, tree-shaking, and property mangling while preserving Node.js compatibility. The sophisticated template rendering system ensures optimal code generation for various export patterns, from simple property assignments to complex `Object.defineProperty` calls.