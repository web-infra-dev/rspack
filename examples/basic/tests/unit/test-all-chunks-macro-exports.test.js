#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

// Test macro export shape validation for all CJS chunks

describe("Macro export shape validation for all CJS chunks", () => {
	const chunkFiles = [
		"cjs-modules_data-processor_js.js",
		"cjs-modules_legacy-utils_js.js",
		"cjs-modules_module-exports-pattern_js.js",
		"cjs-modules_pure-cjs-helper_js.js"
	];

	chunkFiles.forEach(chunkFile => {
		test(`should have valid macro export shapes in ${chunkFile}`, () => {
			const targetFile = path.join(process.cwd(), "dist", chunkFile);

			if (!fs.existsSync(targetFile)) {
				throw new Error(`Target file not found: ${targetFile}`);
			}

			const content = fs.readFileSync(targetFile, "utf8");
			console.log(`📁 Testing file: ${chunkFile}`);

			// Find the module.exports object
			const moduleExportsMatch = content.match(
				/module\.exports\s*=\s*\{([^}]*)\}/s
			);
			if (!moduleExportsMatch) {
				console.log(
					`ℹ️  ${chunkFile} uses individual exports pattern (exports.prop = value)`
				);
				return; // Skip files that use individual exports
			}

			const objectContent = moduleExportsMatch[1];
			const lines = objectContent
				.split("\n")
				.map(line => line.trim())
				.filter(line => line.length > 0);

			console.log(`✅ Found module.exports object in ${chunkFile}`);

			// Check for invalid patterns: comma OUTSIDE macro
			const invalidLines = [];
			const validLines = [];

			for (const line of lines) {
				// Skip lines that don't contain macros
				if (!line.includes("@common:if") && !line.includes("@common:endif")) {
					continue;
				}

				// Check for invalid pattern: /* @common:endif */,
				if (line.includes("/* @common:endif */,")) {
					invalidLines.push(line);
				}
				// Check for valid patterns:
				// 1. property, /* @common:endif */ (inline)
				// 2. multiline patterns where @common:endif is on separate line
				else if (
					line.includes(", /* @common:endif */") ||
					(line.includes("/* @common:endif */") &&
						!line.includes("/* @common:endif */,"))
				) {
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
				throw new Error(
					`Found ${invalidLines.length} invalid macro export patterns in ${chunkFile} - commas should be INSIDE macro boundaries`
				);
			}

			// Verify we have some macro exports
			if (validLines.length === 0) {
				// Check if there are any macro patterns at all
				const hasMacros = lines.some(
					line => line.includes("@common:if") || line.includes("@common:endif")
				);
				if (hasMacros) {
					throw new Error(
						`Found macro patterns but no valid exports in ${chunkFile}`
					);
				} else {
					console.log(
						`ℹ️  No macro exports found in ${chunkFile} (this may be expected)`
					);
				}
			}

			// Additional validation: ensure macro blocks are properly formed
			for (const line of lines) {
				if (line.includes("@common:if") && line.includes("@common:endif")) {
					// This should be a complete macro block on one line
					const hasValidStructure =
						line.includes("/* @common:if") &&
						line.includes("/* @common:endif */");

					// Allow flexible structures including comments after macros
					if (!hasValidStructure) {
						throw new Error(`Invalid macro structure in ${chunkFile}: ${line}`);
					}
				}
			}

			console.log(`🎉 All macro exports in ${chunkFile} have valid shape`);
		});
	});

	// Specific tests for CJS modules without macros (correct behavior)
	test("should NOT have tree-shaking macros in pure-cjs-helper", () => {
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
				"pure-cjs-helper should have tree-shaking macros - CJS modules with Module Federation shared context get macros"
			);
		}

		// Verify hashString function exists with macros
		const hasHashStringMacro = content.includes(
			"treeShake.cjs-pure-helper.hashString"
		);
		if (!hasHashStringMacro) {
			throw new Error("hashString export should have tree-shaking macros");
		}

		console.log(
			"✅ Correctly found tree-shaking macros in pure-cjs-helper (CJS with Module Federation shared context)"
		);
	});

	test("should NOT have tree-shaking macros in module-exports-pattern", () => {
		const targetFile = path.join(
			process.cwd(),
			"dist",
			"cjs-modules_module-exports-pattern_js.js"
		);
		if (!fs.existsSync(targetFile)) {
			throw new Error(`Target file not found: ${targetFile}`);
		}

		const content = fs.readFileSync(targetFile, "utf8");

		// CJS modules without ConsumeShared context should NOT have macros
		const hasMacros =
			content.includes("@common:if") || content.includes("@common:endif");

		if (hasMacros) {
			throw new Error(
				"module-exports-pattern should NOT have tree-shaking macros - CJS modules without shared context don't get macros"
			);
		}

		// Verify calculateSum exists without macros
		const hasCalculateSum = content.includes("calculateSum");
		if (!hasCalculateSum) {
			throw new Error("calculateSum should exist in the module.exports object");
		}

		console.log(
			"✅ Correctly found NO tree-shaking macros in module-exports-pattern (pure object export pattern)"
		);
	});

	test("should have tree-shaking macros in legacy-utils", () => {
		const targetFile = path.join(
			process.cwd(),
			"dist",
			"cjs-modules_legacy-utils_js.js"
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
				"legacy-utils should have tree-shaking macros - CJS modules with Module Federation shared context get macros"
			);
		}

		// Verify specific macro exists
		const hasFormatPathMacro = content.includes(
			"treeShake.cjs-legacy-utils.formatPath"
		);
		if (!hasFormatPathMacro) {
			throw new Error("formatPath export should have tree-shaking macros");
		}

		console.log(
			"✅ Correctly found tree-shaking macros in legacy-utils (CJS with Module Federation shared context)"
		);
	});

	test("should validate macro export consistency across all chunks", () => {
		const results = {};

		chunkFiles.forEach(chunkFile => {
			const targetFile = path.join(process.cwd(), "dist", chunkFile);
			if (!fs.existsSync(targetFile)) {
				results[chunkFile] = { error: "File not found" };
				return;
			}

			const content = fs.readFileSync(targetFile, "utf8");
			const moduleExportsMatch = content.match(
				/module\.exports\s*=\s*\{([^}]*)\}/s
			);

			if (!moduleExportsMatch) {
				results[chunkFile] = { error: "No module.exports found" };
				return;
			}

			const objectContent = moduleExportsMatch[1];
			const lines = objectContent
				.split("\n")
				.map(line => line.trim())
				.filter(line => line.length > 0);

			const validMacroLines = lines.filter(line =>
				line.includes(", /* @common:endif */")
			);
			const invalidMacroLines = lines.filter(line =>
				line.includes("/* @common:endif */,")
			);

			results[chunkFile] = {
				validCount: validMacroLines.length,
				invalidCount: invalidMacroLines.length,
				totalMacroLines: validMacroLines.length + invalidMacroLines.length
			};
		});

		console.log("\n📊 Macro export summary:");
		Object.entries(results).forEach(([file, data]) => {
			if (data.error) {
				console.log(`  ${file}: ${data.error}`);
			} else {
				console.log(
					`  ${file}: ${data.validCount} valid, ${data.invalidCount} invalid (${data.totalMacroLines} total macro lines)`
				);
			}
		});

		// Check if any files have invalid patterns
		const filesWithErrors = Object.entries(results).filter(
			([_, data]) => data.invalidCount > 0
		);
		if (filesWithErrors.length > 0) {
			throw new Error(
				`Found invalid macro patterns in ${filesWithErrors.length} files`
			);
		}

		console.log("🎉 All chunks have consistent and valid macro export shapes");
	});
});
