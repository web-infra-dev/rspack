# RFC: Annotation-Guided Tree-Shaking for Module Federation Shared Modules

**RFC Number:** TBD  
**Date:** 2025-08-19  
**Status:** Proposal  
**Author:** Rspack Team  
**Related PRs:**

- [swc_macro_sys#24](https://github.com/CPunisher/swc_macro_sys/pull/24) - SWC macro implementation
- [rspack#10920](https://github.com/web-infra-dev/rspack/pull/10920) - ShareUsagePlugin integration

## Abstract

This RFC proposes an implementation for build-time tree-shaking of Module Federation shared modules using annotation-based conditional compilation. The system consists of a ShareUsagePlugin that tracks export usage across federated applications and a SWC macro processor that optimizes shared chunks based on actual usage patterns.

## Motivation

Module Federation enables sharing of dependencies across applications, but shared modules often include unused exports that increase bundle size. Current implementations bundle entire shared modules regardless of actual usage patterns, resulting in unnecessary overhead. This proposal addresses the need for fine-grained tree-shaking while maintaining Module Federation's runtime flexibility.

## Design Overview

The system operates in two distinct phases:

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

## Proposed Solutions

### 1. Transitive Dependency Tracking

**Option A**: Extend ShareUsagePlugin to analyze non-shared modules that are dependencies of shared modules:

- Track module graph connections beyond ConsumeShared boundaries
- Include transitive dependencies in usage analysis
- Generate warnings for potential runtime issues

**Option B**: Automatically promote transitive dependencies to shared status:

- Detect when shared modules depend on non-shared modules
- Add them to the shared configuration with appropriate settings
- Ensure consistent behavior across federated apps

### 2. Enhanced Configuration

```javascript
// Proposed: Explicit transitive dependency handling
shared: {
  "@reduxjs/toolkit": {
    singleton: true,
    requiredVersion: "^2.5.0",
    // New option to include transitive dependencies
    includeTransitive: ["redux"]
  }
}
```

### 3. Build-Time Validation

Add validation to detect potential runtime failures:

- Analyze internal module dependencies
- Warn when tree-shaking might break transitive dependencies
- Provide actionable error messages with configuration suggestions

### 2. Federation-Aware Optimization

**Cross-App Dependency Validation**:

```javascript
// Enhanced optimize-shared-chunks.js
function validateCrossFederationDependencies(mergedConfig, apps) {
	const issues = [];

	// Check for re-export conflicts
	Object.entries(mergedConfig.treeShake).forEach(([library, exports]) => {
		Object.entries(exports).forEach(([exportName, isUsed]) => {
			if (!isUsed) {
				// Check if any app actually depends on this via re-exports
				const dependents = findTransitiveDependents(library, exportName, apps);
				if (dependents.length > 0) {
					issues.push({
						library,
						exportName,
						dependents,
						solution: "Mark as required due to transitive dependencies"
					});
				}
			}
		});
	});

	return issues;
}
```

### 3. Annotation System Improvements

**Graceful Degradation and Debugging**:

```javascript
// Enhanced SWC macro error handling
function optimizeChunkWithFallback(chunkPath, config, optimizer) {
	try {
		const result = optimizer.optimize_with_prune_result_json(
			sourceCode,
			configJson
		);
		const parsed = JSON.parse(result);

		if (parsed?.error) {
			console.warn(`Optimization failed for ${chunkPath}: ${parsed.error}`);
			// Fallback to conservative tree-shaking
			return applyConservativeOptimization(sourceCode, config);
		}

		return parsed;
	} catch (error) {
		console.error(`WASM optimizer error for ${chunkPath}:`, error);
		// Return original source with debug annotations
		return {
			optimized_source: addDebugAnnotations(sourceCode),
			skip_reason: error.message
		};
	}
}
```

### 4. Advanced Configuration Options

**Fine-Grained Control**:

```javascript
// rspack.config.js - Actual module federation configuration
module.exports = {
	plugins: [
		new ModuleFederationPlugin({
			name: "host",
			shared: {
				redux: {
					singleton: true,
					requiredVersion: "^4.0.0",
					strictVersion: false,
					eager: false
				},
				"@reduxjs/toolkit": {
					singleton: true,
					requiredVersion: "^1.8.0",
					strictVersion: false
				}
			}
		})

		// ShareUsagePlugin is automatically applied by ModuleFederationRuntimePlugin
		// with default options: { filename: "share-usage.json" }
	]
};
```

## Implementation Timeline

### Phase 1: Transitive Dependency Support

- Extend ShareUsagePlugin to track non-shared module dependencies
- Add configuration options for transitive dependency handling
- Implement build-time validation and warnings

### Phase 2: Enhanced Developer Experience

- Improve error messages and debugging capabilities
- Add dependency visualization tools
- Create migration guides for existing projects

### Phase 3: Performance Optimization

- Implement incremental analysis caching
- Optimize memory usage for large dependency graphs
- Add parallel processing capabilities

## Conclusion

This RFC proposes a build-time tree-shaking system for Module Federation shared modules that achieves significant bundle size reductions through annotation-based conditional compilation. The implementation demonstrates 51.3% total size reduction in real-world applications.

### Key Benefits

- **Fine-grained Optimization**: Export-level tree-shaking for shared modules
- **Zero Runtime Overhead**: All processing occurs at build time
- **Federation Compatible**: Maintains consistency across federated applications

### Critical Issue

The system currently only tracks exports from explicitly shared modules. This limitation can cause runtime failures when shared modules internally depend on non-shared modules, as demonstrated by the redux/isPlainObject case.

### Next Steps

1. **Immediate**: Document the limitation and provide configuration guidance
2. **Short-term**: Implement transitive dependency tracking
3. **Long-term**: Develop automated dependency analysis and promotion

This proposal balances optimization benefits with Module Federation's flexibility requirements while acknowledging current limitations that need to be addressed.
