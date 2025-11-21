# ShareUsagePlugin Test

This test validates that ShareUsagePlugin correctly tracks used and unused exports from shared modules in Module Federation configurations.

## Test Coverage

### Import Pattern Coverage

- **ESM imports**: `import { map, filter } from "lodash-es"`
- **CommonJS requires**: `const { map, groupBy } = require("lodash-es")`
- **Default imports**: `import React from "react"`
- **Mixed imports**: `import defaultExport, { useState } from "react"`
- **Re-exports**: `export { default as ReactDefault } from "react"`

### Module Type Coverage

- **External npm packages**: lodash-es, react
- **Local CJS modules**: Various `module.exports` patterns
- **Local ESM modules**: Named exports, default exports, constants

### Export Pattern Coverage

- **CJS patterns**:
  - Object literal: `module.exports = { func1, func2 }`
  - Direct property: `module.exports.property = value`
- **ESM patterns**:
  - Named exports: `export function util() {}`
  - Default export: `export default function() {}`
  - Const exports: `export const CONSTANT = "value"`

## Test Files

- `rspack.config.js` - Module Federation config with npm and local shared modules
- `index.js` - Main test with comprehensive import patterns
- `utils.js` - Uses lodash-es functions (isEmpty, isArray, isObject)
- `components.js` - Uses React.createElement
- `cjs-test-module.js` - Tests CommonJS require patterns
- `esm-test-module.js` - Tests ESM import/export patterns
- `local-cjs-module.js` - Local CJS module with various export patterns
- `local-esm-module.js` - Local ESM module with various export patterns
- `validate-share-usage.js` - Strict assertion-based validation
- `test.config.js` - Test configuration with afterBuild validation

## Validation Assertions

The test uses strict assertions (no console logs) to verify:

1. **File Generation**: share-usage.json is created by ShareUsagePlugin
2. **Module Coverage**: All 4 shared modules are tracked (lodash-es, react, local modules)
3. **Export Accuracy**:
   - Used exports are correctly identified
   - Unused but imported exports are tracked
   - No false positives/negatives
4. **Module ID Assignment**: Each module has a valid entry_module_id
5. **Macro Compatibility**: Export names are valid JavaScript identifiers

## Expected JSON Structure

```json
{
	"lodash-es": {
		"used_exports": [
			"map",
			"filter",
			"isEmpty",
			"isArray",
			"isObject",
			"clone",
			"merge"
		],
		"unused_exports": [
			"uniq",
			"debounce",
			"groupBy",
			"partition",
			"cloneDeep",
			"mergeWith"
		],
		"possibly_unused_exports": [],
		"entry_module_id": "42"
	},
	"react": {
		"used_exports": ["createElement", "default"],
		"unused_exports": ["useState", "useEffect"],
		"possibly_unused_exports": [],
		"entry_module_id": "24"
	},
	"local-cjs-module": {
		"used_exports": ["usedLocalFunction", "constantValue"],
		"unused_exports": ["unusedLocalFunction", "unusedConstant"],
		"possibly_unused_exports": [],
		"entry_module_id": "101"
	},
	"local-esm-module": {
		"used_exports": ["usedLocalUtil", "USED_LOCAL_CONSTANT", "default"],
		"unused_exports": ["unusedLocalUtil", "UNUSED_LOCAL_CONSTANT"],
		"possibly_unused_exports": [],
		"entry_module_id": "102"
	}
}
```

## ShareUsagePlugin Integration

ShareUsagePlugin is automatically applied by ShareRuntimePlugin, which is enabled when using Module Federation. The plugin:

1. Analyzes ConsumeShared module usage during compilation
2. Tracks imports from both CommonJS and ESM modules
3. Identifies which exports are actually used vs just imported
4. Generates share-usage.json in the output directory

## Usage for Tree-Shaking Macros

The generated JSON enables conditional macros like:

```javascript
/* @common:if [condition="treeShake.lodash-es.map"] */
// Code that uses lodash map
/* @common:endif */
```
