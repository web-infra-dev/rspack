#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

// Test for correct comma positioning in CommonJS object literals - specifically targeting module-exports-pattern

describe("Comma positioning in macro comments", () => {
	test("should have comma inside macro comments for module-exports-pattern", () => {
		const targetFile = path.join(process.cwd(), "dist/cjs-modules_module-exports-pattern_js.js");
		
		if (!fs.existsSync(targetFile)) {
			throw new Error(`Target file not found: ${targetFile}`);
		}

		const content = fs.readFileSync(targetFile, "utf8");
		console.log(`📁 Testing file: ${targetFile}`);

		// Find the module.exports object
		const moduleExportsMatch = content.match(/module\.exports\s*=\s*\{([^}]*)\}/s);
		if (!moduleExportsMatch) {
			throw new Error("No module.exports object found in target file");
		}

		const objectContent = moduleExportsMatch[1];
		console.log(`✅ Found module.exports object`);

		// Look for the incorrect pattern: comma OUTSIDE macro (should fail test)
		// This checks for: property /* @common:endif */, (comma after endif)
		const incorrectMatches = [];
		const lines = objectContent.split('\n');
		
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i].trim();
			if (line.includes('/* @common:endif */,')) {
				incorrectMatches.push(line);
			}
		}

		// Look for correct pattern: property, /* @common:endif */ (comma before endif)
		const correctMatches = [];
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i].trim();
			if (line.includes(', /* @common:endif */') && !line.includes('/* @common:endif */,')) {
				correctMatches.push(line);
			}
		}

		console.log(`✅ Found ${correctMatches.length} correctly positioned commas`);
		if (incorrectMatches.length > 0) {
			console.log(`❌ Found ${incorrectMatches.length} incorrectly positioned commas`);
			for (const match of incorrectMatches) {
				console.log(`  Incorrect: ${match[0].trim()}`);
			}
			throw new Error(`Found ${incorrectMatches.length} incorrectly positioned commas - commas should be INSIDE macro boundaries, not outside`);
		}

		if (correctMatches.length === 0) {
			throw new Error("No correctly positioned commas found - check if macro generation is working");
		}

		// Verify specific examples - ensure calculateSum has comma INSIDE macro
		let foundCalculateSum = false;
		for (const line of lines) {
			if (line.includes('calculateSum') && line.includes('calculateSum, /* @common:endif */')) {
				foundCalculateSum = true;
				console.log(`✅ Found correctly formatted calculateSum with comma inside macro`);
				break;
			}
		}
		
		if (!foundCalculateSum) {
			throw new Error("calculateSum should have comma INSIDE macro: /* @common:if [...] */ calculateSum, /* @common:endif */");
		}

		// Verify no properties have commas outside macros
		const allMacroLines = objectContent.match(/\/\*\s*@common:if[^}]+@common:endif\s*\*\/[^,}]*/g);
		if (allMacroLines) {
			for (const line of allMacroLines) {
				if (/\/\*\s*@common:endif\s*\*\/\s*,/.test(line)) {
					throw new Error(`Found comma OUTSIDE macro boundary: ${line}`);
				}
			}
		}
	});

	test("should validate that all commas are inside macro boundaries", () => {
		const targetFile = path.join(process.cwd(), "dist/cjs-modules_module-exports-pattern_js.js");
		const content = fs.readFileSync(targetFile, "utf8");

		// Find the module.exports object
		const moduleExportsMatch = content.match(/module\.exports\s*=\s*\{([^}]*)\}/s);
		const objectContent = moduleExportsMatch[1];

		// This regex finds ALL macro blocks and checks if they end with comma outside
		const macroWithExternalComma = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/[^\/]*\/\*\s*@common:endif\s*\*\/\s*,/g;
		const violatingMatches = [...objectContent.matchAll(macroWithExternalComma)];

		if (violatingMatches.length > 0) {
			console.log(`❌ Found ${violatingMatches.length} patterns with commas OUTSIDE macro boundaries:`);
			for (const match of violatingMatches) {
				console.log(`  WRONG: ${match[0].trim()}`);
			}
			throw new Error(`All commas should be INSIDE macro boundaries. Found ${violatingMatches.length} violations.`);
		}

		console.log("✅ All commas are correctly positioned inside macro boundaries");
	});
});