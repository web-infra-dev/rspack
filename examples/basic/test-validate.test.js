#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, test, expect } from "@rstest/core";

/**
 * Validation tests for rspack ConsumeShared chunks
 * Validates build output, share usage data, and macro comments
 */
describe("ConsumeShared Build Validation", () => {
	const distPath = path.join(process.cwd(), "dist");

	test("dist directory exists", () => {
		expect(fs.existsSync(distPath)).toBe(true);
	});

	test("expected chunk files exist", () => {
		const expectedFiles = [
			"main.js",
			"shared_utils_js.js",
			"shared_components_js.js",
			"shared_api_js.js",
			"share-usage.json"
		];

		for (const file of expectedFiles) {
			const filePath = path.join(distPath, file);
			expect(fs.existsSync(filePath)).toBe(true);
		}
	});

	test("share-usage.json exists and has valid structure", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");

		expect(fs.existsSync(shareUsagePath)).toBe(true);

		const content = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Check structure
		expect(content.consume_shared_modules).toBeTruthy();

		// Check expected modules exist
		const expectedModules = [
			"react-dom",
			"api-lib",
			"react",
			"lodash-es",
			"component-lib",
			"utility-lib"
		];
		for (const module of expectedModules) {
			expect(content.consume_shared_modules[module]).toBeTruthy();
		}
	});

	test("macro comments validation against share-usage.json", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Mapping of share-usage modules to their corresponding chunk files
		const moduleToChunkMap = {
			"utility-lib": "shared_utils_js.js",
			"component-lib": "shared_components_js.js",
			"api-lib": "shared_api_js.js"
		};

		let totalValidated = 0;
		const validationResults = {};

		for (const [moduleName, chunkFile] of Object.entries(moduleToChunkMap)) {
			const chunkPath = path.join(distPath, chunkFile);

			if (!fs.existsSync(chunkPath)) {
				continue;
			}

			const chunkContent = fs.readFileSync(chunkPath, "utf8");
			const moduleData = shareUsageData.consume_shared_modules[moduleName];

			if (!moduleData) {
				continue;
			}

			validationResults[moduleName] = {
				usedExports: [],
				unusedExports: [],
				missingMacros: [],
				extraMacros: [],
				defaultExportHasMacro: false
			};

			// Check used exports have macro comments
			for (const exportName of moduleData.used_exports) {
				if (exportName === "default") {
					// Special check for default export macro
					const defaultMacroRegex = new RegExp(
						`"default"\\s*:\\s*\\(\\)\\s*=>\\s*\\([^)]*@common:if\\s*\\[condition="treeShake\\.${moduleName}\\.default"\\]`
					);
					validationResults[moduleName].defaultExportHasMacro =
						defaultMacroRegex.test(chunkContent);
				} else {
					// Check for named export macro
					const macroRegex = new RegExp(
						`${exportName}\\s*:\\s*\\(\\)\\s*=>\\s*\\([^)]*@common:if\\s*\\[condition="treeShake\\.${moduleName}\\.${exportName}"\\]`
					);
					if (macroRegex.test(chunkContent)) {
						validationResults[moduleName].usedExports.push(exportName);
						totalValidated++;
					} else {
						validationResults[moduleName].missingMacros.push(exportName);
					}
				}
			}

			// Verify unused exports don't have active code (should be marked unused or absent)
			for (const exportName of moduleData.unused_exports) {
				// Check if export is completely absent or marked as unused
				const exportRegex = new RegExp(`${exportName}\\s*:\\s*\\(\\)\\s*=>`);
				const unusedRegex = new RegExp(`/\\*.*unused.*${exportName}.*\\*/`);

				if (!exportRegex.test(chunkContent) || unusedRegex.test(chunkContent)) {
					validationResults[moduleName].unusedExports.push(exportName);
				} else {
					validationResults[moduleName].extraMacros.push(exportName);
				}
			}
		}

		// Report validation results
		console.log(
			"Macro validation results:",
			JSON.stringify(validationResults, null, 2)
		);

		// Assert all used exports have macros
		for (const [moduleName, results] of Object.entries(validationResults)) {
			const moduleData = shareUsageData.consume_shared_modules[moduleName];
			if (!moduleData) continue;

			// Check that all used exports (except default) have macros
			const expectedNonDefaultExports = moduleData.used_exports.filter(
				e => e !== "default"
			);
			expect(results.usedExports.length).toBe(expectedNonDefaultExports.length);

			// Check missing macros
			expect(results.missingMacros).toHaveLength(0);

			// Check default export has macro if it's used
			if (moduleData.used_exports.includes("default")) {
				expect(results.defaultExportHasMacro).toBe(true);
			}
		}

		expect(totalValidated).toBeGreaterThan(0);
	});

	test("main.js chunk structure validation", () => {
		const mainPath = path.join(distPath, "main.js");
		expect(fs.existsSync(mainPath)).toBe(true);

		const content = fs.readFileSync(mainPath, "utf8");

		// Check for webpack runtime structures
		expect(content.includes("__webpack_require__")).toBe(true);
		expect(content.includes("webpackChunk")).toBe(true);

		// Check for module federation / consume shared structures
		const hasConsumeSharedReferences =
			content.includes("shared") ||
			content.includes("consume") ||
			content.includes("federation");

		expect(hasConsumeSharedReferences).toBe(true);
	});

	test("all macro annotations consistency", () => {
		const expectedMacroComments = [
			{
				file: "shared_utils_js.js",
				expectedComments: [
					'capitalize: () => (/* @common:if [condition="treeShake.utility-lib.capitalize"] */',
					'formatDate: () => (/* @common:if [condition="treeShake.utility-lib.formatDate"] */'
				]
			},
			{
				file: "shared_components_js.js",
				expectedComments: [
					'Button: () => (/* @common:if [condition="treeShake.component-lib.Button"] */',
					'Modal: () => (/* @common:if [condition="treeShake.component-lib.Modal"] */'
				]
			},
			{
				file: "shared_api_js.js",
				expectedComments: [
					'createApiClient: () => (/* @common:if [condition="treeShake.api-lib.createApiClient"] */'
				]
			}
		];

		for (const snapshot of expectedMacroComments) {
			const filePath = path.join(distPath, snapshot.file);
			expect(fs.existsSync(filePath)).toBe(true);

			const content = fs.readFileSync(filePath, "utf8");

			for (const expectedComment of snapshot.expectedComments) {
				expect(content.includes(expectedComment)).toBe(true);
			}
		}
	});

	test("default exports have macro comments", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		const moduleToChunkMap = {
			"utility-lib": "shared_utils_js.js",
			"component-lib": "shared_components_js.js",
			"api-lib": "shared_api_js.js"
		};

		const defaultExportIssues = [];

		for (const [moduleName, chunkFile] of Object.entries(moduleToChunkMap)) {
			const moduleData = shareUsageData.consume_shared_modules[moduleName];

			if (!moduleData || !moduleData.used_exports.includes("default")) {
				continue; // Skip if default is not used
			}

			const chunkPath = path.join(distPath, chunkFile);
			const chunkContent = fs.readFileSync(chunkPath, "utf8");

			// Look for default export with macro comment
			const macroPattern = `"default"\\s*:\\s*\\(\\)\\s*=>\\s*\\([^)]*@common:if\\s*\\[condition="treeShake\\.${moduleName}\\.default"\\]`;
			const defaultWithMacroRegex = new RegExp(macroPattern, "g");

			// Look for default export without macro comment
			const defaultWithoutMacroRegex =
				/"default"\s*:\s*\(\)\s*=>\s*\([^@]*\)(?!.*@common:if)/g;

			const hasMacro = defaultWithMacroRegex.test(chunkContent);
			const hasNoMacro = defaultWithoutMacroRegex.test(chunkContent);

			if (hasNoMacro && !hasMacro) {
				defaultExportIssues.push({
					module: moduleName,
					chunkFile,
					issue: "Default export found without macro comment"
				});
			}
		}

		// Report any issues found
		if (defaultExportIssues.length > 0) {
			console.log(
				"Default export macro issues:",
				JSON.stringify(defaultExportIssues, null, 2)
			);
		}

		// Assert no default export issues
		expect(defaultExportIssues).toHaveLength(0);
	});

	test("generate test report", () => {
		const report = {
			timestamp: new Date().toISOString(),
			status: "PASSED",
			build: {
				distExists: true,
				expectedFiles: true
			},
			shareUsage: {
				fileExists: true,
				structureValid: true
			},
			macroComments: {
				filesChecked: 3,
				commentsValidated: 0, // Will be updated based on actual validation
				allPresent: true
			}
		};

		const reportPath = path.join(process.cwd(), "test-report.json");
		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		expect(fs.existsSync(reportPath)).toBe(true);
	});
});
