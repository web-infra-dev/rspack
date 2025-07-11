#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Test for the correct transformData pattern in module-exports-pattern chunk
const targetFile = path.join(
	__dirname,
	"dist/cjs-modules_module-exports-pattern_js.js"
);

if (!fs.existsSync(targetFile)) {
	console.error(`❌ Target file not found: ${targetFile}`);
	process.exit(1);
}

const content = fs.readFileSync(targetFile, "utf8");

// Look for the correct pattern: transformData, inside the macro
const correctPattern =
	'/* @common:if [condition="treeShake.cjs-module-exports.transformData"] */ transformData, /* @common:endif */';
const hasCorrectPattern = content.includes(correctPattern);

// Look for incorrect pattern: comma outside
const incorrectPattern = "/* @common:endif */,";
const linesWithIncorrectPattern = content
	.split("\n")
	.filter(
		line => line.includes("transformData") && line.includes(incorrectPattern)
	);

console.log("🔍 Testing transformData comma positioning...");

if (hasCorrectPattern) {
	console.log("✅ Found correct transformData pattern with comma inside macro");
} else {
	console.log("❌ transformData pattern not found or incorrect");
}

if (linesWithIncorrectPattern.length > 0) {
	console.log("❌ Found incorrect patterns with comma outside macro:");
	linesWithIncorrectPattern.forEach((line, i) => {
		console.log(`  ${i + 1}. ${line.trim()}`);
	});
	console.log("❌ Test FAILED - transformData has comma outside macro");
	process.exit(1);
} else {
	console.log("✅ No incorrect comma patterns found for transformData");
}

// Additional validation - check that transformData appears with comma inside
const transformDataLine = content
	.split("\n")
	.find(line => line.includes("transformData"));
if (transformDataLine) {
	console.log(`📝 transformData line: ${transformDataLine.trim()}`);

	if (
		transformDataLine.includes("transformData,") &&
		transformDataLine.includes("/* @common:if") &&
		transformDataLine.includes("/* @common:endif */")
	) {
		console.log("✅ transformData has comma inside macro boundary");
	} else {
		console.log("❌ transformData comma positioning is incorrect");
		process.exit(1);
	}
}

console.log("🎉 All transformData comma positioning tests passed!");
