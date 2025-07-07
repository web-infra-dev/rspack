#!/usr/bin/env node

import fs from "node:fs";

/**
 * Simple test to check if CommonJS chunks have @common:if strings
 */
console.log("🔍 Checking for @common:if in CommonJS chunks...\n");

const filesToCheck = [
	"/Users/bytedance/RustroverProjects/rspack/examples/basic/dist/cjs-modules_legacy-utils_js.js",
	"/Users/bytedance/RustroverProjects/rspack/examples/basic/dist/cjs-modules_data-processor_js.js"
];

let allTestsPassed = true;

for (const filePath of filesToCheck) {
	const fileName = filePath.split("/").pop();

	if (!fs.existsSync(filePath)) {
		console.log(`❌ ${fileName}: File not found`);
		allTestsPassed = false;
		continue;
	}

	const content = fs.readFileSync(filePath, "utf8");
	const hasCommonIf = content.includes("@common:if");

	if (hasCommonIf) {
		console.log(`✅ ${fileName}: Found @common:if string`);

		// Show examples
		const matches = content.match(/@common:if[^@]*/g) || [];
		console.log(`   Found ${matches.length} occurrences`);
		if (matches.length > 0) {
			console.log(`   Example: ${matches[0].substring(0, 80)}...`);
		}
	} else {
		console.log(`❌ ${fileName}: NO @common:if string found`);
		allTestsPassed = false;
	}
}

console.log("\n=== Test Result ===");
if (allTestsPassed) {
	console.log("🎉 ALL TESTS PASSED - @common:if found in all CommonJS chunks");
	process.exit(0);
} else {
	console.log("💥 TESTS FAILED - @common:if NOT found in some CommonJS chunks");
	process.exit(1);
}
