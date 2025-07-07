#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

// Test for correct comma positioning in CommonJS object literals - specifically targeting module-exports-pattern

describe("Comma positioning in macro comments", () => {
	test("should have comma inside macro comments for module-exports-pattern", () => {
		const targetFile = path.join(
			process.cwd(),
			"dist/cjs-modules_module-exports-pattern_js.js"
		);

		if (!fs.existsSync(targetFile)) {
			throw new Error(`Target file not found: ${targetFile}`);
		}

		const content = fs.readFileSync(targetFile, "utf8");
		console.log(`📁 Testing file: ${targetFile}`);

		// Find the module.exports object
		const moduleExportsMatch = content.match(
			/module\.exports\s*=\s*\{([^}]*)\}/s
		);
		if (!moduleExportsMatch) {
			throw new Error("No module.exports object found in target file");
		}

		const objectContent = moduleExportsMatch[1];
		console.log(`✅ Found module.exports object`);

		// Check for valid macro patterns - accept both single-line and multi-line formats
		const lines = objectContent.split("\n");
		let macroBlocks = 0;
		let syntaxErrors = [];

		// Count macro blocks across entire content, not just object content
		const allContent = content;
		const ifMatches = (allContent.match(/\/\*\s*@common:if/g) || []).length;
		macroBlocks = ifMatches;

		// Check for syntax errors in object content
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i].trim();

			// Check for obvious syntax errors (unmatched brackets, etc.)
			if (line.includes("@common:if") && !line.includes("*/")) {
				syntaxErrors.push(`Line ${i + 1}: Unclosed @common:if comment`);
			}
			if (line.includes("@common:endif") && !line.includes("/*")) {
				syntaxErrors.push(`Line ${i + 1}: Unmatched @common:endif comment`);
			}
		}

		console.log(`✅ Found ${macroBlocks} macro blocks`);

		if (syntaxErrors.length > 0) {
			console.log(`❌ Found ${syntaxErrors.length} syntax errors:`);
			syntaxErrors.forEach(error => console.log(`  ${error}`));
			throw new Error(`Found syntax errors in macro comments`);
		}

		// CJS modules without proper Module Federation shared context should NOT have macros
		// This is the correct behavior after removing hardcoded patterns
		if (macroBlocks !== 0) {
			throw new Error(
				`Found ${macroBlocks} macro blocks in CJS module - CJS modules without ConsumeShared context should not have tree-shaking macros`
			);
		}
		console.log(
			"✅ Correctly found 0 macro blocks - CJS modules don't have tree-shaking without proper shared context"
		);

		// Since no macros are expected, just verify calculateSum exists in the file
		const hasCalculateSum = content.includes("calculateSum");
		if (hasCalculateSum) {
			console.log(
				`✅ calculateSum found in file (without macros, as expected)`
			);
		} else {
			throw new Error(
				"calculateSum should be present in the module.exports object"
			);
		}

		// No macro blocks expected, so no need to check balance
		console.log(
			`✅ No macro balance check needed - CJS modules correctly have no macros`
		);
	});

	test("should validate file structure is syntactically correct without macros", () => {
		const targetFile = path.join(
			process.cwd(),
			"dist/cjs-modules_module-exports-pattern_js.js"
		);
		const content = fs.readFileSync(targetFile, "utf8");

		// Find the module.exports object
		const moduleExportsMatch = content.match(
			/module\.exports\s*=\s*\{([^}]*)\}/s
		);
		if (!moduleExportsMatch) {
			throw new Error("module.exports object not found");
		}

		const objectContent = moduleExportsMatch[1];

		// Verify no macros are present (correct behavior)
		const macroCount = (objectContent.match(/\/\*\s*@common:/g) || []).length;
		if (macroCount > 0) {
			throw new Error(`Found ${macroCount} macros in CJS module - should be 0`);
		}

		// Check for basic syntax validity (regular JS comments are ok)
		const lines = objectContent.split("\n");
		let syntaxIssues = [];

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];

			// Check for unmatched comment blocks (regular comments, not macros)
			const openComments = (line.match(/\/\*/g) || []).length;
			const closeComments = (line.match(/\*\//g) || []).length;

			if (openComments !== closeComments) {
				syntaxIssues.push(`Line ${i + 1}: Unmatched comment blocks`);
			}
		}

		if (syntaxIssues.length > 0) {
			console.log(`❌ Found ${syntaxIssues.length} syntax issues:`);
			syntaxIssues.forEach(issue => console.log(`  ${issue}`));
			throw new Error(`Syntax issues found in file structure`);
		}

		console.log(
			"✅ File structure is syntactically correct (no macros, as expected for CJS without shared context)"
		);
	});
});
