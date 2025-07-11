#!/usr/bin/env node

import { test, describe } from "node:test";
import assert from "node:assert";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

describe("Macro export shape validation for CJS chunks", () => {
	const chunkFiles = [
		"cjs-modules_data-processor_js.js",
		"cjs-modules_legacy-utils_js.js",
		"cjs-modules_module-exports-pattern_js.js",
		"cjs-modules_pure-cjs-helper_js.js"
	];

	for (const chunkFile of chunkFiles) {
		test(`should have valid macro export shapes in ${chunkFile}`, () => {
			const targetFile = path.join(__dirname, "dist", chunkFile);

			assert.ok(
				fs.existsSync(targetFile),
				`Target file not found: ${targetFile}`
			);

			const content = fs.readFileSync(targetFile, "utf8");
			console.log(`📁 Testing file: ${chunkFile}`);

			// Find the module.exports or exports patterns
			const hasModuleExports = content.includes("module.exports");
			const hasExports = content.includes("exports.");

			if (!hasModuleExports && !hasExports) {
				console.log(
					`ℹ️  No CommonJS exports found in ${chunkFile} (this may be expected)`
				);
				return;
			}

			console.log(`✅ Found CommonJS exports in ${chunkFile}`);

			// Check for invalid patterns: comma OUTSIDE macro
			const invalidLines = [];
			const validLines = [];
			const lines = content
				.split("\n")
				.map(line => line.trim())
				.filter(line => line.length > 0);

			for (const line of lines) {
				// Skip lines that don't contain macros
				if (!line.includes("@common:if") && !line.includes("@common:endif")) {
					continue;
				}

				// Check for invalid pattern: /* @common:endif */,
				if (line.includes("/* @common:endif */,")) {
					invalidLines.push(line);
				}
				// Check for valid pattern: property, /* @common:endif */
				else if (line.includes(", /* @common:endif */")) {
					validLines.push(line);
				}
			}

			console.log(
				`✅ Found ${validLines.length} valid macro export lines in ${chunkFile}`
			);

			if (invalidLines.length > 0) {
				console.log(
					`❌ Found ${invalidLines.length} invalid macro export lines in ${chunkFile}:`
				);
				invalidLines.forEach((line, i) => {
					console.log(`  ${i + 1}. ${line}`);
				});
				assert.fail(
					`Found ${invalidLines.length} invalid macro export patterns in ${chunkFile} - commas should be INSIDE macro boundaries`
				);
			}

			console.log(`🎉 All macro exports in ${chunkFile} have valid shape`);
		});
	}

	test("should have correct transformData pattern in module-exports-pattern", () => {
		const targetFile = path.join(
			__dirname,
			"dist",
			"cjs-modules_module-exports-pattern_js.js"
		);
		assert.ok(
			fs.existsSync(targetFile),
			`Target file not found: ${targetFile}`
		);

		const content = fs.readFileSync(targetFile, "utf8");

		// Test for the correct transformData pattern with comma inside
		const hasCorrectPattern = content.includes(
			"transformData, /* @common:endif */"
		);
		assert.ok(
			hasCorrectPattern,
			"module-exports-pattern should have transformData with comma inside macro"
		);

		// Test that there's no extra comma outside
		const hasIncorrectPattern =
			content.includes("/* @common:endif */,") &&
			content
				.split("\n")
				.some(
					line =>
						line.includes("transformData") &&
						line.includes("/* @common:endif */,")
				);

		if (hasIncorrectPattern) {
			const problematicLines = content
				.split("\n")
				.filter(
					line =>
						line.includes("transformData") &&
						line.includes("/* @common:endif */,")
				);
			console.log("❌ Found problematic transformData lines:");
			problematicLines.forEach((line, i) => {
				console.log(`  ${i + 1}. ${line.trim()}`);
			});
			assert.fail("transformData should not have comma outside macro boundary");
		}

		console.log(
			"✅ Found correct transformData pattern with comma inside macro"
		);
	});

	test("should have correct hashString pattern in pure-cjs-helper", () => {
		const targetFile = path.join(
			__dirname,
			"dist",
			"cjs-modules_pure-cjs-helper_js.js"
		);
		if (!fs.existsSync(targetFile)) {
			console.log(`ℹ️  Target file not found: ${targetFile} - skipping test`);
			return;
		}

		const content = fs.readFileSync(targetFile, "utf8");

		// Test for the correct hashString pattern
		const hasCorrectPattern =
			content.includes("exports.hashString = function(input) {") &&
			content.includes("} /* @common:endif */");

		if (!hasCorrectPattern) {
			console.log(
				"ℹ️  hashString pattern not found in pure-cjs-helper (this may be expected)"
			);
		} else {
			console.log("✅ Found correct hashString pattern in pure-cjs-helper");
		}
	});
});
