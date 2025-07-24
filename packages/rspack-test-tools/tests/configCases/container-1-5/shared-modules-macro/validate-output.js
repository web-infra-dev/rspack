// Post-build validation script to check PURE annotations in output
const fs = require("fs");
const path = require("path");

// This will be run after the build to validate the output
module.exports = function validateOutput(outputPath) {
	const mainJs = fs.readFileSync(path.join(outputPath, "main.js"), "utf-8");
	
	// Check for PURE annotations in the output
	const pureAnnotations = mainJs.match(/\/\*\s*#__PURE__\s*\*\/\s*__webpack_require__/g) || [];
	const treeshakingMacros = mainJs.match(/\/\*\s*@common:if\s*\[condition="treeShake\.[^"]+"\]\s*\*\//g) || [];
	
	// Shared modules should have PURE annotations
	if (pureAnnotations.length === 0) {
		throw new Error("Expected PURE annotations in shared modules, but found none");
	}
	
	// Check for tree-shaking macros
	if (treeshakingMacros.length === 0) {
		throw new Error("Expected tree-shaking macros in shared modules, but found none");
	}
	
	// Verify macros don't contain 'placeholder'
	treeshakingMacros.forEach(macro => {
		if (macro.includes('placeholder')) {
			throw new Error(`Found placeholder in macro: ${macro}`);
		}
	});
	
	console.log(`✓ Found ${pureAnnotations.length} PURE annotations`);
	console.log(`✓ Found ${treeshakingMacros.length} tree-shaking macros`);
	console.log(`✓ All macros have proper share keys (no placeholders)`);
	
	return true;
};