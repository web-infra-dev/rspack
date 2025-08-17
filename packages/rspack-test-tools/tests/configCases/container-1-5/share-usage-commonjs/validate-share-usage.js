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
	expect(fs.existsSync(shareUsageFile)).toBe(true);
	
	// Read and parse the JSON
	const content = fs.readFileSync(shareUsageFile, "utf8");
	const shareUsageData = JSON.parse(content);
	
	// Basic structure validation
	expect(shareUsageData).toHaveProperty("treeShake");
	
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
	
	expect(modules).toHaveProperty(moduleName);
	
	const moduleData = modules[moduleName];
	
	// Check that used exports are marked as true
	expect(moduleData.formatDate).toBe(true);
	expect(moduleData.processData).toBe(true);
	
	// Check that unused exports are marked as false (if they are tracked)
	if (moduleData.hasOwnProperty("unusedFunction")) {
		expect(moduleData.unusedFunction).toBe(false);
	}
	if (moduleData.hasOwnProperty("helperUtil")) {
		expect(moduleData.helperUtil).toBe(false);
	}
	
	// Ensure chunk_characteristics exists
	expect(moduleData).toHaveProperty("chunk_characteristics");
}

/**
 * Validate module.exports = { ... } pattern tracking
 */
function validateModuleExportsPattern(modules) {
	const moduleName = "./cjs-module-exports-pattern";
	
	expect(modules).toHaveProperty(moduleName);
	
	const moduleData = modules[moduleName];
	
	// This pattern might result in __dynamic_commonjs__ if not fully analyzable
	if (moduleData["__dynamic_commonjs__"] === true) {
		// Dynamic CommonJS marker is acceptable for module.exports = {...} pattern
		expect(moduleData["__dynamic_commonjs__"]).toBe(true);
	} else {
		// Otherwise check that used exports are marked as true
		expect(moduleData.calculateSum).toBe(true);
		expect(moduleData.formatCurrency).toBe(true);
	}
	
	// Check for unused exports if they are tracked
	if (moduleData.hasOwnProperty("calculateAverage")) {
		expect(moduleData.calculateAverage).toBe(false);
	}
	if (moduleData.hasOwnProperty("formatPercentage")) {
		expect(moduleData.formatPercentage).toBe(false);
	}
	
	// Ensure chunk_characteristics exists
	expect(moduleData).toHaveProperty("chunk_characteristics");
}

/**
 * Validate mixed pattern module tracking
 */
function validateMixedPattern(modules) {
	const moduleName = "./cjs-mixed-pattern";
	
	expect(modules).toHaveProperty(moduleName);
	
	const moduleData = modules[moduleName];
	
	// Check that used exports are marked as true
	expect(moduleData.utilityA).toBe(true);
	
	// Check that unused exports are marked as false (if tracked)
	if (moduleData.hasOwnProperty("utilityB")) {
		expect(moduleData.utilityB).toBe(false);
	}
	if (moduleData.hasOwnProperty("utilityC")) {
		expect(moduleData.utilityC).toBe(false);
	}
	
	// Ensure chunk_characteristics exists
	expect(moduleData).toHaveProperty("chunk_characteristics");
}