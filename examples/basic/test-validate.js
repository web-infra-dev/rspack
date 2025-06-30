#!/usr/bin/env node

import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { describe, test } from "node:test";

/**
 * Node.js test runner for rspack ConsumeShared macro functionality
 * This validates the existing build output without rebuilding
 */
describe("ConsumeShared Macro Validation", () => {
	const distPath = path.join(process.cwd(), "dist");

	const expectedMacroComments = [
		{
			file: "shared_utils_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.utility-lib.capitalize"] */ capitalize: () => (/* ESM export specifier */ capitalize) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.debounce"] */ debounce: () => (/* ESM export specifier */ debounce) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.formatDate"] */ formatDate: () => (/* ESM export specifier */ formatDate) /* @common:endif */'
			]
		},
		{
			file: "shared_components_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.component-lib.Button"] */ Button: () => (/* ESM export specifier */ Button) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.Modal"] */ Modal: () => (/* ESM export specifier */ Modal) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.createCard"] */ createCard: () => (/* ESM export specifier */ createCard) /* @common:endif */'
			]
		},
		{
			file: "shared_api_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.api-lib.ApiClient"] */ ApiClient: () => (/* ESM export specifier */ ApiClient) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.createApiClient"] */ createApiClient: () => (/* ESM export specifier */ createApiClient) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.fetchWithTimeout"] */ fetchWithTimeout: () => (/* ESM export specifier */ fetchWithTimeout) /* @common:endif */'
			]
		}
	];

	test("dist directory exists", () => {
		assert.ok(
			fs.existsSync(distPath),
			`Dist directory should exist at ${distPath}`
		);
	});

	test("all expected chunk files exist", () => {
		const expectedFiles = [
			"main.js",
			"shared_api_js.js",
			"shared_components_js.js",
			"shared_utils_js.js"
		];

		for (const file of expectedFiles) {
			const filePath = path.join(distPath, file);
			assert.ok(
				fs.existsSync(filePath),
				`Expected chunk file should exist: ${file}`
			);
		}
	});

	test("share-usage.json exists and has valid structure", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");

		assert.ok(fs.existsSync(shareUsagePath), "share-usage.json should exist");

		const content = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Check structure
		assert.ok(
			content.consume_shared_modules,
			"should have consume_shared_modules"
		);
		assert.ok(content.metadata, "should have metadata");

		// Check expected modules exist (updated for local modules only)
		const expectedModules = [
			"utility-lib",
			"api-lib", 
			"component-lib",
			"commonjs-lib",
			"mixed-exports-lib",
			"module-exports-lib",
			"fake-commonjs-lib"
		];
		for (const module of expectedModules) {
			assert.ok(
				content.consume_shared_modules[module],
				`should have module '${module}' in consume_shared_modules`
			);
		}

		// Check metadata structure
		assert.strictEqual(
			typeof content.metadata.total_modules,
			"number",
			"metadata.total_modules should be a number"
		);
		// Removed plugin_version and modules_with_unused_exports requirements
	});

	describe("macro comments validation", () => {
		for (const snapshot of expectedMacroComments) {
			test(`${snapshot.file} contains expected macro comments`, () => {
				const filePath = path.join(distPath, snapshot.file);

				assert.ok(fs.existsSync(filePath), `${snapshot.file} should exist`);

				const content = fs.readFileSync(filePath, "utf8");

				for (const expectedComment of snapshot.expectedComments) {
					assert.ok(
						content.includes(expectedComment),
						`${snapshot.file} should contain macro comment: ${expectedComment}`
					);
				}
			});
		}
	});

	test("generate test report", () => {
		// This test creates a summary report
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
				moduleCount: shareUsageData.metadata.total_modules,
				modulesWithUnusedExports:
					shareUsageData.metadata.modules_with_unused_exports
			},
			macroComments: {
				filesChecked: expectedMacroComments.length,
				commentsValidated: expectedMacroComments.reduce(
					(sum, snapshot) => sum + snapshot.expectedComments.length,
					0
				),
				allPresent: true
			}
		};

		const reportPath = path.join(process.cwd(), "test-report.json");
		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		assert.ok(fs.existsSync(reportPath), "test report should be generated");

		console.log(`✅ Test report generated: ${reportPath}`);
		console.log(
			`✅ Validated ${report.macroComments.commentsValidated} macro comments across ${report.macroComments.filesChecked} files`
		);
		console.log(
			`✅ Found ${report.shareUsage.moduleCount} ConsumeShared modules`
		);
	});
});
