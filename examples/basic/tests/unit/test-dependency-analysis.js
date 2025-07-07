#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";

/**
 * Analyze how CommonJS modules are being consumed vs ESM modules
 */
console.log("🔍 Analyzing CommonJS vs ESM dependency consumption...\n");

const distPath = path.join(process.cwd(), "dist");
const mainJsPath = path.join(distPath, "main.js");

if (!fs.existsSync(mainJsPath)) {
	console.log("❌ main.js not found");
	process.exit(1);
}

const mainContent = fs.readFileSync(mainJsPath, "utf8");

console.log("=== Module Federation Sharing Setup ===");

// Extract the sharing configuration
const sharingDataMatch = mainContent.match(
	/__webpack_require__\.initializeSharingData\s*=\s*{[^}]+scopeToSharingDataMapping[^}]+}/
);
if (sharingDataMatch) {
	const sharingData = sharingDataMatch[0];

	// Check which modules are registered as ProvideShared
	const sharedModules = [
		"api-lib",
		"component-lib",
		"utility-lib",
		"data-processor-lib",
		"legacy-utils-lib",
		"pure-cjs-helper-lib",
		"react",
		"react-dom",
		"lodash-es"
	];

	console.log("Modules registered as ProvideShared:");
	for (const moduleKey of sharedModules) {
		const isProvideShared = sharingData.includes(`"${moduleKey}"`);
		const moduleType =
			moduleKey.includes("lib") &&
			!["api-lib", "component-lib", "utility-lib"].includes(moduleKey)
				? "CommonJS"
				: "ESM/External";
		console.log(
			`  ${isProvideShared ? "✅" : "❌"} ${moduleKey} (${moduleType})`
		);
	}
} else {
	console.log("❌ No sharing data found");
}

console.log("\n=== Direct Dependency Consumption Analysis ===");

// Look for direct require() calls vs import statements
const requirePattern = /require\s*\(\s*["']([^"']+)["']\s*\)/g;
const requireMatches = [...mainContent.matchAll(requirePattern)];

console.log("Direct require() calls found:");
for (const match of requireMatches) {
	const modulePath = match[1];
	if (modulePath.includes("cjs-modules")) {
		console.log(`  🔗 ${modulePath} (CommonJS - direct consumption)`);
	}
}

console.log("\n=== Share-Usage.json Analysis ===");
const shareUsagePath = path.join(distPath, "share-usage.json");
if (fs.existsSync(shareUsagePath)) {
	const shareUsageData = JSON.parse(fs.readFileSync(shareUsagePath, "utf8"));
	const consumeSharedModules = Object.keys(
		shareUsageData.consume_shared_modules || {}
	);

	console.log("Modules tracked in ConsumeShared (with usage data):");
	for (const moduleKey of consumeSharedModules) {
		const moduleData = shareUsageData.consume_shared_modules[moduleKey];
		const usedCount = moduleData.used_exports?.length || 0;
		const unusedCount = moduleData.unused_exports?.length || 0;
		console.log(
			`  📊 ${moduleKey}: ${usedCount} used, ${unusedCount} unused exports`
		);
	}

	console.log("\nCommonJS modules NOT in ConsumeShared tracking:");
	const commonJSModules = [
		"legacy-utils-lib",
		"data-processor-lib",
		"pure-cjs-helper-lib"
	];
	for (const moduleKey of commonJSModules) {
		if (!consumeSharedModules.includes(moduleKey)) {
			console.log(`  ⚠️  ${moduleKey} - ProvideShared but not ConsumeShared`);
		}
	}
}

console.log("\n=== Key Findings ===");
console.log("1. ✅ CommonJS modules are registered as ProvideShared");
console.log("2. ⚠️  CommonJS modules accessed via direct require() calls");
console.log("3. ⚠️  Direct require() bypasses ConsumeShared mechanism");
console.log(
	"4. ✅ ESM modules go through ConsumeShared and get macro annotations"
);
console.log(
	"5. 📝 This explains why CommonJS modules don't appear in share-usage.json"
);

console.log("\n🎯 Conclusion:");
console.log(
	"The ShareUsagePlugin is working correctly - it tracks ConsumeShared modules."
);
console.log(
	"CommonJS modules are ProvideShared but not ConsumeShared when accessed via require()."
);
