# CommonJS Macro Implementation Analysis

## 🔍 Current Status

### What We Found

1. **CommonJS dependencies ALREADY have macro generation code**:
   ```rust
   // In common_js_exports_dependency.rs:599-608
   let define_property_start = if let Some(ref share_key) = consume_shared_info {
     format!(
       "/* @common:if [condition=\"treeShake.{}.{}\"] */ Object.defineProperty({}{}, {}, (",
       share_key, export_name, base_expression, property_path,
       serde_json::to_string(export_name)?
     )
   } else {
     format!("Object.defineProperty({}{}, {}, (", ...)
   };
   ```

2. **CommonJS modules are configured as ProvideShared** (confirmed in build output):
   ```javascript
   // In main.js sharing setup:
   { name: "data-processor-lib", version: "0", factory: () => ... }
   { name: "legacy-utils-lib", version: "0", factory: () => ... }
   ```

3. **CommonJS modules accessed via require() don't become ConsumeShared**:
   - Direct `require("./cjs-modules/legacy-utils.js")` calls bypass Module Federation
   - They're bundled as regular dependencies, not shared dependencies

### What's Missing

The issue is **NOT** in the CommonJS dependency implementation - it's in how CommonJS modules are **consumed**. The macro generation code exists but isn't triggered because:

1. **`get_condition()` method missing**: CommonJS dependencies don't implement the ModuleDependency condition system
2. **Direct require() bypasses sharing**: CommonJS modules accessed via `require()` don't go through ConsumeShared

### Two Possible Solutions

#### Option 1: Add `get_condition()` to CommonJS Dependencies ⚠️ 
**RISKY** - We tried this but hit compatibility issues:
- `InlineConstDependencyCondition` expects `ESMImportSpecifierDependency`
- Adding `used_by_exports` field requires changing dependency structures
- Could break existing CommonJS functionality

#### Option 2: Fix CommonJS ConsumeShared Behavior ✅ 
**SAFER** - Investigate why CommonJS modules don't become ConsumeShared when they should:
- Why don't `require()` calls trigger ConsumeShared mechanism?
- Should Module Federation automatically convert direct `require()` to shared consumption?
- This preserves existing CommonJS behavior while enabling sharing

## 📊 Test Results Confirm Analysis

### Manual Testing Shows:
- **ESM shared modules**: 24 macro annotations found ✅
- **CommonJS modules**: 0 macro annotations found ❌  
- **Reason**: CommonJS modules don't go through ConsumeShared mechanism

### Build Output Shows:
- CommonJS modules are **ProvideShared** ✅
- CommonJS modules are **NOT ConsumeShared** ❌
- ESM modules are **both ProvideShared AND ConsumeShared** ✅

## 🎯 Recommendation

**Focus on Option 2**: Investigate the Module Federation consumption mechanism rather than modifying CommonJS dependency internals. The macro generation infrastructure already exists - we just need to ensure CommonJS modules flow through the ConsumeShared path when appropriate.

### Next Steps:
1. Study how Module Federation decides between direct bundling vs ConsumeShared
2. Determine if `require()` calls to shared modules should automatically become ConsumeShared
3. Implement consumption logic changes rather than dependency structure changes

This approach is:
- ✅ Less disruptive to existing CommonJS code
- ✅ Leverages existing macro generation infrastructure  
- ✅ Addresses the root cause (consumption) rather than symptoms (missing conditions)
- ✅ Maintains backward compatibility