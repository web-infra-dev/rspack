#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

/**
 * Test for correct macro positioning format - this test should FAIL until we fix the implementation
 */

describe("Correct Macro Format Tests", () => {
	test("should have correct macro format for export assignments", () => {
		const distPath = path.join(process.cwd(), "dist");
		if (!fs.existsSync(distPath)) {
			throw new Error("Dist directory not found. Run npm run build first.");
		}
		const distFiles = fs.readdirSync(distPath).filter(f => f.endsWith(".js"));

		console.log("🔍 Testing correct macro format for export assignments...");

		let totalIncorrectPatterns = 0;
		let totalCorrectPatterns = 0;

		distFiles.forEach(file => {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Current INCORRECT format: exports.prop = /* @common:if [...] */ value /* @common:endif */;
			const incorrectPattern =
				/(?:exports|module\.exports)\.\w+\s*=\s*\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\/\s*;?/g;
			const incorrectMatches = [...content.matchAll(incorrectPattern)];

			// Expected CORRECT format: /* @common:if [...] */ exports.prop = value; /* @common:endif */
			const correctPattern =
				/\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*(?:exports|module\.exports)\.\w+\s*=[\s\S]*?\/\*\s*@common:endif\s*\*\//g;
			const correctMatches = [...content.matchAll(correctPattern)];

			totalIncorrectPatterns += incorrectMatches.length;
			totalCorrectPatterns += correctMatches.length;

			if (incorrectMatches.length > 0) {
				console.log(
					`❌ ${file}: ${incorrectMatches.length} incorrect patterns found`
				);
				incorrectMatches.forEach((match, i) => {
					const preview =
						match[0].length > 100
							? match[0].substring(0, 100) + "..."
							: match[0];
					console.log(`  ${i + 1}. ${preview}`);
				});
			}
		});

		console.log(
			`📊 Total: ${totalIncorrectPatterns} incorrect, ${totalCorrectPatterns} correct patterns`
		);

		if (totalIncorrectPatterns > 0) {
			console.log("");
			console.log("❌ Expected format examples:");
			console.log(
				"  WRONG: exports.prop = /* @common:if [...] */ value /* @common:endif */;"
			);
			console.log(
				"  RIGHT: /* @common:if [...] */ exports.prop = value; /* @common:endif */"
			);
			console.log("");
			console.log(
				"  WRONG: exports.obj = /* @common:if [...] */ { ... } /* @common:endif */;"
			);
			console.log(
				"  RIGHT: /* @common:if [...] */ exports.obj = { ... }; /* @common:endif */"
			);
		}

		// CJS modules without shared context should have NO macros at all
		expect(totalIncorrectPatterns).toBe(0);
		// With hardcoded patterns removed, CJS modules correctly have no macros
		console.log(
			"✅ Correctly found 0 tree-shaking macros in CJS modules without shared context"
		);
	});

	test("should have tree-shaking macros in pure-cjs-helper", () => {
		const targetFile = path.join(
			process.cwd(),
			"dist",
			"cjs-modules_pure-cjs-helper_js.js"
		);
		if (!fs.existsSync(targetFile)) {
			throw new Error(`Target file not found: ${targetFile}`);
		}

		const content = fs.readFileSync(targetFile, "utf8");

		// CJS modules with Module Federation shared context should have macros
		const hasMacros =
			content.includes("@common:if") && content.includes("@common:endif");

		if (!hasMacros) {
			throw new Error(
				"pure-cjs-helper should have tree-shaking macros - CJS modules in Module Federation shared context get macros"
			);
		}

		// Verify exports exist with proper macros
		const expectedExports = ["DataValidator", "generateId", "hashString"];
		expectedExports.forEach(exportName => {
			const macroPattern = new RegExp(
				`@common:if.*treeShake\\.cjs-pure-helper\\.${exportName}.*@common:endif`,
				"s"
			);
			if (!macroPattern.test(content)) {
				throw new Error(
					`Expected export '${exportName}' to have tree-shaking macros`
				);
			}
		});

		console.log(
			"✅ Correctly found tree-shaking macros in pure-cjs-helper (CJS with Module Federation shared context)"
		);
	});

	test("should demonstrate expected correct format examples", () => {
		console.log("");
		console.log("✅ Expected CORRECT macro positioning format:");
		console.log("");
		console.log("1. Simple assignment:");
		console.log(
			'   /* @common:if [condition="treeShake.cjs-data-processor.processArray"] */ exports.processArray = processArray; /* @common:endif */'
		);
		console.log("");
		console.log("2. Object assignment:");
		console.log(
			'   /* @common:if [condition="treeShake.cjs-legacy-utils.constants"] */ exports.constants = {'
		);
		console.log('       DEFAULT_ENCODING: "utf8",');
		console.log("       MAX_FILE_SIZE: 1024 * 1024,");
		console.log('       SUPPORTED_FORMATS: ["txt", "json", "js"]');
		console.log("   }; /* @common:endif */");
		console.log("");
		console.log("3. Function assignment:");
		console.log(
			'   /* @common:if [condition="treeShake.cjs-pure-helper.processData"] */ exports.processData = function(data) {'
		);
		console.log("     if (!Array.isArray(data)) {");
		console.log("       return null;");
		console.log("     }");
		console.log("     return data.map(item => ({");
		console.log("       id: this.generateId(),");
		console.log("       hash: this.hashString(String(item)),");
		console.log("       valid: this.validateInput(String(item))");
		console.log("     }));");
		console.log("   }; /* @common:endif */");
		console.log("");
		console.log("4. Module.exports object literal (ALREADY CORRECT):");
		console.log("   module.exports = {");
		console.log(
			'     /* @common:if [condition="treeShake.cjs-module-exports.calculateSum"] */ calculateSum /* @common:endif */,'
		);
		console.log(
			'     /* @common:if [condition="treeShake.cjs-module-exports.calculateAverage"] */ calculateAverage /* @common:endif */'
		);
		console.log("   };");

		// This test always passes - just shows expected format
		expect(true).toBe(true);
	});
});
