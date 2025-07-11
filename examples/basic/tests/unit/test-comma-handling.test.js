#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

/**
 * Test for trailing comma handling in object literals after macro evaluation
 */

describe("Comma Handling Tests", () => {
	test("should handle trailing commas correctly in object literals", () => {
		const distPath = path.join(process.cwd(), "dist");
		if (!fs.existsSync(distPath)) {
			throw new Error("Dist directory not found. Run npm run build first.");
		}
		const distFiles = fs.readdirSync(distPath).filter(f => f.endsWith(".js"));

		console.log("🔍 Testing comma handling in object literals...");

		let foundObjectLiterals = 0;
		let syntaxErrors = 0;

		for (const file of distFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Find object literals with macros
			const objectLiteralPattern =
				/(module\.exports|exports)\s*=\s*\{[\s\S]*?\}/g;
			const objectMatches = [...content.matchAll(objectLiteralPattern)];

			for (const match of objectMatches) {
				foundObjectLiterals++;
				const objectContent = match[0];

				console.log(`📦 Found object literal in ${file}:`);

				// Check for potential comma issues
				const issues = checkCommaIssues(objectContent);

				if (issues.length > 0) {
					console.log(`❌ Potential comma issues found:`);
					for (const issue of issues) {
						console.log(`  - ${issue}`);
						syntaxErrors++;
					}
				} else {
					console.log(`✅ No comma issues detected`);
				}
			}
		}

		console.log(
			`📊 Summary: ${foundObjectLiterals} object literals checked, ${syntaxErrors} potential issues`
		);

		// This should pass when comma handling is implemented correctly
		expect(syntaxErrors).toBe(0);
	});
});

function checkCommaIssues(objectContent) {
	const issues = [];

	// Split into lines for analysis
	const lines = objectContent.split("\n");

	for (let i = 0; i < lines.length; i++) {
		const line = lines[i].trim();

		// Check for orphaned commas (comma at start of line after macro)
		if (line.match(/^,\s*(?:\/\/|$)/)) {
			issues.push(`Line ${i + 1}: Orphaned comma at start of line`);
		}

		// Check for double commas
		if (line.includes(",,")) {
			issues.push(`Line ${i + 1}: Double commas detected`);
		}

		// Check for macro-wrapped properties with trailing commas that could become orphaned
		if (
			line.match(/\/\*\s*@common:endif\s*\*\/\s*,\s*$/) &&
			i < lines.length - 1
		) {
			const nextLine = lines[i + 1].trim();
			// If next line is also a macro or end of object, this comma could become orphaned
			if (nextLine.match(/^\/\*\s*@common:if/) || nextLine.match(/^\s*\}/)) {
				issues.push(
					`Line ${i + 1}: Trailing comma after macro could become orphaned`
				);
			}
		}
	}

	return issues;
}

describe("Comma Placement Strategy Tests", () => {
	test("should demonstrate correct comma placement strategy", () => {
		console.log("");
		console.log("✅ Correct comma placement strategies for object literals:");
		console.log("");
		console.log("1. Leading comma strategy (recommended):");
		console.log("   module.exports = {");
		console.log("     /* @common:if [...] */ prop1 /* @common:endif */");
		console.log("     /* @common:if [...] */, prop2 /* @common:endif */");
		console.log("     /* @common:if [...] */, prop3 /* @common:endif */");
		console.log("   };");
		console.log("");
		console.log("2. Conditional comma strategy:");
		console.log("   module.exports = {");
		console.log("     /* @common:if [...] */ prop1 /* @common:endif */");
		console.log("     /* @common:if [...] && hasMore */ , /* @common:endif */");
		console.log("     /* @common:if [...] */ prop2 /* @common:endif */");
		console.log("   };");
		console.log("");
		console.log("3. Smart trailing comma removal:");
		console.log("   - Remove trailing commas from last property in object");
		console.log("   - Use post-processing to clean up orphaned commas");

		// This test always passes - just shows strategies
		expect(true).toBe(true);
	});
});
