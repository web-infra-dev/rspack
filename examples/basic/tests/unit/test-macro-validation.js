#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

/**
 * Direct macro annotation validation test
 */
console.log("🔍 Testing macro annotations in dist files...\n");

const distPath = path.join(process.cwd(), "dist");

// Expected ESM ConsumeShared files that MUST have macro annotations
const expectedESMFiles = [
	"shared_utils_js.js",
	"shared_components_js.js",
	"shared_api_js.js"
];

// CommonJS files that should NOT have macro annotations
const expectedCommonJSFiles = [
	"cjs-modules_legacy-utils_js.js",
	"cjs-modules_data-processor_js.js",
	"cjs-modules_pure-cjs-helper_js.js"
];

let totalMacrosFound = 0;
let testsPassed = 0;
let testsFailed = 0;

console.log("=== ESM ConsumeShared Files (MUST have macros) ===");
for (const file of expectedESMFiles) {
	const filePath = path.join(distPath, file);
	if (!fs.existsSync(filePath)) {
		console.log(`❌ ${file}: File not found`);
		testsFailed++;
		continue;
	}

	const content = fs.readFileSync(filePath, "utf8");

	// Count macro annotations in this file
	const macroMatches =
		content.match(/@common:if\s*\[condition="treeShake\.[^"]+"\]/g) || [];
	const macroCount = macroMatches.length;

	if (macroCount > 0) {
		console.log(`✅ ${file}: Found ${macroCount} macro annotations`);
		console.log(`   Examples: ${macroMatches.slice(0, 2).join(", ")}`);
		testsPassed++;
	} else {
		console.log(`❌ ${file}: Expected macros but found none`);
		testsFailed++;
	}

	totalMacrosFound += macroCount;
}

console.log("\n=== CommonJS Files (should NOT have macros) ===");
for (const file of expectedCommonJSFiles) {
	const filePath = path.join(distPath, file);
	if (!fs.existsSync(filePath)) {
		console.log(`⚠️  ${file}: File not found`);
		continue;
	}

	const content = fs.readFileSync(filePath, "utf8");

	// Check for any macro annotations (should be none)
	const hasMacros =
		content.includes("@common:if") || content.includes("treeShake");
	const hasCommonJS =
		content.includes("module.exports") && content.includes("exports.");

	if (!hasMacros && hasCommonJS) {
		console.log(
			`✅ ${file}: No macros found (correct), CommonJS structure present`
		);
		testsPassed++;
	} else if (hasMacros) {
		console.log(`❌ ${file}: Unexpected macros found`);
		testsFailed++;
	} else {
		console.log(`❌ ${file}: Not a proper CommonJS file`);
		testsFailed++;
	}
}

console.log("\n=== Summary ===");
console.log(`📊 Total macro annotations found: ${totalMacrosFound}`);
console.log(`✅ Tests passed: ${testsPassed}`);
console.log(`❌ Tests failed: ${testsFailed}`);

if (testsFailed === 0 && totalMacrosFound > 0) {
	console.log("\n🎉 ALL TESTS PASSED!");
	console.log("✅ ESM ConsumeShared files have macro annotations");
	console.log("✅ CommonJS files don't have macro annotations (expected)");
	process.exit(0);
} else {
	console.log("\n💥 TESTS FAILED!");
	if (totalMacrosFound === 0) {
		console.log("❌ No macro annotations found in any ESM files");
	}
	if (testsFailed > 0) {
		console.log(`❌ ${testsFailed} validation tests failed`);
	}
	process.exit(1);
}
