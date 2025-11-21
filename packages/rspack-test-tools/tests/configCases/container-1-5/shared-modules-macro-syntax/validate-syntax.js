// Syntax validation script to check macro processing doesn't break JavaScript
const fs = require("fs");
const path = require("path");

module.exports = function validateSyntax(outputPath) {
	const mainJs = fs.readFileSync(path.join(outputPath, "main.js"), "utf-8");

	// Test 1: Check for basic syntax errors by attempting to parse
	try {
		new Function(mainJs);
	} catch (error) {
		throw new Error(`JavaScript syntax error in main.js: ${error.message}`);
	}

	// Test 2: Simulate complete macro removal and test syntax
	const withoutMacros = mainJs.replace(
		/\/\*\s*@common:if[^*]*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//gs,
		""
	);

	try {
		new Function(withoutMacros);
	} catch (error) {
		throw new Error(`Syntax error after macro removal: ${error.message}`);
	}

	// Test 3: Check for common syntax issues
	const syntaxIssues = [];

	// Check for double commas
	if (withoutMacros.includes(",,")) {
		syntaxIssues.push("Double commas found after macro removal");
	}

	// Check for orphaned commas in object literals
	const orphanedCommaPattern = /\{\s*,|\,\s*\}/g;
	if (orphanedCommaPattern.test(withoutMacros)) {
		syntaxIssues.push("Orphaned commas in object literals");
	}

	// Check for empty arrow functions that should be null
	const emptyArrowPattern = /\(\s*\)\s*=>\s*\(\s*\)/g;
	if (emptyArrowPattern.test(withoutMacros)) {
		syntaxIssues.push("Empty arrow functions detected - should be () => null");
	}

	if (syntaxIssues.length > 0) {
		throw new Error(`Syntax issues detected: ${syntaxIssues.join(", ")}`);
	}

	console.log("✓ JavaScript syntax validation passed");
	console.log("✓ Macro removal simulation passed");
	console.log("✓ No syntax issues detected");

	return true;
};
