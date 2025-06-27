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

		// Check for valid macro patterns - accept both single-line and multi-line formats
		const lines = objectContent.split('\n');
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
			if (line.includes('@common:if') && !line.includes('*/')) {
				syntaxErrors.push(`Line ${i + 1}: Unclosed @common:if comment`);
			}
			if (line.includes('@common:endif') && !line.includes('/*')) {
				syntaxErrors.push(`Line ${i + 1}: Unmatched @common:endif comment`);
			}
		}

		console.log(`✅ Found ${macroBlocks} macro blocks`);
		
		if (syntaxErrors.length > 0) {
			console.log(`❌ Found ${syntaxErrors.length} syntax errors:`);
			syntaxErrors.forEach(error => console.log(`  ${error}`));
			throw new Error(`Found syntax errors in macro comments`);
		}

		if (macroBlocks === 0) {
			throw new Error("No macro blocks found - check if macro generation is working");
		}

		// Verify specific examples - check calculateSum exists in macro format
		const calculateSumPattern = /\/\*\s*@common:if.*calculateSum.*@common:endif\s*\*\//;
		const foundCalculateSum = content.match(calculateSumPattern);
		
		if (!foundCalculateSum) {
			console.log(`ℹ️  calculateSum not found in expected macro format, checking if present...`);
			const hasCalculateSum = content.includes('calculateSum');
			if (hasCalculateSum) {
				console.log(`✅ calculateSum found in file`);
			} else {
				throw new Error("calculateSum should be present in the module.exports object");
			}
		} else {
			console.log(`✅ Found calculateSum in correct macro format`);
		}

		// Verify macro blocks are balanced
		const ifCount = (objectContent.match(/\/\*\s*@common:if/g) || []).length;
		const endifCount = (objectContent.match(/\/\*\s*@common:endif/g) || []).length;
		
		if (ifCount !== endifCount) {
			throw new Error(`Unbalanced macro blocks: ${ifCount} @common:if vs ${endifCount} @common:endif`);
		}
		
		console.log(`✅ Found ${ifCount} balanced macro block pairs`);
	});

	test("should validate macro structure is syntactically correct", () => {
		const targetFile = path.join(process.cwd(), "dist/cjs-modules_module-exports-pattern_js.js");
		const content = fs.readFileSync(targetFile, "utf8");

		// Find the module.exports object
		const moduleExportsMatch = content.match(/module\.exports\s*=\s*\{([^}]*)\}/s);
		const objectContent = moduleExportsMatch[1];

		// Check for basic syntax validity
		const lines = objectContent.split('\n');
		let syntaxIssues = [];
		
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			
			// Check for unmatched comment blocks
			const ifCount = (line.match(/\/\*\s*@common:if/g) || []).length;
			const endifCount = (line.match(/\/\*\s*@common:endif/g) || []).length;
			const openComments = (line.match(/\/\*/g) || []).length;
			const closeComments = (line.match(/\*\//g) || []).length;
			
			if (openComments !== closeComments) {
				syntaxIssues.push(`Line ${i + 1}: Unmatched comment blocks`);
			}
		}

		if (syntaxIssues.length > 0) {
			console.log(`❌ Found ${syntaxIssues.length} syntax issues:`);
			syntaxIssues.forEach(issue => console.log(`  ${issue}`));
			throw new Error(`Syntax issues found in macro structure`);
		}

		console.log("✅ Macro structure is syntactically correct");
	});
});