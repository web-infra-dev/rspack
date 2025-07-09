# Tree Shaking Split Chunks Problem Statement

## Overview
Tree shaking is not working effectively for rspack split chunks, resulting in 640 lodash modules remaining in the bundle even when ALL exports are disabled.

## Current Behavior

### What Works ✅
1. **Standard webpack bundles** (`var __webpack_modules__ = {...}`):
   - Tree shaking removes unreachable modules correctly
   - Example: 269 bytes removed from a 295-byte test bundle

2. **Macro transformations**:
   - Unused exports are correctly replaced with `null`
   - Export declarations are removed
   - Results in 43.3% size reduction for lodash chunk

### What Doesn't Work ❌
1. **Split chunk format** (`(self["webpackChunk..."] = ...).push([...], {...}]`):
   - NO modules are removed by tree shaking
   - Even with ALL exports disabled, 640 modules remain
   - Module dependencies are not properly tracked after macro transformation

## Root Causes

### 1. No Entry Points in Split Chunks
- Split chunks have no explicit `__webpack_require__(moduleId)` entry points
- All modules are considered potentially reachable since they could be imported dynamically
- The webpack_graph parser cannot determine which modules are truly dead code

### 2. Broken Dependency Chain
After macro transformation removes unused exports:
```javascript
// Before: Working dependency chain
const debounce = __webpack_require__("./debounce.js");
export { debounce };

// After: Broken dependency chain
export { debounce: () => null };
// The require statement is removed, breaking the dependency graph!
```

### 3. Parser Limitations
The current webpack_graph parser:
- Successfully detects split chunk format
- Extracts all 640 modules
- But finds 0 dependencies between modules (they appear isolated)
- Cannot build a proper dependency graph for tree shaking

## Test Results

### Test 1: All Exports False
- **Config**: Set all 323 lodash exports to false
- **Expected**: Most/all modules should be removed
- **Actual**: 640 modules remain (0 removed)
- **Size**: 904,808 bytes (only 43.3% reduction from removing export declarations)

### Test 2: Only VERSION Export
- **Config**: Enable only the VERSION export
- **Expected**: Only VERSION and its dependencies remain
- **Actual**: 640 modules remain (same as all false)

### Test 3: Single Complex Function (debounce)
- **Config**: Enable only debounce
- **Expected**: Only debounce and its dependencies remain
- **Actual**: 640 modules remain (same as all false)

## Technical Details

### Split Chunk Structure
```javascript
(self["webpackChunkrspack_basic_example"] = self["webpackChunkrspack_basic_example"] || []).push([
    ["vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js"],
    {
        "../../node_modules/.pnpm/lodash-es@4.17.21/node_modules/lodash-es/_DataView.js": function(...) { ... },
        "../../node_modules/.pnpm/lodash-es@4.17.21/node_modules/lodash-es/_Hash.js": function(...) { ... },
        // ... 638 more modules
    }
]);
```

### Why Modules Remain
1. Every module exports something (even internal utilities)
2. No explicit entry points to start reachability analysis
3. Conservative tree shaking assumes all exports might be needed
4. Dependency information is lost during macro transformation

## Potential Solutions

### 1. Aggressive Split Chunk Tree Shaking
- When all exports are disabled, remove the entire chunk
- When specific exports are enabled, trace their dependencies and remove unreachable modules
- Requires special handling for split chunks vs standard bundles

### 2. Preserve Dependencies During Transformation
- Keep `__webpack_require__` statements even when exports are nullified
- Allow tree shaker to trace the dependency graph
- Remove modules after tree shaking completes

### 3. Two-Phase Optimization
- Phase 1: Mark unused exports
- Phase 2: Tree shake based on usage patterns
- Requires coordination between macro transformation and tree shaking

### 4. Build-Time Tree Shaking
- Perform tree shaking during rspack build before chunk splitting
- Generate optimized chunks with only required modules
- Most effective but requires rspack core changes

## Recommendation

The current implementation achieves maximum optimization possible with the split chunk format limitations. For further optimization:

1. **Short term**: Implement aggressive tree shaking mode for split chunks when usage is known
2. **Long term**: Integrate tree shaking into the rspack build process before chunk generation
3. **Alternative**: Convert to standard webpack bundle format for better tree shaking

## Impact

- Current optimization: 43.3% size reduction (removing export declarations)
- Potential optimization: ~90%+ size reduction (removing unused modules)
- For lodash specifically: Could reduce from 640 to ~50-100 modules for typical usage