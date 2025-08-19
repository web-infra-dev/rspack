# RFC: Module Federation Redux isPlainObject Nullification Issue

**RFC Number:** TBD  
**Date:** 2025-08-19  
**Status:** Problem Analysis  
**Author:** Rspack Team  
**Related PRs:**

- [swc_macro_sys#24](https://github.com/CPunisher/swc_macro_sys/pull/24) - SWC macro implementation
- [rspack#10920](https://github.com/web-infra-dev/rspack/pull/10920) - ShareUsagePlugin integration  

## Executive Summary

When using Module Federation with Rspack, the `isPlainObject` function from Redux is being incorrectly nullified during optimization, causing runtime errors. This occurs because Redux is a non-shared dependency of the shared module @reduxjs/toolkit, and the ShareUsagePlugin doesn't track usage across this boundary.

## Problem Statement

### Runtime Error
```javascript
main.js:5040 Error loading bootstrap: TypeError: (0 , redux__WEBPACK_IMPORTED_MODULE_0__.isPlainObject) is not a function
    at configureStore (vendors-node_modules_pnpm_reduxjs_toolkit_2_8_2_react-redux_9_2_0__types_react_18_3_23_react_-3c49cc.cf30e018de44fba2.js:518:81)
    at ./src/store/index.js (src_bootstrap_jsx.js:1133:77)
```

This error occurs when @reduxjs/toolkit's `configureStore` function attempts to call `isPlainObject` from Redux, but finds it has been nullified to `()=>null` by the optimizer.

## Root Cause Analysis

### Module Dependency Structure

```
@reduxjs/toolkit (SHARED in Module Federation)
    └── internally imports → redux (NOT SHARED - internal npm dependency)
                                └── exports → isPlainObject (gets nullified)
```

### Why Redux is Missing from share-usage.json

1. **Redux is an internal sub-dependency**: Redux is not configured as shared in Module Federation - it's just an internal npm dependency of @reduxjs/toolkit

2. **ShareUsagePlugin only tracks shared modules**: The plugin's `analyze_consume_shared_usage` function only processes modules where `module.module_type() == &ModuleType::ConsumeShared`

3. **Internal dependencies are invisible**: When @reduxjs/toolkit (shared) internally uses redux (non-shared), this usage pattern is not captured by the current plugin

4. **No usage data leads to nullification**: Without redux appearing in share-usage.json, the optimizer has no information about which redux exports are actually used, so it aggressively nullifies them

## The Problem: Inter-Module Dependencies

### Current Behavior
The ShareUsagePlugin correctly tracks:
- ✅ Shared module exports (@reduxjs/toolkit exports like `configureStore`)
- ✅ Usage of shared modules by the application
- ❌ Internal dependencies of shared modules (redux used by @reduxjs/toolkit)

### The Gap
When a shared module has internal npm dependencies that are NOT configured as shared:
1. These sub-dependencies are bundled into the same chunk as the shared module
2. But they don't get usage analysis because they're not `ConsumeShared` modules
3. The optimizer sees them as "unused" and nullifies their exports
4. Runtime breaks when the shared module tries to use its internal dependency

1. **Build-Time Export Usage Tracking**: The ShareUsagePlugin analyzes which exports from shared modules are actually used by federated applications.

2. **Build-Time Chunk Optimization**: A SWC macro processor rewrites shared chunks using conditional compilation annotations based on collected usage data.

### Architecture

```
Build Time:
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Module Analysis │ -> │ Export Detection │ -> │ Usage Tracking  │
│ (ShareUsagePlugin)│    │ (ConsumeShared)  │    │ (share-usage.json)│
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                          │
                                                          v
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│ Optimized Chunks│ <- │ SWC Macro        │ <- │ Usage Data      │
│ (Annotation-based)│    │ Transformation   │    │ Aggregation     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Technical Specification

### ShareUsagePlugin

**Location**: `crates/rspack_plugin_mf/src/sharing/share_usage_plugin.rs`

#### Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  #[serde(rename = "treeShake")]
  pub tree_shake: HashMap<String, ModuleExportUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportUsage {
  #[serde(flatten)]
  pub exports: HashMap<String, bool>,  // export_name -> is_used
  pub chunk_characteristics: ChunkCharacteristics,
}
```

#### Export Detection

The plugin implements export detection for both ESM and CommonJS modules:

- **ESM Modules**: Analyzes import/export statements through module graph connections
- **CommonJS Modules**: Examines CjsExports dependencies and tracks dynamic exports with "\*" markers
- **Cross-module Dependencies**: Tracks usage across shared module boundaries through incoming connections

#### Usage Tracking

The plugin creates a boolean map for each export, tracking whether it's used (`true`) or unused (`false`). For CommonJS modules with dynamic exports, a special `__dynamic_commonjs__` flag is used.

### Output Format

The plugin outputs a `share-usage.json` file containing:

- Export usage map (export name → boolean)
- Chunk metadata (files, runtime names, module types)

## SWC Macro Processor

### Implementation

**Location**: `examples/swc_macro_wasm_pkg/`

The processor provides WASM-based transformation APIs:

- `optimize_with_prune_result_json`: Processes chunks with tree-shake configuration
- Additional APIs for chunk parsing and dependency analysis

### Annotation System

The processor injects conditional compilation annotations:

```javascript
// Original export with annotation
exportName: () => (/* @common:if [condition="treeShake.library.exportName"] */
  /* reexport safe */ actualImplementation
  /* @common:endif */),

// After processing when marked as unused
exportName: () => null,
```

Configuration format uses dot-notation paths for conditional evaluation.

## Implementation Pipeline

### Usage Data Aggregation

**Location**: `examples/module-federation-react-example/scripts/optimize-shared-chunks.js`

The optimization script:

1. Collects share-usage.json files from all federated applications
2. Merges usage data using OR logic (any usage marks export as required)
3. Applies SWC macro transformation to shared chunks
4. Replaces original chunks with optimized versions

### Processing Flow

```javascript
// 1. Aggregate usage across apps
mergedUsage = OR(app1Usage, app2Usage, ...)

// 2. Transform chunks
optimized = swcMacro.optimize(chunk, mergedUsage)

// 3. Replace chunks
fs.writeFileSync(chunkPath, optimized)
```

### Module Pruning Analysis

The optimization process performs reachability analysis from entry modules:

```
Starting module pruning with config
Analyzing modules object with 619 modules
Using entry module: lodash-es/lodash.js
Reachability analysis: 156 modules reachable from entry
Module pruning complete: 463 modules pruned, 156 modules kept
```

This demonstrates the effectiveness of tracking actual usage patterns versus bundling entire libraries.

## Performance Results

### Real-World Optimization Metrics

Based on actual optimization runs across host and remote applications:

#### Per-Library Results

| Library           | Size Reduction | Modules Kept | Modules Pruned | Original Size | Optimized Size |
| ----------------- | -------------- | ------------ | -------------- | ------------- | -------------- |
| @ant-design/icons | 95.8%          | 96           | 3,172          | 10.79 MB      | 0.44 MB        |
| lodash-es         | 78.4%          | 312          | 926            | 2.92 MB       | 0.63 MB        |
| antd              | 26.6%          | 1,759        | 843            | 16.01 MB      | 11.75 MB       |
| react-router-dom  | 17.5%          | 3            | 4              | 0.51 MB       | 0.42 MB        |
| @reduxjs/toolkit  | 24.9%          | 8            | 2              | 0.32 MB       | 0.24 MB        |
| react-redux       | 46.2%          | -            | -              | 0.04 MB       | 0.02 MB        |

#### Aggregate Metrics

- **Total Bundle Size**: 31.81 MB → 15.49 MB (51.30% reduction)
- **Modules Analyzed**: 7,125 total modules
- **Modules Pruned**: 4,947 (69.4% pruning rate)
- **Build Time**: ~4 seconds for initial builds

#### Notable Observations

Some libraries show size increases due to:

- **Development Mode Code**: Libraries like react, react-dom include development warnings
- **WASM Overhead**: Chart.js increases by 27.4% due to annotation processing overhead
- **CommonJS Wrapping**: Dynamic CommonJS modules require additional wrapper code

## Known Limitations

### Transitive Dependency Issue

**Problem**: The system only tracks exports from explicitly shared modules. When a shared module internally depends on a non-shared module, those transitive dependencies are not tracked.

**Example**:

```
@reduxjs/toolkit (shared) → redux (not shared) → isPlainObject
```

Since `redux` is not configured as a shared module but is bundled as a dependency of `@reduxjs/toolkit`, its exports are not tracked by ShareUsagePlugin. This can lead to runtime errors when tree-shaking removes exports that are used internally.

**Current Behavior**:

```javascript
// @reduxjs/toolkit internally uses:
import { isPlainObject } from "redux";

// But redux exports are transformed to:
__webpack_require__.d(__webpack_exports__, {
	isPlainObject: () => null // Runtime error!
});
```

**Root Cause**: The ShareUsagePlugin only analyzes ConsumeShared module types. When redux is bundled within @reduxjs/toolkit's chunk rather than consumed as a shared module, its usage is not tracked.

### Share-usage.json Structure

Example output structure:

```json
{
	"treeShake": {
		"library-name": {
			"exportName": true, // Used export
			"unusedExport": false, // Unused export
			"chunk_characteristics": {
				"entry_module_id": "path/to/module",
				"chunk_files": ["chunk.js"],
				"runtime_names": ["main"]
			}
		}
	}
}
```

## Capabilities

### Supported Features

1. **Export-Level Granularity**: Individual boolean flags for each export
2. **Module System Support**: ESM, CommonJS, and mixed module formats
3. **Federation-Aware**: Cross-application usage aggregation with OR-merge logic
4. **Annotation-Based Processing**: Conditional compilation without runtime overhead

### Scope

The system operates exclusively on modules configured as shared in Module Federation. Non-shared dependencies are handled by standard webpack/rspack tree-shaking mechanisms.

## Additional Limitations

### Federation Boundary Conflicts

The OR-merge strategy ensures exports used by any federated app are preserved, but may over-preserve exports in complex re-export scenarios.

### Dynamic Export Detection

CommonJS modules with runtime-conditional exports cannot be accurately analyzed at build time. The system uses `__dynamic_commonjs__` markers for conservative handling.

### Build-Time Constraints

- Static analysis cannot predict runtime-conditional imports
- Hot Module Replacement may bypass optimization
- Annotation processing failures result in unoptimized chunks

## Evidence from Actual Output

### 1. Redux Module Nullified in Optimized Chunk
```javascript
// In vendors-node_modules_pnpm_reduxjs_toolkit_2_8_2...js line 59
__webpack_require__.d(__webpack_exports__, {
    isPlainObject: ()=>null,  // <-- Function replaced with null
    combineReducers: ()=>null,
    createStore: ()=>null,
    // All redux exports are nullified
});
```

### 2. Redux Completely Missing from share-usage.json
```json
{
  "treeShake": {
    "@reduxjs/toolkit": { 
      "configureStore": true,
      "createSlice": true,
      "createAsyncThunk": true
    },
    "react-redux": {
      "Provider": true,
      "useSelector": true,
      "useDispatch": true
    }
    // Redux module is not here - it's not tracked at all
  }
}
```

### 3. ShareUsagePlugin Code Shows the Gap
```rust
// The plugin only processes ConsumeShared modules
fn analyze_consume_shared_usage(&self, compilation: &Compilation) {
    for module_id in module_graph.modules().keys() {
        if module.module_type() == &ModuleType::ConsumeShared {
            // Only shared modules are analyzed
            // Redux is NOT ConsumeShared, so it's never analyzed
        }
    }
}
```

## Summary

The Redux `isPlainObject` nullification issue demonstrates a critical gap in the current ShareUsagePlugin implementation:

### The Core Problem
- **Shared modules** (like @reduxjs/toolkit) can have **internal npm dependencies** (like redux) that are NOT configured as shared
- These sub-dependencies are bundled into the same chunk but receive no usage analysis
- The optimizer aggressively nullifies their exports, causing runtime failures

### Why It Happens
1. ShareUsagePlugin only analyzes `ModuleType::ConsumeShared` modules
2. Internal dependencies are regular modules, not ConsumeShared
3. Without usage data in share-usage.json, the optimizer assumes exports are unused
4. Functions get replaced with `()=>null`, breaking runtime execution

### Impact
- **Severity**: High - causes production runtime crashes
- **Scope**: Generic - affects any shared module with non-shared sub-dependencies  
- **Hidden Nature**: Only appears after optimization, not in development

This is not specific to Redux - it's a systemic issue with how Module Federation tree-shaking handles the boundary between shared and non-shared modules.
