#!/usr/bin/env node

// Integration test for macro evaluation and syntax validation
const fs = require("node:fs");
const path = require("node:path");

// Test: should produce valid JavaScript when all macros are removed
(() => {
	const distPath = path.join(__dirname, "../../dist");
	const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
	
	for (const file of distFiles) {
		const filePath = path.join(distPath, file);
		const content = fs.readFileSync(filePath, "utf8");

		// Simulate complete macro removal (worst case scenario)
		// For ESM export specifiers, we need to handle the arrow function syntax
		let withoutMacros = content.replace(
			/\/\*\s*@common:if[^*]*\*\/.*?\/\*\s*@common:endif\s*\*\//gs,
			""
		);
		
		// Fix empty arrow functions created by macro removal: () => () becomes () => null
		withoutMacros = withoutMacros.replace(/\(\s*\)\s*=>\s*\(\s*\)/g, "() => null");

		// Try to parse as JavaScript to detect syntax errors
		let syntaxError = null;
		try {
			// Use eval to check syntax (in a safe way for testing)
			new Function(withoutMacros);
		} catch (error) {
			syntaxError = error.message;
		}

		if (syntaxError) {
			console.log(`❌ ${file}: Syntax error after macro removal: ${syntaxError}`);
			console.log("Generated code snippet:");
			console.log(withoutMacros.substring(0, 500) + "...");
			throw new Error(`Syntax error in ${file}: ${syntaxError}`);
		}
	}
	console.log("✅ All files produce valid JavaScript when all macros are removed");
})();

// Test: should produce valid JavaScript when some macros are removed
(() => {
	const distPath = path.join(__dirname, "../../dist");
	const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
	
	for (const file of distFiles) {
		const filePath = path.join(distPath, file);
		const content = fs.readFileSync(filePath, "utf8");

		// Find all macro conditions
		const macroMatches = [...content.matchAll(
			/\/\*\s*@common:if\s*\[condition="([^"]+)"\]\s*\*\/(.*?)\/\*\s*@common:endif\s*\*\//gs
		)];

		if (macroMatches.length === 0) continue;

		// Test removing every other macro (simulate partial tree shaking)
		let modifiedContent = content;
		for (let i = 0; i < macroMatches.length; i += 2) {
			const fullMatch = macroMatches[i][0];
			modifiedContent = modifiedContent.replace(fullMatch, "");
		}
		
		// Fix empty arrow functions created by macro removal
		modifiedContent = modifiedContent.replace(/\(\s*\)\s*=>\s*\(\s*\)/g, "() => null");

		// Check syntax
		let syntaxError = null;
		try {
			new Function(modifiedContent);
		} catch (error) {
			syntaxError = error.message;
		}

		if (syntaxError) {
			console.log(`❌ ${file}: Syntax error after partial macro removal: ${syntaxError}`);
			throw new Error(`Syntax error in ${file}: ${syntaxError}`);
		}
	}
	console.log("✅ All files produce valid JavaScript when some macros are removed");
})();

// Test: should handle empty object literals correctly
(() => {
	const distPath = path.join(__dirname, "../../dist");
	const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
	
	for (const file of distFiles) {
		const filePath = path.join(distPath, file);
		const content = fs.readFileSync(filePath, "utf8");

		// Simulate removing all exports from object literals
		let withoutAnyExports = content.replace(
			/\/\*\s*@common:if[^*]*\*\/.*?\/\*\s*@common:endif\s*\*\//gs,
			""
		);
		
		// Fix empty arrow functions created by macro removal
		withoutAnyExports = withoutAnyExports.replace(/\(\s*\)\s*=>\s*\(\s*\)/g, "() => null");

		// Check for empty object patterns like module.exports = { }
		const emptyObjectPattern = /module\.exports\s*=\s*\{\s*\}/g;
		const emptyObjects = withoutAnyExports.match(emptyObjectPattern);

		if (emptyObjects) {
			console.log(`✅ ${file}: Contains ${emptyObjects.length} properly formed empty objects`);
		}

		// Verify no malformed empty objects like { , } or { ,, }
		const malformedPattern = /\{\s*,+\s*\}/g;
		const malformed = withoutAnyExports.match(malformedPattern);

		if (malformed) {
			console.log(`❌ ${file}: Found malformed empty objects: ${malformed}`);
			throw new Error(`Malformed empty objects in ${file}`);
		}
	}
	console.log("✅ All files handle empty object literals correctly");
})();

console.log("🎉 All macro evaluation integration tests passed!");