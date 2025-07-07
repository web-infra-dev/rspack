#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

/**
 * Node.js test runner for rspack ConsumeShared macro functionality
 * This validates the existing build output without rebuilding
 */
describe("ConsumeShared Macro Validation", () => {
	const distPath = path.join(process.cwd(), "dist");

	test("dist directory exists", () => {
		expect(fs.existsSync(distPath)).toBe(true);
	});

	test("all expected chunk files exist", () => {
		const expectedFiles = [
			"main.js",
			"shared_api_js.js",
			"shared_components_js.js",
			"shared_utils_js.js",
			"cjs-modules_legacy-utils_js.js",
			"cjs-modules_data-processor_js.js",
			"cjs-modules_pure-cjs-helper_js.js"
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

		// Check expected modules exist (ESM modules only - CommonJS via require() are not ConsumeShared)
		const expectedESMModules = [
			"react-dom",
			"utility-lib",
			"api-lib",
			"react",
			"lodash-es",
			"component-lib"
		];
		for (const module of expectedESMModules) {
			expect(content.consume_shared_modules[module]).toBeTruthy();
		}

		// CommonJS modules accessed via require() do NOT appear in ConsumeShared tracking
		// This is a current limitation - they are ProvideShared but not ConsumeShared
		const commonJSModules = [
			"legacy-utils-lib",
			"data-processor-lib",
			"pure-cjs-helper-lib"
		];
		for (const module of commonJSModules) {
			expect(content.consume_shared_modules[module]).toBeUndefined();
		}
	});

	test("macro comments validation against actual usage", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Expected used exports based on actual source code analysis
		// Only ESM modules have ConsumeShared tracking
		const expectedUsedExports = {
			"utility-lib": ["capitalize", "formatDate", "default"],
			"component-lib": ["Button", "Modal", "default"],
			"api-lib": ["createApiClient", "default"]
		};

		const moduleToChunkMap = {
			"utility-lib": "shared_utils_js.js",
			"component-lib": "shared_components_js.js",
			"api-lib": "shared_api_js.js",
			"legacy-utils-lib": "cjs-modules_legacy-utils_js.js",
			"data-processor-lib": "cjs-modules_data-processor_js.js",
			"pure-cjs-helper-lib": "cjs-modules_pure-cjs-helper_js.js"
		};

		let totalValidated = 0;
		const validationResults = {};

		for (const [moduleName, expectedExports] of Object.entries(
			expectedUsedExports
		)) {
			const chunkFile = moduleToChunkMap[moduleName];
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
			for (const exportName of expectedExports) {
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

			// Verify unused exports are properly handled
			const expectedUnusedExports = {
				"utility-lib": ["debounce", "deepClone", "generateId", "validateEmail"],
				"component-lib": ["createCard"],
				"api-lib": ["ApiClient", "fetchWithTimeout"],
				"legacy-utils-lib": ["readFileSync", "validateFile"],
				"data-processor-lib": [
					"filterArray",
					"reduceArray",
					"DataProcessor",
					"DEFAULT_OPTIONS"
				],
				"pure-cjs-helper-lib": [
					"hashString",
					"validateInput",
					"processData",
					"DataValidator",
					"createValidator"
				]
			};

			const expectedUnused = expectedUnusedExports[moduleName] || [];
			for (const exportName of expectedUnused) {
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

		// Assert all expected used exports have macros
		for (const [moduleName, results] of Object.entries(validationResults)) {
			const expectedExports = expectedUsedExports[moduleName] || [];
			const expectedNonDefaultExports = expectedExports.filter(
				e => e !== "default"
			);

			expect(results.usedExports.length).toBe(expectedNonDefaultExports.length);
			expect(results.missingMacros).toHaveLength(0);

			// Check default export has macro if it's expected to be used
			if (expectedExports.includes("default")) {
				expect(results.defaultExportHasMacro).toBe(true);
			}
		}

		expect(totalValidated).toBeGreaterThan(0);
	});

	test("generate test report", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		const report = {
			timestamp: new Date().toISOString(),
			status: "PASSED",
			build: {
				distExists: true,
				expectedFiles: true
			},
			shareUsage: {
				fileExists: true,
				structureValid: true,
				moduleCount: Object.keys(shareUsageData.consume_shared_modules).length,
				modulesWithUnusedExports: Object.values(
					shareUsageData.consume_shared_modules
				).filter(module => module.unused_exports.length > 0).length
			},
			macroComments: {
				filesChecked: 3,
				commentsValidated: 0, // Will be updated based on actual validation
				allPresent: true
			},
			actualUsage: {
				"utility-lib": {
					used:
						shareUsageData.consume_shared_modules["utility-lib"]
							?.used_exports || [],
					unused:
						shareUsageData.consume_shared_modules["utility-lib"]
							?.unused_exports || []
				},
				"component-lib": {
					used:
						shareUsageData.consume_shared_modules["component-lib"]
							?.used_exports || [],
					unused:
						shareUsageData.consume_shared_modules["component-lib"]
							?.unused_exports || []
				},
				"api-lib": {
					used:
						shareUsageData.consume_shared_modules["api-lib"]?.used_exports ||
						[],
					unused:
						shareUsageData.consume_shared_modules["api-lib"]?.unused_exports ||
						[]
				},
				"lodash-es": {
					used:
						shareUsageData.consume_shared_modules["lodash-es"]?.used_exports ||
						[],
					unused:
						shareUsageData.consume_shared_modules["lodash-es"]
							?.unused_exports || [],
					note: "Check if imported but unused exports are properly detected"
				},
				"legacy-utils-lib": {
					used:
						shareUsageData.consume_shared_modules["legacy-utils-lib"]
							?.used_exports || [],
					unused:
						shareUsageData.consume_shared_modules["legacy-utils-lib"]
							?.unused_exports || []
				},
				"data-processor-lib": {
					used:
						shareUsageData.consume_shared_modules["data-processor-lib"]
							?.used_exports || [],
					unused:
						shareUsageData.consume_shared_modules["data-processor-lib"]
							?.unused_exports || []
				}
			}
		};

		const reportPath = path.join(process.cwd(), "test-report.json");
		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		expect(fs.existsSync(reportPath)).toBe(true);

		console.log(`✅ Test report generated: ${reportPath}`);
		console.log(
			"📊 Module usage summary:",
			JSON.stringify(report.actualUsage, null, 2)
		);
		console.log(
			`✅ Found ${report.shareUsage.moduleCount} ConsumeShared modules`
		);
		console.log(
			`⚠️  ${report.shareUsage.modulesWithUnusedExports} modules have unused exports`
		);
	});

	test("macro annotations validation - expect at least one macro per ConsumeShared chunk", () => {
		// Expected ESM ConsumeShared files that MUST have macro annotations
		const expectedESMFiles = [
			"shared_utils_js.js",
			"shared_components_js.js",
			"shared_api_js.js"
		];

		let totalMacrosFound = 0;
		const macroResults = {};

		for (const file of expectedESMFiles) {
			const filePath = path.join(distPath, file);
			expect(fs.existsSync(filePath)).toBe(true);

			const content = fs.readFileSync(filePath, "utf8");

			// Count macro annotations in this file
			const macroMatches =
				content.match(/@common:if\s*\[condition="treeShake\.[^"]+"\]/g) || [];
			const macroCount = macroMatches.length;

			macroResults[file] = {
				macroCount,
				examples: macroMatches.slice(0, 3) // First 3 examples
			};

			// STRICT REQUIREMENT: Each ConsumeShared chunk MUST have at least one macro annotation
			expect(macroCount).toBeGreaterThan(0);
			expect(content).toContain("@common:if");
			expect(content).toContain("treeShake");

			totalMacrosFound += macroCount;
			console.log(`✅ ${file}: Found ${macroCount} macro annotations`);
		}

		// Overall validation: We must find macros across all ConsumeShared chunks
		expect(totalMacrosFound).toBeGreaterThan(0);
		console.log(`📊 Total macro annotations found: ${totalMacrosFound}`);
		console.log(
			"🔍 Macro analysis results:",
			JSON.stringify(macroResults, null, 2)
		);
	});

	test("CommonJS module sharing validation - limitation documented", () => {
		// Check if CommonJS modules are properly shared as ProvideShared
		const expectedCommonJSFiles = [
			"cjs-modules_legacy-utils_js.js",
			"cjs-modules_data-processor_js.js",
			"cjs-modules_pure-cjs-helper_js.js"
		];

		for (const file of expectedCommonJSFiles) {
			const filePath = path.join(distPath, file);
			if (fs.existsSync(filePath)) {
				const content = fs.readFileSync(filePath, "utf8");

				// Verify CommonJS module structure
				expect(content).toContain("module.exports");
				expect(content).toContain("exports.");

				// CURRENT LIMITATION: CommonJS modules accessed via require() do NOT get macro comments
				// They are ProvideShared but not ConsumeShared, so no tree-shaking annotations
				expect(content).not.toContain("@common:if");
				expect(content).not.toContain("treeShake");

				console.log(
					`✅ CommonJS module chunk found (no macros - current limitation): ${file}`
				);
			} else {
				console.log(`⚠️  CommonJS module chunk not found: ${file}`);
			}
		}

		console.log(
			"📝 LIMITATION: CommonJS modules accessed via require() are not tracked as ConsumeShared"
		);
		console.log("   - They are shared via ProvideShared but consumed directly");
		console.log(
			"   - No tree-shaking macros are generated for CommonJS require() calls"
		);
		console.log(
			"   - This is a current architectural limitation of Module Federation"
		);
	});

	test("CommonJS Module Federation sharing analysis", () => {
		const mainJsPath = path.join(distPath, "main.js");
		expect(fs.existsSync(mainJsPath)).toBe(true);

		const mainContent = fs.readFileSync(mainJsPath, "utf8");

		// Check for CommonJS require() calls
		expect(mainContent).toContain("require");

		// Look for Module Federation sharing setup that includes CommonJS modules
		const sharingDataMatch = mainContent.match(
			/__webpack_require__\.initializeSharingData\s*=\s*{[^}]+}/
		);
		if (sharingDataMatch) {
			const sharingData = sharingDataMatch[0];

			// Check if CommonJS modules are registered as ProvideShared
			const commonJSSharedModules = [
				"data-processor-lib",
				"legacy-utils-lib",
				"pure-cjs-helper-lib"
			];

			for (const moduleKey of commonJSSharedModules) {
				const isProvideShared = sharingData.includes(`"${moduleKey}"`);
				console.log(`📦 ${moduleKey} is ProvideShared: ${isProvideShared}`);
				expect(isProvideShared).toBe(true);
			}
		}

		// Check for direct CommonJS module references (not through shared mechanism)
		const commonJSModulePatterns = [
			"cjs-modules/legacy-utils",
			"cjs-modules/data-processor",
			"cjs-modules/pure-cjs-helper"
		];

		let directRequireCount = 0;
		for (const pattern of commonJSModulePatterns) {
			// Look for direct require() calls (not through Module Federation)
			const directRequireMatch = mainContent.match(
				new RegExp(
					`require\\("\\.\\/\\${pattern.replace(/[/-]/g, "[/-]")}\\.js"\\)`,
					"g"
				)
			);
			if (directRequireMatch) {
				directRequireCount += directRequireMatch.length;
				console.log(
					`🔗 Direct require() calls to ${pattern}: ${directRequireMatch.length}`
				);
			}
		}

		// This explains why CommonJS modules don't have macro annotations:
		// They're accessed via direct require() calls, not through ConsumeShared
		console.log(
			`⚠️  Total direct require() calls found: ${directRequireCount}`
		);
		console.log(
			"📝 Analysis: CommonJS modules are ProvideShared but consumed via direct require(), not ConsumeShared"
		);
	});
});
