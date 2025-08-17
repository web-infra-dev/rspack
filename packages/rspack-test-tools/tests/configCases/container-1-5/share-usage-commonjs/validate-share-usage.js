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
	
	// Validate exports.X pattern module
	validateExportsPattern(modules);
	
	// Validate module.exports = { ... } pattern module
	validateModuleExportsPattern(modules);
	
	// Validate mixed pattern module
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
	
	// Check that used exports are marked as true
	if (moduleData.formatDate !== true) {
		throw new Error(`${moduleName}.formatDate should be true, got ${moduleData.formatDate}`);
	}
	if (moduleData.processData !== true) {
		throw new Error(`${moduleName}.processData should be true, got ${moduleData.processData}`);
	}
	
	// Check that unused exports are marked as false (if they are tracked)
	if (moduleData.hasOwnProperty("unusedFunction") && moduleData.unusedFunction !== false) {
		throw new Error(`${moduleName}.unusedFunction should be false, got ${moduleData.unusedFunction}`);
	}
	if (moduleData.hasOwnProperty("helperUtil") && moduleData.helperUtil !== false) {
		throw new Error(`${moduleName}.helperUtil should be false, got ${moduleData.helperUtil}`);
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
	
	// This pattern might result in __dynamic_commonjs__ if not fully analyzable
	if (moduleData["__dynamic_commonjs__"] === true) {
		// Dynamic CommonJS marker is acceptable for module.exports = {...} pattern
		// Just check that it exists
	} else {
		// Otherwise check that used exports are marked as true
		if (moduleData.calculateSum !== true) {
			throw new Error(`${moduleName}.calculateSum should be true, got ${moduleData.calculateSum}`);
		}
		if (moduleData.formatCurrency !== true) {
			throw new Error(`${moduleName}.formatCurrency should be true, got ${moduleData.formatCurrency}`);
		}
	}
	
	// Check for unused exports if they are tracked
	if (moduleData.hasOwnProperty("calculateAverage") && moduleData.calculateAverage !== false) {
		throw new Error(`${moduleName}.calculateAverage should be false, got ${moduleData.calculateAverage}`);
	}
	if (moduleData.hasOwnProperty("formatPercentage") && moduleData.formatPercentage !== false) {
		throw new Error(`${moduleName}.formatPercentage should be false, got ${moduleData.formatPercentage}`);
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
	
	// Check that used exports are marked as true
	if (moduleData.utilityA !== true) {
		throw new Error(`${moduleName}.utilityA should be true, got ${moduleData.utilityA}`);
	}
	
	// Check that unused exports are marked as false (if tracked)
	if (moduleData.hasOwnProperty("utilityB") && moduleData.utilityB !== false) {
		throw new Error(`${moduleName}.utilityB should be false, got ${moduleData.utilityB}`);
	}
	if (moduleData.hasOwnProperty("utilityC") && moduleData.utilityC !== false) {
		throw new Error(`${moduleName}.utilityC should be false, got ${moduleData.utilityC}`);
	}
	
	// Ensure chunk_characteristics exists
	if (!moduleData.chunk_characteristics) {
		throw new Error(`${moduleName}: Missing chunk_characteristics`);
	}
}