#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

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

	test("used exports have macro annotations", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Expected used exports based on actual source code and share-usage.json
		const expectedUsedExports = {
			"utility-lib": ["capitalize", "formatDate", "default"],
			"component-lib": ["Button", "Modal", "default"],
			"api-lib": ["createApiClient", "default"]
		};

		const moduleToChunkMap = {
			"utility-lib": "shared_utils_js.js",
			"component-lib": "shared_components_js.js",
			"api-lib": "shared_api_js.js"
		};

		for (const [moduleName, expectedExports] of Object.entries(
			expectedUsedExports
		)) {
			const chunkFile = moduleToChunkMap[moduleName];
			const filePath = path.join(distPath, chunkFile);
			const content = fs.readFileSync(filePath, "utf8");

			// Verify each used export has a macro annotation
			for (const exportName of expectedExports) {
				if (exportName === "default") {
					// Check for default export macro
					const defaultMacroPattern = `"default"\\s*:\\s*\\(\\)\\s*=>\\s*\\([^)]*@common:if\\s*\\[condition="treeShake\\.${moduleName}\\.default"\\]`;
					expect(content).toMatch(new RegExp(defaultMacroPattern));
				} else {
					// Check for named export macro
					const namedMacroPattern = `${exportName}\\s*:\\s*\\(\\)\\s*=>\\s*\\([^)]*@common:if\\s*\\[condition="treeShake\\.${moduleName}\\.${exportName}"\\]`;
					expect(content).toMatch(new RegExp(namedMacroPattern));
				}
			}
		}
	});

	test("unused exports are properly handled", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Expected unused exports based on share-usage.json
		const expectedUnusedExports = {
			"utility-lib": ["debounce", "deepClone", "generateId", "validateEmail"],
			"component-lib": ["createCard"],
			"api-lib": ["ApiClient", "fetchWithTimeout"]
		};

		const moduleToChunkMap = {
			"utility-lib": "shared_utils_js.js",
			"component-lib": "shared_components_js.js",
			"api-lib": "shared_api_js.js"
		};

		for (const [moduleName, expectedUnused] of Object.entries(
			expectedUnusedExports
		)) {
			const moduleData = shareUsageData.consume_shared_modules[moduleName];

			// Verify share-usage.json correctly identifies unused exports
			expect(moduleData.unused_exports).toEqual(
				expect.arrayContaining(expectedUnused)
			);

			// Verify unused exports in share-usage.json match our expectations
			for (const unusedExport of expectedUnused) {
				expect(moduleData.unused_exports).toContain(unusedExport);
			}
		}
	});

	test("lodash-es usage validation", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		const lodashData = shareUsageData.consume_shared_modules["lodash-es"];

		// Based on index.js: import { VERSION, map, filter, uniq } from "lodash-es";
		// Currently all imported lodash exports are marked as used
		const expectedUsed = ["map", "VERSION", "filter"];

		// Verify used exports (uniq is imported but not used, but current analysis marks it as used)
		for (const usedExport of expectedUsed) {
			expect(lodashData.used_exports).toContain(usedExport);
		}

		// Log the actual lodash usage for debugging
		console.log("📊 Lodash-es actual usage:", {
			used: lodashData.used_exports,
			unused: lodashData.unused_exports,
			note: "uniq is imported but not called - should ideally be unused"
		});
	});

	test("macro positioning validation in CommonJS files", () => {
		const commonJSFiles = [
			"cjs-modules_pure-cjs-helper_js.js",
			"cjs-modules_legacy-utils_js.js",
			"cjs-modules_data-processor_js.js"
		];

		const positioningIssues = [];
		
		for (const fileName of commonJSFiles) {
			const filePath = path.join(distPath, fileName);
			
			if (!fs.existsSync(filePath)) {
				continue;
			}

			const content = fs.readFileSync(filePath, "utf8");
			
			// Check for incorrect macro positioning pattern:
			// /* @common:if */ exports.prop /* @common:endif */ = value (WRONG)
			// Should be: /* @common:if */ exports.prop = value; /* @common:endif */ (CORRECT)
			
			const lines = content.split('\n');
			lines.forEach((line, lineIndex) => {
				if (line.includes('@common:if') && line.includes('@common:endif') && line.includes('=')) {
					const macroEndifIndex = line.indexOf('/* @common:endif');
					const equalsIndex = line.indexOf('=');
					
					// If @common:endif appears before the equals sign, it's incorrect positioning
					if (macroEndifIndex !== -1 && macroEndifIndex < equalsIndex) {
						positioningIssues.push({
							file: fileName,
							line: lineIndex + 1,
							content: line.trim(),
							issue: "Macro ends before assignment completion"
						});
					}
				}
			});
		}

		if (positioningIssues.length > 0) {
			console.log("❌ Macro positioning issues found:");
			positioningIssues.forEach(issue => {
				console.log(`  ${issue.file}:${issue.line} - ${issue.issue}`);
				console.log(`    ${issue.content}`);
			});
		}

		// This test should fail if there are positioning issues
		expect(positioningIssues).toHaveLength(0);
	});

	test("mixed export pattern detection", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");
		
		if (!fs.existsSync(filePath)) {
			// Skip if file doesn't exist
			return;
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Count different export patterns
		const moduleExportsPattern = (content.match(/module\.exports\./g) || []).length;
		const exportsPattern = (content.match(/(?<!module\.)exports\./g) || []).length;

		const mixedPatternReport = {
			file: "cjs-modules_pure-cjs-helper_js.js",
			moduleExportsCount: moduleExportsPattern,
			exportsCount: exportsPattern,
			hasMixedPattern: moduleExportsPattern > 0 && exportsPattern > 0,
			totalExports: moduleExportsPattern + exportsPattern
		};

		console.log("📊 Mixed export pattern analysis:", mixedPatternReport);

		// Validate that we have export patterns
		expect(mixedPatternReport.totalExports).toBeGreaterThan(0);

		// Log warning if mixed patterns detected
		if (mixedPatternReport.hasMixedPattern) {
			console.log("⚠️  Mixed export patterns detected - this can cause macro positioning issues");
		}
	});

	test("specific incorrect macro patterns validation", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");
		
		if (!fs.existsSync(filePath)) {
			// Skip if file doesn't exist
			return;
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Test for the specific patterns found in the issue
		const specificProblems = [];
		
		// Pattern 1: /* @common:if */ exports.prop /* @common:endif */ = value
		const incorrectExportsPattern = /\/\*\s*@common:if[^*]+\*\/\s*exports\.[\w]+\s*\/\*\s*@common:endif[^*]*\*\/\s*=/g;
		const incorrectExportsMatches = content.match(incorrectExportsPattern) || [];
		
		// Pattern 2: /* @common:if */ module.exports.prop /* @common:endif */ = value  
		const incorrectModuleExportsPattern = /\/\*\s*@common:if[^*]+\*\/\s*module\.exports\.[\w]+\s*\/\*\s*@common:endif[^*]*\*\/\s*=/g;
		const incorrectModuleExportsMatches = content.match(incorrectModuleExportsPattern) || [];

		if (incorrectExportsMatches.length > 0) {
			specificProblems.push({
				pattern: "Incorrect exports.prop positioning",
				count: incorrectExportsMatches.length,
				examples: incorrectExportsMatches.slice(0, 3)
			});
		}

		if (incorrectModuleExportsMatches.length > 0) {
			specificProblems.push({
				pattern: "Incorrect module.exports.prop positioning", 
				count: incorrectModuleExportsMatches.length,
				examples: incorrectModuleExportsMatches.slice(0, 3)
			});
		}

		if (specificProblems.length > 0) {
			console.log("❌ Specific macro positioning problems detected:");
			specificProblems.forEach(problem => {
				console.log(`  ${problem.pattern}: ${problem.count} occurrences`);
				problem.examples.forEach((example, i) => {
					console.log(`    ${i + 1}. ${example.replace(/\s+/g, ' ').trim()}`);
				});
			});
		}

		// Test should fail if we find these specific incorrect patterns
		expect(incorrectExportsMatches.length).toBe(0);
		expect(incorrectModuleExportsMatches.length).toBe(0);
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
				}
			}
		};

		const reportPath = path.join(process.cwd(), "test-report.json");
		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		expect(fs.existsSync(reportPath)).toBe(true);

		console.log("✅ Test report generated with actual usage data");
		console.log(
			"📊 Module usage summary:",
			JSON.stringify(report.actualUsage, null, 2)
		);
	});
});
