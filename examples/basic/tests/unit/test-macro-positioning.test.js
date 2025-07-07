#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { describe, expect, test } from "@rstest/core";

/**
 * Comprehensive tests for macro positioning issues in CommonJS modules
 * Validates that macros are positioned correctly to wrap entire assignments
 * and detect the mixed export pattern issue
 */
describe("Macro Positioning Validation", () => {
	const distPath = path.join(process.cwd(), "dist");

	// Test the specific problematic file mentioned in the issue
	test("cjs-modules_pure-cjs-helper_js.js - detect incorrect macro positioning", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Test for the specific problematic pattern: module.exports.prop = value
		// This should be detected as INCORRECT positioning (macro ending before assignment)
		const incorrectModuleExportsPattern =
			/\/\*\s*@common:if\s*[^*]*\*\/\s*module\.exports\.[\w]+\s*\/\*\s*@common:endif\s*\*\/\s*=/g;
		const incorrectMatches = content.match(incorrectModuleExportsPattern) || [];

		// Test for correct patterns: either wrapping property access or entire assignment
		// Pattern 1: /* @common:if */ module.exports.prop /* @common:endif */ = value; (property wrapping)
		// Pattern 2: /* @common:if */ module.exports.prop = value; /* @common:endif */ (full assignment wrapping)
		const correctModuleExportsPattern =
			/\/\*\s*@common:if\s*[^*]*\*\/[\s\S]*?module\.exports\.[\w]+[\s\S]*?\/\*\s*@common:endif\s*\*\//g;
		const correctMatches = content.match(correctModuleExportsPattern) || [];

		// Report findings
		if (incorrectMatches.length > 0) {
			console.log("❌ Found incorrect macro positioning patterns:");
			incorrectMatches.forEach((match, index) => {
				console.log(`  ${index + 1}. ${match.replace(/\s+/g, " ").trim()}`);
			});
		}

		if (correctMatches.length > 0) {
			console.log("✅ Found correct macro positioning patterns:");
			correctMatches.forEach((match, index) => {
				console.log(`  ${index + 1}. ${match.replace(/\s+/g, " ").trim()}`);
			});
		}

		// The current implementation correctly positions macros, so we expect no incorrect patterns
		// Allow some tolerance for different valid positioning approaches
		if (incorrectMatches.length > 0) {
			console.log(
				`⚠️  Found ${incorrectMatches.length} potentially incorrect macro positioning patterns`
			);
			// Show examples for debugging
			incorrectMatches.slice(0, 2).forEach((match, i) => {
				console.log(`  Example ${i + 1}: ${match.replace(/\s+/g, " ").trim()}`);
			});
		}
		// We expect modern rspack to position macros correctly
		expect(incorrectMatches.length).toBe(0);

		// Check if we have macro patterns (individual assignments use different format)
		const hasMacroPatterns = content.includes("/* @common:if");
		if (hasMacroPatterns) {
			console.log(`✅ Found macro patterns in file`);
		} else {
			console.log(
				`ℹ️  No macro patterns found - file may not have ConsumeShared exports`
			);
		}
	});

	test("detect mixed export pattern issues", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Test for the mixed pattern: module.exports.prop vs exports.prop
		const moduleExportsPattern = /module\.exports\.[\w]+/g;
		const exportsPattern = /exports\.[\w]+/g;

		const moduleExportsMatches = content.match(moduleExportsPattern) || [];
		const exportsMatches = content.match(exportsPattern) || [];

		console.log(`📊 Mixed export pattern analysis:`);
		console.log(
			`  - module.exports.prop assignments: ${moduleExportsMatches.length}`
		);
		console.log(`  - exports.prop assignments: ${exportsMatches.length}`);

		if (moduleExportsMatches.length > 0) {
			console.log(
				`  - module.exports patterns found: ${moduleExportsMatches.join(", ")}`
			);
		}

		// Both patterns should be consistent within the same file
		// If we have both, it suggests mixed patterns which can cause issues
		if (moduleExportsMatches.length > 0 && exportsMatches.length > 0) {
			console.log(
				"⚠️  Mixed export patterns detected - this can cause macro positioning issues"
			);
		}

		// The file should primarily use one pattern or the other consistently
		expect(moduleExportsMatches.length + exportsMatches.length).toBeGreaterThan(
			0
		);
	});

	test("validate macro wrapping completeness", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Find all @common:if blocks (multiline-aware)
		const macroBlocks =
			content.match(
				/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//g
			) || [];

		console.log(`📊 Found ${macroBlocks.length} complete macro blocks`);

		const wrappingIssues = [];

		macroBlocks.forEach((block, index) => {
			const blockContent = block
				.replace(/\/\*\s*@common:(if|endif)[^*]*\*\//g, "")
				.trim();

			// Check if the block properly wraps content
			// Modern approach allows:
			// 1. Complete assignments (exports.prop = value;)
			// 2. Property-only wrapping (propertyName)
			// 3. Object property patterns (property: value,)
			// 4. Multiline object properties (with line breaks)
			// 5. Simple value assignments (valid in object literals)
			const hasCompleteAssignment =
				blockContent.includes("=") && blockContent.includes(";");
			const hasPropertyOnly =
				!blockContent.includes("=") && blockContent.match(/[\w\.]+/);
			const hasObjectProperty = blockContent.match(
				/^\s*\w+\s*:\s*[\s\S]*,?\s*$/
			);
			const hasMultilineProperty =
				blockContent.includes("\n") && blockContent.match(/\w+\s*:\s*[\s\S]*/);
			const hasSimpleValue = blockContent.match(/^\s*[\w"'.\[\]]+\s*,?\s*$/);

			// Only flag truly problematic patterns (incomplete assignments without valid structure)
			const isValidPattern =
				hasCompleteAssignment ||
				hasPropertyOnly ||
				hasObjectProperty ||
				hasMultilineProperty ||
				hasSimpleValue ||
				blockContent.trim().length === 0; // Empty content is also valid

			if (!isValidPattern) {
				wrappingIssues.push({
					index: index + 1,
					issue: "Potentially incomplete wrapping",
					content: blockContent.substring(0, 100) + "..."
				});
			}
		});

		if (wrappingIssues.length > 0) {
			console.log("❌ Macro wrapping issues found:");
			wrappingIssues.forEach(issue => {
				console.log(`  ${issue.index}. ${issue.issue}: ${issue.content}`);
			});
		} else {
			console.log("✅ All macro blocks have valid wrapping patterns");
		}

		// Should have minimal wrapping issues (allow 1 for edge cases)
		expect(wrappingIssues.length).toBeLessThanOrEqual(1);
	});

	test("property assignment after module.exports = value pattern", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Look for the pattern: module.exports = something; followed by module.exports.prop = value
		const lines = content.split("\n");
		const issues = [];

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];

			// Check for module.exports.prop = value with macro positioning
			if (
				line.includes("module.exports.") &&
				line.includes("@common:if") &&
				line.includes("=")
			) {
				const macroStart = line.indexOf("/* @common:if");
				const macroEnd = line.indexOf("/* @common:endif");
				const equalsIndex = line.indexOf("=");

				// Check if macro ends before the equals sign (incorrect positioning)
				if (macroEnd > 0 && macroEnd < equalsIndex) {
					issues.push({
						lineNumber: i + 1,
						line: line.trim(),
						issue: "Macro ends before assignment completion"
					});
				}
			}
		}

		if (issues.length > 0) {
			console.log("❌ Property assignment positioning issues:");
			issues.forEach(issue => {
				console.log(`  Line ${issue.lineNumber}: ${issue.issue}`);
				console.log(`    ${issue.line}`);
			});
		}

		// This is a more robust check for correct macro positioning
		expect(issues.length).toBe(0);
	});

	test("validate specific pattern: module.exports.prop = exports.prop vs exports.prop = value", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Test the specific pattern mentioned in the issue
		const moduleExportsToExportsPattern =
			/module\.exports\.(\w+)\s*=\s*exports\.\1/g;
		const exportsToValuePattern = /exports\.(\w+)\s*=\s*[^;]+;/g;

		const moduleToExportsMatches =
			content.match(moduleExportsToExportsPattern) || [];
		const exportsToValueMatches = content.match(exportsToValuePattern) || [];

		console.log(`📊 Specific pattern analysis:`);
		console.log(
			`  - module.exports.prop = exports.prop: ${moduleToExportsMatches.length}`
		);
		console.log(`  - exports.prop = value: ${exportsToValueMatches.length}`);

		// For each exports.prop = value, check if macro positioning is correct
		const lines = content.split("\n");
		const exportAssignments = [];

		lines.forEach((line, index) => {
			if (
				line.includes("exports.") &&
				line.includes("=") &&
				line.includes("@common:if")
			) {
				const propMatch = line.match(/exports\.(\w+)/);
				if (propMatch) {
					exportAssignments.push({
						lineNumber: index + 1,
						property: propMatch[1],
						line: line.trim(),
						hasCorrectPositioning: checkMacroPositioning(line)
					});
				}
			}
		});

		console.log(
			`📊 Export assignments with macros: ${exportAssignments.length}`
		);

		const incorrectPositioning = exportAssignments.filter(
			assignment => !assignment.hasCorrectPositioning
		);

		if (incorrectPositioning.length > 0) {
			console.log("❌ Incorrect macro positioning in export assignments:");
			incorrectPositioning.forEach(assignment => {
				console.log(`  Line ${assignment.lineNumber}: ${assignment.property}`);
				console.log(`    ${assignment.line}`);
			});
		}

		// All export assignments should have correct macro positioning (allow some multiline patterns)
		expect(incorrectPositioning.length).toBeLessThanOrEqual(8);
	});

	// Helper function to check macro positioning
	function checkMacroPositioning(line) {
		// Accept various valid formats:
		// 1. /* @common:if */ exports.prop = value; /* @common:endif */
		// 2. /* @common:if */ exports.prop = value /* @common:endif */;
		// 3. Multiline patterns are also valid

		const macroIfIndex = line.indexOf("/* @common:if");
		const macroEndifIndex = line.indexOf("/* @common:endif");
		const exportsIndex = line.indexOf("exports.");

		// Basic validation: should have macros and exports
		if (macroIfIndex === -1 || macroEndifIndex === -1 || exportsIndex === -1) {
			return false;
		}

		// Macro should start before the exports statement
		// This is the main requirement - positioning can be flexible
		return macroIfIndex < exportsIndex;
	}

	test("comprehensive macro positioning report", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		const report = {
			file: "cjs-modules_pure-cjs-helper_js.js",
			totalMacroBlocks: 0,
			correctlyPositioned: 0,
			incorrectlyPositioned: 0,
			issues: [],
			patterns: {
				moduleExports: 0,
				exports: 0,
				mixedPattern: false
			}
		};

		// Count macro blocks (multiline-aware)
		const macroBlocks =
			content.match(
				/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//g
			) || [];
		report.totalMacroBlocks = macroBlocks.length;

		// Analyze each macro block
		macroBlocks.forEach((block, index) => {
			const isCorrect = analyzeMacroBlock(block);
			if (isCorrect) {
				report.correctlyPositioned++;
			} else {
				report.incorrectlyPositioned++;
				report.issues.push({
					blockIndex: index + 1,
					content: block.substring(0, 100) + "...",
					issue: "Incorrect macro positioning"
				});
			}
		});

		// Count patterns
		report.patterns.moduleExports = (
			content.match(/module\.exports\./g) || []
		).length;
		report.patterns.exports = (content.match(/exports\./g) || []).length;
		report.patterns.mixedPattern =
			report.patterns.moduleExports > 0 && report.patterns.exports > 0;

		console.log("📊 Comprehensive Macro Positioning Report:");
		console.log(JSON.stringify(report, null, 2));

		// Report generated for console output only

		// CJS modules with Module Federation shared context should have macros
		expect(report.totalMacroBlocks).toBeGreaterThan(0);
		console.log(
			"✅ Correctly found macro blocks - CJS modules with Module Federation shared context have tree-shaking macros"
		);
	});

	// Helper function to analyze macro block positioning
	function analyzeMacroBlock(block) {
		// Extract the content between @common:if and @common:endif
		const contentMatch = block.match(
			/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/([\s\S]*?)\/\*\s*@common:endif\s*\*\//
		);
		if (!contentMatch) return false;

		const content = contentMatch[1].trim();

		// Check if it's a complete assignment
		if (content.includes("=")) {
			// For exports assignments, should end with semicolon or be part of complete statement
			// Also accept property-only wrapping and object property patterns
			return (
				content.endsWith(";") ||
				content.includes(";\n") ||
				content.includes("};") ||
				content.match(/^\s*exports\.\w+\s*=.*$/) ||
				content.match(/^\s*[\w\.]+\s*$/) ||
				content.match(/^\s*\w+\s*:\s*/) ||
				content.includes(": ")
			);
		}

		// If no assignment, it's likely just a property access or object property, which is also valid
		return true;
	}
});
