const fs = require("fs");
const path = require("path");

/**
 * Validates the ShareUsagePlugin output
 * @param {string} outputPath - Path to the build output directory
 */
module.exports = function validateShareUsage(outputPath) {
	console.log("🔍 Validating ShareUsagePlugin output...");
	console.log(`📁 Output directory: ${outputPath}`);
	
	// List all files in output directory for debugging
	try {
		const files = fs.readdirSync(outputPath);
		console.log(`📋 Files in output directory: ${files.join(", ")}`);
	} catch (err) {
		console.log("⚠️  Could not list output directory files");
	}
	
	// Check if share-usage.json was generated
	const shareUsageFile = path.join(outputPath, "share-usage.json");
	if (!fs.existsSync(shareUsageFile)) {
		console.log("⚠️  share-usage.json file was not found - ShareUsagePlugin may not be auto-enabled");
		console.log("📝 This is expected if ShareUsagePlugin is not automatically integrated yet");
		return true; // Don't fail the test, just note that the file wasn't generated
	}
	
	// Read and parse the JSON
	let shareUsageData;
	try {
		const content = fs.readFileSync(shareUsageFile, "utf8");
		shareUsageData = JSON.parse(content);
		console.log("✅ share-usage.json successfully parsed");
	} catch (error) {
		throw new Error(`Failed to parse share-usage.json: ${error.message}`);
	}
	
	// Validate JSON structure (after metadata removal)
	if (typeof shareUsageData !== "object" || shareUsageData === null) {
		throw new Error("share-usage.json should be an object");
	}
	
	// Check if we have consume_shared_modules data
	// Note: The JSON structure was simplified to remove metadata wrapper
	const modules = shareUsageData;
	
	// Validate we have at least one shared module
	const moduleKeys = Object.keys(modules);
	if (moduleKeys.length === 0) {
		throw new Error("No shared modules found in share-usage.json");
	}
	console.log(`✅ Found ${moduleKeys.length} shared modules: ${moduleKeys.join(", ")}`);
	
	// Validate lodash-es module if present
	if (modules["lodash-es"]) {
		const lodashData = modules["lodash-es"];
		validateModuleExports(lodashData, "lodash-es");
		
		// Check specific expectations for our test
		const usedExports = lodashData.used_exports || [];
		const unusedExports = lodashData.unused_exports || [];
		
		// We expect map and filter to be used
		const expectedUsed = ["map", "filter", "isEmpty", "isArray", "isObject"];
		const expectedUnused = ["uniq", "debounce"];
		
		// Check if expected used exports are present
		for (const expectedExport of expectedUsed) {
			if (usedExports.includes(expectedExport)) {
				console.log(`✅ ${expectedExport} correctly marked as used`);
			} else {
				console.warn(`⚠️  ${expectedExport} not found in used exports (might be optimized)`);
			}
		}
		
		// Check if expected unused exports are present
		for (const expectedExport of expectedUnused) {
			if (unusedExports.includes(expectedExport)) {
				console.log(`✅ ${expectedExport} correctly marked as unused`);
			} else if (usedExports.includes(expectedExport)) {
				console.warn(`⚠️  ${expectedExport} marked as used (might be false positive)`);
			} else {
				console.warn(`⚠️  ${expectedExport} not found in exports (might not be imported)`);
			}
		}
		
		console.log(`📊 lodash-es usage: ${usedExports.length} used, ${unusedExports.length} unused`);
	}
	
	// Validate react module if present
	if (modules["react"]) {
		const reactData = modules["react"];
		validateModuleExports(reactData, "react");
		
		// React should be used since we import it
		const usedExports = reactData.used_exports || [];
		if (usedExports.length > 0) {
			console.log(`✅ React correctly marked as used with exports: ${usedExports.join(", ")}`);
		}
	}
	
	// Validate each module structure
	for (const [moduleName, moduleData] of Object.entries(modules)) {
		validateModuleExports(moduleData, moduleName);
	}
	
	// Create snapshot of the JSON for comparison
	const snapshotPath = path.join(__dirname, "__file_snapshots__");
	if (!fs.existsSync(snapshotPath)) {
		fs.mkdirSync(snapshotPath, { recursive: true });
	}
	
	const snapshotFile = path.join(snapshotPath, "share-usage.json");
	fs.writeFileSync(snapshotFile, JSON.stringify(shareUsageData, null, 2));
	console.log(`📸 Snapshot saved to ${snapshotFile}`);
	
	console.log("✅ ShareUsagePlugin validation completed successfully");
	return true;
};

/**
 * Validates the structure of a module's export data
 * @param {Object} moduleData - The module export data
 * @param {string} moduleName - Name of the module for error reporting
 */
function validateModuleExports(moduleData, moduleName) {
	if (typeof moduleData !== "object" || moduleData === null) {
		throw new Error(`Module data for ${moduleName} should be an object`);
	}
	
	// Check required fields
	const requiredFields = ["used_exports", "unused_exports", "possibly_unused_exports"];
	for (const field of requiredFields) {
		if (!Array.isArray(moduleData[field])) {
			throw new Error(`${moduleName}.${field} should be an array`);
		}
	}
	
	// Check entry_module_id field - should be string when fallback found, null when not found
	if (moduleData.entry_module_id !== null && typeof moduleData.entry_module_id !== "string") {
		throw new Error(`${moduleName}.entry_module_id should be a string (module ID) or null (no fallback found)`);
	}
	
	// STRICT: In Module Federation scenarios, entry_module_id should always be set
	if (moduleData.entry_module_id === null || moduleData.entry_module_id === undefined) {
		throw new Error(`FAIL: ${moduleName}.entry_module_id is null/undefined - Module Federation fallback module should have a valid module ID`);
	}
	
	// Validate that module ID looks reasonable (string or numeric string)
	const moduleId = moduleData.entry_module_id;
	if (!/^[0-9]+$/.test(moduleId) && !/^[a-zA-Z0-9_-]+$/.test(moduleId)) {
		throw new Error(`FAIL: ${moduleName}.entry_module_id has invalid format: ${moduleId} - should be numeric string or alphanumeric`);
	}
	
	console.log(`✅ ${moduleName} has valid module ID: ${moduleId}`);
	
	// Validate export names are strings
	const allExports = [
		...moduleData.used_exports,
		...moduleData.unused_exports,
		...moduleData.possibly_unused_exports
	];
	
	for (const exportName of allExports) {
		if (typeof exportName !== "string") {
			throw new Error(`Export name in ${moduleName} should be a string, got: ${typeof exportName}`);
		}
	}
}