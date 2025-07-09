#!/usr/bin/env node

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import { optimize } from "./swc_macro_sys/crates/swc_macro_wasm/pkg/swc_macro_wasm.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DIST_DIR = path.join(__dirname, "dist");
const LODASH_CHUNK_FILE = path.join(
	DIST_DIR,
	"vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js"
);
const SHARE_USAGE_FILE = path.join(DIST_DIR, "share-usage.json");
const LOCAL_OPTIMIZED_FILE = path.join(
	DIST_DIR,
	"vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.local-optimized.js"
);

async function testLocalOptimizer() {
	try {
		console.log("🧪 Testing Local SWC Macro Transformer on Lodash Chunk");
		console.log("=========================================================");

		// Read the lodash chunk file
		console.log("📖 Reading lodash chunk file...");
		const lodashChunkCode = fs.readFileSync(LODASH_CHUNK_FILE, "utf8");
		const originalSize = Buffer.byteLength(lodashChunkCode, "utf8");
		console.log(
			`   Original size: ${originalSize} bytes (${(originalSize / 1024).toFixed(2)} KB)`
		);

		// Read share usage data
		console.log("📋 Reading share usage data...");
		const shareUsageData = JSON.parse(
			fs.readFileSync(SHARE_USAGE_FILE, "utf8")
		);

		// Generate tree-shaking configuration for lodash-es
		console.log("🌳 Generating tree-shaking configuration...");
		const lodashUsage = shareUsageData.consume_shared_modules["lodash-es"];

		if (!lodashUsage) {
			throw new Error("lodash-es usage data not found in share-usage.json");
		}

		const lodashConfig = {};

		// Set used exports to true (keep them)
		lodashUsage.used_exports.forEach(exportName => {
			lodashConfig[exportName] = true;
		});

		// Set unused exports to false (remove them)
		lodashUsage.unused_exports.forEach(exportName => {
			lodashConfig[exportName] = false;
		});

		// Set possibly unused exports to false (conservative approach)
		lodashUsage.possibly_unused_exports.forEach(exportName => {
			lodashConfig[exportName] = false;
		});

		const treeShakeConfig = {
			treeShake: {
				"lodash-es": lodashConfig
			}
		};

		console.log(
			`   Generated ${Object.keys(lodashConfig).length} tree-shaking rules`
		);
		console.log(`   Used exports: ${lodashUsage.used_exports.length}`);
		console.log(`   Unused exports: ${lodashUsage.unused_exports.length}`);
		console.log(
			`   Possibly unused exports: ${lodashUsage.possibly_unused_exports.length}`
		);

		console.log("\n🔧 Testing local transformer...");

		// Test with local optimizer
		const startTime = performance.now();
		const localOptimizedCode = optimize(
			lodashChunkCode,
			JSON.stringify(treeShakeConfig)
		);
		const endTime = performance.now();

		console.log(
			`   ⚡ Local optimization completed in ${(endTime - startTime).toFixed(2)}ms`
		);

		// Check for invalid transforms
		console.log("\n🔍 Checking for transformation issues...");

		const issues = [];

		// Check for <invalid> tokens
		const invalidMatches = localOptimizedCode.match(/<invalid>/g);
		if (invalidMatches) {
			issues.push(`Found ${invalidMatches.length} <invalid> tokens`);
		}

		// Check for syntax errors by looking for common patterns
		const syntaxIssues = [];

		// Check for unclosed macros
		const unclosedIf = localOptimizedCode.match(
			/\/\* @common:if \[.*?\] \*\//g
		);
		const unclosedEndif = localOptimizedCode.match(/\/\* @common:endif \*\//g);
		if (
			unclosedIf &&
			unclosedEndif &&
			unclosedIf.length !== unclosedEndif.length
		) {
			syntaxIssues.push(
				`Mismatched macro pairs: ${unclosedIf.length} @common:if vs ${unclosedEndif.length} @common:endif`
			);
		}

		// Check for incomplete removals
		const incompleteRemovals = localOptimizedCode.match(
			/\/\* @common:if \[condition="treeShake\.lodash-es\.[^"]*"\] \*\/\s*\/\* @common:endif \*\//g
		);
		if (incompleteRemovals) {
			syntaxIssues.push(
				`Found ${incompleteRemovals.length} empty macro blocks (incomplete removals)`
			);
		}

		if (syntaxIssues.length > 0) {
			issues.push(...syntaxIssues);
		}

		// Write optimized code to file
		console.log("💾 Writing locally optimized code to disk...");
		fs.writeFileSync(LOCAL_OPTIMIZED_FILE, localOptimizedCode, "utf8");

		// Calculate file sizes
		const optimizedSize = Buffer.byteLength(localOptimizedCode, "utf8");
		const reduction = originalSize - optimizedSize;
		const reductionPercent = ((reduction / originalSize) * 100).toFixed(2);

		console.log("\n📊 Local Optimization Results:");
		console.log(
			`   Original size:  ${originalSize} bytes (${(originalSize / 1024).toFixed(2)} KB)`
		);
		console.log(
			`   Optimized size: ${optimizedSize} bytes (${(optimizedSize / 1024).toFixed(2)} KB)`
		);
		console.log(
			`   Reduction:      ${reduction} bytes (${(reduction / 1024).toFixed(2)} KB)`
		);
		console.log(`   Percentage:     ${reductionPercent}% smaller`);

		console.log(`\n📁 Local optimized file created:`);
		console.log(`   ${LOCAL_OPTIMIZED_FILE}`);

		// Report issues
		if (issues.length > 0) {
			console.log("\n⚠️  Issues Found:");
			issues.forEach(issue => console.log(`   ❌ ${issue}`));
		} else {
			console.log("\n✅ No transformation issues detected");
		}

		// Show sample of optimized code
		console.log("\n📄 Sample of locally optimized code (first 50 lines):");
		console.log("====================================================");
		const lines = localOptimizedCode.split("\n");
		console.log(lines.slice(0, 50).join("\n"));
		if (lines.length > 50) {
			console.log("... (truncated)");
		}
		console.log("====================================================");

		// Sample the tree-shaking configuration
		console.log("\n🌳 Tree-shaking configuration used:");
		console.log("   treeShake: {");
		console.log('     "lodash-es": {');
		const sampleKeys = Object.keys(lodashConfig).slice(0, 8);
		sampleKeys.forEach(key => {
			console.log(`       "${key}": ${lodashConfig[key]},`);
		});
		if (Object.keys(lodashConfig).length > 8) {
			console.log(
				`       ... and ${Object.keys(lodashConfig).length - 8} more exports`
			);
		}
		console.log("     }");
		console.log("   }");
	} catch (error) {
		console.error("❌ Error:", error.message);
		if (error.stack) {
			console.error("Stack trace:", error.stack);
		}
		process.exit(1);
	}
}

// Run the test
testLocalOptimizer();
