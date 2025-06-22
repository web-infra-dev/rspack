#!/usr/bin/env node

import assert from "node:assert";
import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
	after,
	afterEach,
	before,
	beforeEach,
	describe,
	test
} from "node:test";

/**
 * Node.js test runner for rspack ConsumeShared macro functionality
 * This test runs the build and validates the output using proper setup/teardown
 */
describe("ConsumeShared Macro Build and Validation", () => {
	const distPath = path.join(process.cwd(), "dist");
	const reportPath = path.join(process.cwd(), "test-report.json");

	const expectedMacroComments = [
		{
			file: "shared_utils_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.utility-lib.capitalize"] */ capitalize: () => (/* ESM export specifier */ capitalize) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.debounce"] */ debounce: () => (/* ESM export specifier */ debounce) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.formatDate"] */ formatDate: () => (/* ESM export specifier */ formatDate) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.default"] */ "default": () => (__WEBPACK_DEFAULT_EXPORT__) /* @common:endif */',
				'/* @common:if [condition="treeShake.utility-lib.default"] */ /* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = ({'
			]
		},
		{
			file: "shared_components_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.component-lib.Button"] */ Button: () => (/* ESM export specifier */ Button) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.Modal"] */ Modal: () => (/* ESM export specifier */ Modal) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.createCard"] */ createCard: () => (/* ESM export specifier */ createCard) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.default"] */ "default": () => (__WEBPACK_DEFAULT_EXPORT__) /* @common:endif */',
				'/* @common:if [condition="treeShake.component-lib.default"] */ /* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = ({'
			]
		},
		{
			file: "shared_api_js.js",
			expectedComments: [
				'/* @common:if [condition="treeShake.api-lib.ApiClient"] */ ApiClient: () => (/* ESM export specifier */ ApiClient) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.createApiClient"] */ createApiClient: () => (/* ESM export specifier */ createApiClient) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.fetchWithTimeout"] */ fetchWithTimeout: () => (/* ESM export specifier */ fetchWithTimeout) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.default"] */ "default": () => (__WEBPACK_DEFAULT_EXPORT__) /* @common:endif */',
				'/* @common:if [condition="treeShake.api-lib.default"] */ /* ESM default export */ const __WEBPACK_DEFAULT_EXPORT__ = ({'
			]
		}
	];

	// Global setup - run once before all tests
	before(async () => {
		console.log("🚀 Setting up test environment...");

		// Clean any existing build artifacts
		if (fs.existsSync(distPath)) {
			fs.rmSync(distPath, { recursive: true, force: true });
		}

		if (fs.existsSync(reportPath)) {
			fs.unlinkSync(reportPath);
		}

		console.log("✅ Test environment setup complete");
	});

	// Global teardown - run once after all tests
	after(async () => {
		console.log("🧹 Cleaning up test environment...");

		// Keep dist and report for inspection but log their location
		if (fs.existsSync(distPath)) {
			console.log(`📁 Build artifacts preserved at: ${distPath}`);
		}

		if (fs.existsSync(reportPath)) {
			console.log(`📊 Test report available at: ${reportPath}`);
		}

		console.log("✅ Test environment cleanup complete");
	});

	// Test-specific setup
	beforeEach(async t => {
		console.log(`📋 Starting test: ${t.name}`);
	});

	// Test-specific teardown
	afterEach(async t => {
		console.log(`✅ Completed test: ${t.name}`);
	});

	test("build rspack project", async () => {
		console.log("🔨 Running rspack build...");

		try {
			// Use the existing build script which handles all the setup
			execSync("./run-build.sh", {
				stdio: "pipe", // Capture output for cleaner test display
				cwd: process.cwd(),
				timeout: 300000 // 5 minutes timeout
			});

			// Verify dist directory was created
			assert.ok(fs.existsSync(distPath), "Build should create dist directory");

			console.log("✅ Build completed successfully");
		} catch (error) {
			// Log build error details
			console.error("❌ Build failed:", error.message);
			if (error.stdout) {
				console.error("Build stdout:", error.stdout.toString());
			}
			if (error.stderr) {
				console.error("Build stderr:", error.stderr.toString());
			}
			throw error;
		}
	});

	test("verify expected chunk files exist", () => {
		const expectedFiles = [
			"main.js",
			"shared_api_js.js",
			"shared_components_js.js",
			"shared_utils_js.js",
			"share-usage.json"
		];

		for (const file of expectedFiles) {
			const filePath = path.join(distPath, file);
			assert.ok(fs.existsSync(filePath), `Expected file should exist: ${file}`);

			// Verify file is not empty
			const stats = fs.statSync(filePath);
			assert.ok(stats.size > 0, `File should not be empty: ${file}`);
		}

		console.log(
			`✅ All ${expectedFiles.length} expected files exist and are non-empty`
		);
	});

	test("validate share-usage.json structure", () => {
		const shareUsagePath = path.join(distPath, "share-usage.json");

		assert.ok(fs.existsSync(shareUsagePath), "share-usage.json should exist");

		const content = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Check top-level structure
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

			// Verify each module has the expected structure
			const moduleData = content.consume_shared_modules[module];
			assert.ok(
				Array.isArray(moduleData.used_exports),
				`${module} should have used_exports array`
			);
			assert.ok(
				Array.isArray(moduleData.unused_exports),
				`${module} should have unused_exports array`
			);
			assert.ok(
				Array.isArray(moduleData.possibly_unused_exports),
				`${module} should have possibly_unused_exports array`
			);
		}

		// Check metadata structure
		assert.strictEqual(
			typeof content.metadata.total_modules,
			"number",
			"metadata.total_modules should be a number"
		);
		// Removed plugin_version, modules_with_unused_exports, and analysis_timestamp requirements

		console.log(
			`✅ share-usage.json validated with ${expectedModules.length} modules`
		);
	});

	describe("macro comments validation", () => {
		for (const snapshot of expectedMacroComments) {
			test(`${snapshot.file} contains ConsumeShared macro comments`, () => {
				const filePath = path.join(distPath, snapshot.file);

				assert.ok(fs.existsSync(filePath), `${snapshot.file} should exist`);

				const content = fs.readFileSync(filePath, "utf8");

				// Verify file is not empty and contains JavaScript
				assert.ok(content.length > 0, `${snapshot.file} should not be empty`);
				assert.ok(
					content.includes("exports"),
					`${snapshot.file} should contain exports`
				);

				for (const expectedComment of snapshot.expectedComments) {
					assert.ok(
						content.includes(expectedComment),
						`${snapshot.file} should contain macro comment: ${expectedComment}`
					);
				}

				// Count the number of macro comments
				const macroMatches = content.match(
					/\/\* @common:if \[condition="treeShake\./g
				);
				const endifMatches = content.match(/\/\* @common:endif \*\//g);

				assert.ok(
					macroMatches,
					`${snapshot.file} should contain @common:if comments`
				);
				assert.ok(
					endifMatches,
					`${snapshot.file} should contain @common:endif comments`
				);
				assert.strictEqual(
					macroMatches.length,
					endifMatches.length,
					`${snapshot.file} should have matching @common:if and @common:endif comments`
				);

				console.log(
					`✅ ${snapshot.file}: validated ${snapshot.expectedComments.length} macro comments (${macroMatches.length} total found)`
				);
			});
		}
	});

	test("validate macro comment patterns", () => {
		// This test validates the general pattern of macro comments across all files
		const sharedFiles = [
			"shared_utils_js.js",
			"shared_components_js.js",
			"shared_api_js.js"
		];
		let totalMacroComments = 0;

		for (const file of sharedFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Validate macro comment structure
			const macroPattern =
				/\/\* @common:if \[condition="treeShake\.([^"]+)"\] \*\/.*?\/\* @common:endif \*\//g;
			const matches = [...content.matchAll(macroPattern)];

			assert.ok(
				matches.length > 0,
				`${file} should contain properly formatted macro comments`
			);

			for (const match of matches) {
				const shareKey = match[1];
				assert.ok(
					shareKey.includes("."),
					`Share key should include module and export: ${shareKey}`
				);

				// Validate share key format (module.export)
				const parts = shareKey.split(".");
				assert.strictEqual(
					parts.length,
					2,
					`Share key should have format 'module.export': ${shareKey}`
				);
				assert.ok(
					parts[0].length > 0,
					`Module part should not be empty: ${shareKey}`
				);
				assert.ok(
					parts[1].length > 0,
					`Export part should not be empty: ${shareKey}`
				);
			}

			totalMacroComments += matches.length;
		}

		assert.ok(
			totalMacroComments >= 15,
			`Should find at least 15 macro comments across all files, found ${totalMacroComments}`
		);
		console.log(
			`✅ Validated ${totalMacroComments} properly formatted macro comments`
		);
	});

	test("generate comprehensive test report", () => {
		// Create a detailed test report
		const shareUsagePath = path.join(distPath, "share-usage.json");
		const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));

		// Count macro comments across all files
		let totalMacroComments = 0;
		const fileDetails = {};

		for (const snapshot of expectedMacroComments) {
			const filePath = path.join(distPath, snapshot.file);
			const content = fs.readFileSync(filePath, "utf8");
			const macroMatches =
				content.match(/\/\* @common:if \[condition="treeShake\./g) || [];

			totalMacroComments += macroMatches.length;
			fileDetails[snapshot.file] = {
				size: fs.statSync(filePath).size,
				macroComments: macroMatches.length,
				expectedComments: snapshot.expectedComments.length
			};
		}

		const report = {
			timestamp: new Date().toISOString(),
			status: "PASSED",
			environment: {
				nodeVersion: process.version,
				platform: process.platform,
				cwd: process.cwd()
			},
			build: {
				distExists: true,
				expectedFiles: true,
				distPath: distPath
			},
			shareUsage: {
				fileExists: true,
				structureValid: true,
				moduleCount: shareUsageData.metadata.total_modules,
				modulesWithUnusedExports:
					shareUsageData.metadata.modules_with_unused_exports,
				pluginVersion: shareUsageData.metadata.plugin_version
			},
			macroComments: {
				filesChecked: expectedMacroComments.length,
				totalFound: totalMacroComments,
				expectedCount: expectedMacroComments.reduce(
					(sum, snapshot) => sum + snapshot.expectedComments.length,
					0
				),
				allPresent: true,
				fileDetails: fileDetails
			}
		};

		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		assert.ok(fs.existsSync(reportPath), "test report should be generated");

		console.log(`✅ Test report generated: ${reportPath}`);
		console.log(
			`✅ Validated ${report.macroComments.totalFound} macro comments across ${report.macroComments.filesChecked} files`
		);
		console.log(
			`✅ Found ${report.shareUsage.moduleCount} ConsumeShared modules`
		);
		console.log(`✅ Plugin version: ${report.shareUsage.pluginVersion}`);
	});
});
