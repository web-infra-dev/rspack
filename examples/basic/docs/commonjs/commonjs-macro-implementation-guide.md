# CommonJS Macro Implementation Guide

## Quick Reference: Files to Modify

Based on the technical analysis, here are the specific file locations and changes needed to enable CommonJS ConsumeShared macro generation:

### 🎯 Primary Target: CommonJS Require Dependency

**File**: `/Users/bytedance/RustroverProjects/rspack/crates/rspack_plugin_javascript/src/dependency/commonjs/common_js_require_dependency.rs`

**Current Code (Lines 140-164)**:
```rust
impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    let dep = dep.as_any().downcast_ref::<CommonJsRequireDependency>().unwrap();
    
    source.replace(
      dep.range.start,
      dep.range.end - 1,
      module_id(ctx.compilation, &dep.id, &dep.request, false).as_str(),
      None,
    );
  }
}
```

**Enhanced Implementation**:
```rust
impl DependencyTemplate for CommonJsRequireDependencyTemplate {
  fn render(&self, dep: &dyn DependencyCodeGeneration, source: &mut TemplateReplaceSource, ctx: &mut TemplateContext) {
    let dep = dep.as_any().downcast_ref::<CommonJsRequireDependency>().unwrap();
    
    // Add ConsumeShared detection
    let consume_shared_info = Self::get_consume_shared_info(dep, &ctx.module_graph);
    let module_reference = module_id(ctx.compilation, &dep.id, &dep.request, false);
    
    let replacement = if let Some(share_key) = consume_shared_info {
      format!(
        "/* @common:if [condition=\"treeShake.{}.default\"] */ {} /* @common:endif */",
        share_key, module_reference
      )
    } else {
      module_reference.to_string()
    };
    
    source.replace(dep.range.start, dep.range.end - 1, &replacement, None);
  }
}

impl CommonJsRequireDependencyTemplate {
  fn get_consume_shared_info(dep: &CommonJsRequireDependency, module_graph: &ModuleGraph) -> Option<String> {
    // Check if target module is ConsumeShared
    if let Some(module) = module_graph.module_by_dependency_id(&dep.id) {
      if let Some(consume_shared_module) = module.downcast_ref::<ConsumeSharedModule>() {
        return Some(consume_shared_module.get_share_key().to_string());
      }
    }
    None
  }
}
```

### 🔧 Required Imports Addition

Add these imports to the CommonJS require dependency file:
```rust
use rspack_core::{ModuleGraph, ConsumeSharedModule};
use rspack_plugin_mf::ConsumeSharedModule; // If in separate crate
```

### 📋 Step-by-Step Implementation

#### Step 1: Add ConsumeShared Detection Helper
```rust
// Add to common_js_require_dependency.rs
fn get_consume_shared_info(
  dependency_id: &DependencyId,
  module_graph: &ModuleGraph,
) -> Option<String> {
  // Get the target module for this dependency
  if let Some(module) = module_graph.module_by_dependency_id(dependency_id) {
    // Check if it's a ConsumeShared module
    if module.module_type() == &ModuleType::ConsumeShared {
      // Extract share key from ConsumeShared module
      if let Some(consume_shared) = module.downcast_ref::<ConsumeSharedModule>() {
        return Some(consume_shared.get_share_key().to_string());
      }
    }
  }
  None
}
```

#### Step 2: Update Template Rendering
Replace the simple `source.replace()` call with macro-aware replacement:

```rust
let base_replacement = module_id(ctx.compilation, &dep.id, &dep.request, false);

let final_replacement = if let Some(share_key) = get_consume_shared_info(&dep.id, &ctx.module_graph) {
  format!(
    "/* @common:if [condition=\"treeShake.{}.default\"] */ {} /* @common:endif */",
    share_key, base_replacement
  )
} else {
  base_replacement.to_string()
};

source.replace(dep.range.start, dep.range.end - 1, &final_replacement, None);
```

#### Step 3: Test Implementation
Create test CommonJS modules that should generate macros:

```javascript
// In rspack.config.cjs - ensure CommonJS modules are shared
shared: {
  "./cjs-modules/legacy-utils.js": {
    singleton: true,
    shareKey: "legacy-utils-lib"
  }
}

// In index.js - access via require (should generate macros after fix)
const legacyUtils = require("./cjs-modules/legacy-utils.js");
```

Expected output after implementation:
```javascript
// Instead of:
const legacyUtils = __webpack_require__(42);

// Should generate:
const legacyUtils = /* @common:if [condition="treeShake.legacy-utils-lib.default"] */ __webpack_require__(42) /* @common:endif */;
```

### 🎯 Alternative: Minimal Proof of Concept

For a quick validation, add just the macro generation without full ConsumeShared detection:

```rust
// Minimal change to test macro generation
let module_reference = module_id(ctx.compilation, &dep.id, &dep.request, false);

// Hardcode for testing - replace with actual detection later
let is_shared_module = dep.request.contains("cjs-modules");
let replacement = if is_shared_module {
  format!(
    "/* @common:if [condition=\"treeShake.test.default\"] */ {} /* @common:endif */",
    module_reference
  )
} else {
  module_reference.to_string()
};
```

### 🧪 Testing Changes

#### Build and Test:
```bash
cd /Users/bytedance/RustroverProjects/rspack/examples/basic
npm run build
```

#### Validate Macro Generation:
```bash
# Check for macros in CommonJS chunks
grep -n "@common:if" dist/cjs-modules_*.js

# Should find macro annotations if implementation works
```

#### Run Test Suite:
```bash
npm test
node test-cjs-macro-check.js
```

### 📊 Expected Results

**Before Implementation:**
```javascript
// cjs-modules_legacy-utils_js.js
const legacyUtils = __webpack_require__(42);
```

**After Implementation:**
```javascript
// cjs-modules_legacy-utils_js.js  
const legacyUtils = /* @common:if [condition="treeShake.legacy-utils-lib.default"] */ __webpack_require__(42) /* @common:endif */;
```

### 🔍 Debugging Tips

1. **Add Debug Logging**:
```rust
println!("🔍 DEBUG: Checking ConsumeShared for request: {}", dep.request);
if let Some(share_key) = consume_shared_info {
  println!("✅ Found ConsumeShared with key: {}", share_key);
}
```

2. **Check Module Graph**:
Ensure CommonJS modules are actually becoming ConsumeShared modules in the first place.

3. **Validate Template Execution**:
Check if the template render method is being called for your CommonJS dependencies.

### 🎯 Success Criteria

1. ✅ CommonJS require calls generate `@common:if` macros
2. ✅ Macros contain correct share keys  
3. ✅ Existing CommonJS functionality preserved
4. ✅ ESM macro generation still works
5. ✅ Test suite passes

This implementation focuses on the minimal changes needed to prove that CommonJS macro generation is possible while maintaining system stability.