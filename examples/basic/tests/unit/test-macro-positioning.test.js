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
		// This should be detected as INCORRECT positioning
		const incorrectModuleExportsPattern = /\/\*\s*@common:if\s*[^*]*\*\/\s*module\.exports\.[\w]+\s*\/\*\s*@common:endif\s*\*\/\s*=/g;
		const incorrectMatches = content.match(incorrectModuleExportsPattern) || [];

		// Test for the correct pattern: entire assignment wrapped
		const correctModuleExportsPattern = /\/\*\s*@common:if\s*[^*]*\*\/\s*module\.exports\.[\w]+\s*=\s*[^;]+;\s*\/\*\s*@common:endif\s*\*\//g;
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

		// A more robust check is now used in the first test
		// This is now a smoke test to ensure no obviously incorrect patterns are present
		expect(incorrectMatches.length).toBe(0);
		
		// Check if we have macro patterns (individual assignments use different format)
		const hasMacroPatterns = content.includes('/* @common:if');
		if (hasMacroPatterns) {
			console.log(`✅ Found macro patterns in file`);
		} else {
			console.log(`ℹ️  No macro patterns found - file may not have ConsumeShared exports`);
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
		console.log(`  - module.exports.prop assignments: ${moduleExportsMatches.length}`);
		console.log(`  - exports.prop assignments: ${exportsMatches.length}`);

		if (moduleExportsMatches.length > 0) {
			console.log(`  - module.exports patterns found: ${moduleExportsMatches.join(", ")}`);
		}

		// Both patterns should be consistent within the same file
		// If we have both, it suggests mixed patterns which can cause issues
		if (moduleExportsMatches.length > 0 && exportsMatches.length > 0) {
			console.log("⚠️  Mixed export patterns detected - this can cause macro positioning issues");
		}

		// The file should primarily use one pattern or the other consistently
		expect(moduleExportsMatches.length + exportsMatches.length).toBeGreaterThan(0);
	});

	test("validate macro wrapping completeness", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");
		
		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Find all @common:if blocks
		const macroBlocks = content.match(/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//g) || [];
		
		console.log(`📊 Found ${macroBlocks.length} complete macro blocks`);

		const wrappingIssues = [];

		macroBlocks.forEach((block, index) => {
			const blockContent = block.replace(/\/\*\s*@common:(if|endif)[^*]*\*\//g, "").trim();
			
			// Check if the block properly wraps the entire assignment
			const hasCompleteAssignment = blockContent.includes("=") && blockContent.includes(";");
			const hasPartialAssignment = blockContent.includes("=") && !blockContent.includes(";");
			
			if (hasPartialAssignment) {
				wrappingIssues.push({
					index: index + 1,
					issue: "Incomplete assignment wrapping",
					content: blockContent.substring(0, 100) + "..."
				});
			}
		});

		if (wrappingIssues.length > 0) {
			console.log("❌ Macro wrapping issues found:");
			wrappingIssues.forEach(issue => {
				console.log(`  ${issue.index}. ${issue.issue}: ${issue.content}`);
			});
		}

		// Should have no wrapping issues
		expect(wrappingIssues.length).toBe(0);
	});

	test("property assignment after module.exports = value pattern", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");
		
		if (!fs.existsSync(filePath)) {
			throw new Error(`File not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Look for the pattern: module.exports = something; followed by module.exports.prop = value
		const lines = content.split('\n');
		const issues = [];

		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			
			// Check for module.exports.prop = value with macro positioning
			if (line.includes('module.exports.') && line.includes('@common:if') && line.includes('=')) {
				const macroStart = line.indexOf('/* @common:if');
				const macroEnd = line.indexOf('/* @common:endif');
				const equalsIndex = line.indexOf('=');
				
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
		const moduleExportsToExportsPattern = /module\.exports\.(\w+)\s*=\s*exports\.\1/g;
		const exportsToValuePattern = /exports\.(\w+)\s*=\s*[^;]+;/g;

		const moduleToExportsMatches = content.match(moduleExportsToExportsPattern) || [];
		const exportsToValueMatches = content.match(exportsToValuePattern) || [];

		console.log(`📊 Specific pattern analysis:`);
		console.log(`  - module.exports.prop = exports.prop: ${moduleToExportsMatches.length}`);
		console.log(`  - exports.prop = value: ${exportsToValueMatches.length}`);

		// For each exports.prop = value, check if macro positioning is correct
		const lines = content.split('\n');
		const exportAssignments = [];

		lines.forEach((line, index) => {
			if (line.includes('exports.') && line.includes('=') && line.includes('@common:if')) {
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

		console.log(`📊 Export assignments with macros: ${exportAssignments.length}`);
		
		const incorrectPositioning = exportAssignments.filter(assignment => !assignment.hasCorrectPositioning);
		
		if (incorrectPositioning.length > 0) {
			console.log("❌ Incorrect macro positioning in export assignments:");
			incorrectPositioning.forEach(assignment => {
				console.log(`  Line ${assignment.lineNumber}: ${assignment.property}`);
				console.log(`    ${assignment.line}`);
			});
		}

		// All export assignments should have correct macro positioning
		expect(incorrectPositioning.length).toBe(0);
	});

	// Helper function to check macro positioning
	function checkMacroPositioning(line) {
		// Correct: /* @common:if */ exports.prop = value; /* @common:endif */
		// Incorrect: /* @common:if */ exports.prop /* @common:endif */ = value;
		
		const macroIfIndex = line.indexOf('/* @common:if');
		const macroEndifIndex = line.indexOf('/* @common:endif');
		const equalsIndex = line.indexOf('=');
		const semicolonIndex = line.lastIndexOf(';');
		
		if (macroIfIndex === -1 || macroEndifIndex === -1 || equalsIndex === -1) {
			return false;
		}
		
		// Check if the macro wraps the entire assignment
		// The @common:if should come before the assignment
		// The @common:endif should come after the assignment (including semicolon)
		return macroIfIndex < equalsIndex && 
		       (semicolonIndex === -1 || macroEndifIndex > semicolonIndex);
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

		// Count macro blocks
		const macroBlocks = content.match(/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/[\s\S]*?\/\*\s*@common:endif\s*\*\//g) || [];
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
		report.patterns.moduleExports = (content.match(/module\.exports\./g) || []).length;
		report.patterns.exports = (content.match(/exports\./g) || []).length;
		report.patterns.mixedPattern = report.patterns.moduleExports > 0 && report.patterns.exports > 0;

		console.log("📊 Comprehensive Macro Positioning Report:");
		console.log(JSON.stringify(report, null, 2));

		// Write report to file for debugging
		const reportPath = path.join(process.cwd(), "macro-positioning-report.json");
		fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));

		// Assertions
		expect(report.totalMacroBlocks).toBeGreaterThan(0);
		expect(report.incorrectlyPositioned).toBe(0);
		expect(report.correctlyPositioned).toBe(report.totalMacroBlocks);
	});

	// Helper function to analyze macro block positioning
	function analyzeMacroBlock(block) {
		// Extract the content between @common:if and @common:endif
		const contentMatch = block.match(/\/\*\s*@common:if\s*\[[^\]]+\]\s*\*\/([\s\S]*?)\/\*\s*@common:endif\s*\*\//);
		if (!contentMatch) return false;

		const content = contentMatch[1].trim();
		
		// Check if it's a complete assignment
		if (content.includes('=')) {
			// Should end with semicolon for complete assignment
			return content.endsWith(';') || content.includes(';\n') || content.includes('};');
		}

		// If no assignment, it's likely just a property access, which is also valid
		return true;
	}
});