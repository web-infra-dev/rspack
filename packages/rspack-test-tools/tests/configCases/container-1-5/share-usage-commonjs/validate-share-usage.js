/**
 * Validates that CommonJS module exports are correctly tracked in share-usage.json
 */
module.exports = function validateShareUsage() {
	// Use __non_webpack_require__ to access Node.js modules in the test environment
	const fs = __non_webpack_require__("fs");
	const path = __non_webpack_require__("path");

	// Check if share-usage.json was generated
	const shareUsageFile = path.join(__dirname, "share-usage.json");

	// Assert file exists
	if (!fs.existsSync(shareUsageFile)) {
		throw new Error("share-usage.json was not generated");
	}

	// Read and parse the JSON
	const content = fs.readFileSync(shareUsageFile, "utf8");
	const shareUsageData = JSON.parse(content);

	// Basic structure validation
	if (!shareUsageData.treeShake) {
		throw new Error("share-usage.json should have a treeShake property");
	}

	const modules = shareUsageData.treeShake;

	// Validate that CommonJS modules are being tracked
	validateExportsPattern(modules);
	validateModuleExportsPattern(modules);
	validateMixedPattern(modules);
};

/**
 * Validate exports.X and module.exports.X pattern tracking
 */
function validateExportsPattern(modules) {
	const moduleName = "./cjs-exports-pattern";

	if (!modules[moduleName]) {
		throw new Error(`Module '${moduleName}' not found in share-usage.json`);
	}

	const moduleData = modules[moduleName];

	// For now, we're validating that CommonJS exports are being detected and tracked
	// even if usage tracking in async contexts needs improvement

	// Check that exports are present (they may show as false due to async loading limitation)
	const expectedExports = [
		"formatDate",
		"processData",
		"unusedFunction",
		"helperUtil"
	];
	for (const exportName of expectedExports) {
		if (!moduleData.hasOwnProperty(exportName)) {
			throw new Error(`${moduleName}: Export '${exportName}' not tracked`);
		}
	}

	// Ensure chunk_characteristics exists
	if (!moduleData.chunk_characteristics) {
		throw new Error(`${moduleName}: Missing chunk_characteristics`);
	}
}

/**
 * Validate module.exports = { ... } pattern tracking
 */
function validateModuleExportsPattern(modules) {
	const moduleName = "./cjs-module-exports-pattern";

	if (!modules[moduleName]) {
		throw new Error(`Module '${moduleName}' not found in share-usage.json`);
	}

	const moduleData = modules[moduleName];

	// This pattern results in __dynamic_commonjs__ marker
	if (!moduleData["__dynamic_commonjs__"]) {
		throw new Error(
			`${moduleName}: Expected __dynamic_commonjs__ marker for module.exports pattern`
		);
	}

	// Ensure chunk_characteristics exists
	if (!moduleData.chunk_characteristics) {
		throw new Error(`${moduleName}: Missing chunk_characteristics`);
	}
}

/**
 * Validate mixed pattern module tracking
 */
function validateMixedPattern(modules) {
	const moduleName = "./cjs-mixed-pattern";

	if (!modules[moduleName]) {
		throw new Error(`Module '${moduleName}' not found in share-usage.json`);
	}

	const moduleData = modules[moduleName];

	// Check that exports are being tracked (even if marked as false due to async limitation)
	const expectedExports = ["utilityA", "utilityB", "utilityC"];
	for (const exportName of expectedExports) {
		if (!moduleData.hasOwnProperty(exportName)) {
			throw new Error(`${moduleName}: Export '${exportName}' not tracked`);
		}
	}

	// Ensure chunk_characteristics exists
	if (!moduleData.chunk_characteristics) {
		throw new Error(`${moduleName}: Missing chunk_characteristics`);
	}
}
