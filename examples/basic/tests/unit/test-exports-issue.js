#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

/**
 * Specific test to detect the exact macro positioning issue from the user's example
 * This focuses on the cjs-modules_pure-cjs-helper_js.js file and the specific patterns:
 * - Incorrect: comment @common:if then module.exports.prop then @common:endif then = value
 * - Correct: comment @common:if then module.exports.prop = value; then @common:endif
 */

console.log(
	"🔍 Testing specific macro positioning issues in CommonJS exports...\n"
);

const filePath =
	"/Users/bytedance/RustroverProjects/rspack/examples/basic/dist/cjs-modules_pure-cjs-helper_js.js";

if (!fs.existsSync(filePath)) {
	console.log("❌ Target file not found:", filePath);
	process.exit(1);
}

const content = fs.readFileSync(filePath, "utf8");
console.log("✅ Successfully loaded file:", filePath.split("/").pop());

// Test 1: Detect the specific incorrect patterns
console.log("\n=== Test 1: Detecting incorrect macro positioning patterns ===");

const incorrectPatterns = [];

// Pattern 1: /* @common:if */ exports.prop /* @common:endif */ = value (INCORRECT)
const incorrectExportsRegex =
	/\/\*\s*@common:if[^*]+\*\/\s*exports\.[\w]+\s*\/\*\s*@common:endif[^*]*\*\/\s*=/g;
const incorrectExportsMatches = content.match(incorrectExportsRegex) || [];

// Pattern 2: /* @common:if */ module.exports.prop /* @common:endif */ = value (INCORRECT)
const incorrectModuleExportsRegex =
	/\/\*\s*@common:if[^*]+\*\/\s*module\.exports\.[\w]+\s*\/\*\s*@common:endif[^*]*\*\/\s*=/g;
const incorrectModuleExportsMatches =
	content.match(incorrectModuleExportsRegex) || [];

if (incorrectExportsMatches.length > 0) {
	console.log(
		`❌ Found ${incorrectExportsMatches.length} incorrect exports.prop patterns:`
	);
	incorrectExportsMatches.forEach((match, i) => {
		console.log(`  ${i + 1}. ${match.replace(/\s+/g, " ").trim()}`);
		incorrectPatterns.push({
			type: "exports.prop",
			pattern: match.trim(),
			issue: "Macro ends before assignment"
		});
	});
}

if (incorrectModuleExportsMatches.length > 0) {
	console.log(
		`❌ Found ${incorrectModuleExportsMatches.length} incorrect module.exports.prop patterns:`
	);
	incorrectModuleExportsMatches.forEach((match, i) => {
		console.log(`  ${i + 1}. ${match.replace(/\s+/g, " ").trim()}`);
		incorrectPatterns.push({
			type: "module.exports.prop",
			pattern: match.trim(),
			issue: "Macro ends before assignment"
		});
	});
}

// Test 2: Line-by-line analysis for detailed positioning issues
console.log("\n=== Test 2: Line-by-line macro positioning analysis ===");

const lines = content.split("\n");
const lineIssues = [];

lines.forEach((line, index) => {
	if (
		line.includes("@common:if") &&
		line.includes("@common:endif") &&
		line.includes("=")
	) {
		const lineNumber = index + 1;
		const macroEndIndex = line.indexOf("/* @common:endif");
		const equalsIndex = line.indexOf("=");

		if (macroEndIndex !== -1 && macroEndIndex < equalsIndex) {
			lineIssues.push({
				line: lineNumber,
				content: line.trim(),
				issue: "Macro ends before assignment completion"
			});

			console.log(`❌ Line ${lineNumber}: Macro positioning issue`);
			console.log(`   ${line.trim()}`);
			console.log(
				`   Issue: @common:endif appears at position ${macroEndIndex}, but = appears at position ${equalsIndex}`
			);
		}
	}
});

// Test 3: Validation of the specific pattern mentioned in the issue
console.log(
	"\n=== Test 3: Validating specific pattern: module.exports.prop = exports.prop vs exports.prop = value ==="
);

// Find the specific pattern: module.exports.prop = exports.prop
const moduleToExportsPattern = /module\.exports\.(\w+)\s*=\s*exports\.\1/g;
const moduleToExportsMatches = content.match(moduleToExportsPattern) || [];

// Find pattern: exports.prop = value
const exportsToValuePattern = /exports\.(\w+)\s*=\s*[^;=]+[;}]/g;
const exportsToValueMatches = content.match(exportsToValuePattern) || [];

console.log(`📊 Pattern Analysis:`);
console.log(
	`  - module.exports.prop = exports.prop: ${moduleToExportsMatches.length}`
);
console.log(`  - exports.prop = value: ${exportsToValueMatches.length}`);

if (moduleToExportsMatches.length > 0) {
	console.log(
		`  - Found module.exports patterns: ${moduleToExportsMatches.join(", ")}`
	);
}

// Test 4: Generate comprehensive report
console.log("\n=== Test 4: Comprehensive Issue Report ===");

const report = {
	file: "cjs-modules_pure-cjs-helper_js.js",
	timestamp: new Date().toISOString(),
	issues: {
		incorrectMacroPositioning: incorrectPatterns,
		lineByLineIssues: lineIssues,
		totalIssues: incorrectPatterns.length + lineIssues.length
	},
	patterns: {
		incorrectExportsCount: incorrectExportsMatches.length,
		incorrectModuleExportsCount: incorrectModuleExportsMatches.length,
		moduleToExportsCount: moduleToExportsMatches.length,
		exportsToValueCount: exportsToValueMatches.length
	},
	recommendations: [
		"Change: /* @common:if */ exports.prop /* @common:endif */ = value",
		"To: /* @common:if */ exports.prop = value; /* @common:endif */",
		"Change: /* @common:if */ module.exports.prop /* @common:endif */ = value",
		"To: /* @common:if */ module.exports.prop = value; /* @common:endif */"
	]
};

console.log("📊 Final Report:");
console.log(JSON.stringify(report, null, 2));

// Save report to file
const reportPath = path.join(
	path.dirname(filePath),
	"../exports-issue-report.json"
);
fs.writeFileSync(reportPath, JSON.stringify(report, null, 2));
console.log(`\n💾 Report saved to: ${reportPath}`);

// Test Result
console.log("\n=== Test Result ===");
const totalIssues = report.issues.totalIssues;

if (totalIssues > 0) {
	console.log(`💥 TEST FAILED: Found ${totalIssues} macro positioning issues`);
	console.log("❌ Issues detected:");
	console.log(
		`   - Incorrect exports positioning: ${report.patterns.incorrectExportsCount}`
	);
	console.log(
		`   - Incorrect module.exports positioning: ${report.patterns.incorrectModuleExportsCount}`
	);
	console.log(`   - Total line issues: ${lineIssues.length}`);
	process.exit(1);
} else {
	console.log("🎉 TEST PASSED: No macro positioning issues found");
	process.exit(0);
}
