#!/usr/bin/env node

import fs from "fs";
import https from "https";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const DIST_DIR = path.join(__dirname, "dist");
const LODASH_CHUNK_FILE = path.join(
	DIST_DIR,
	"vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.js"
);
const SHARE_USAGE_FILE = path.join(DIST_DIR, "share-usage.json");
const OPTIMIZED_OUTPUT_FILE = path.join(
	DIST_DIR,
	"vendors-node_modules_pnpm_lodash-es_4_17_21_node_modules_lodash-es_lodash_js.optimized.js"
);

async function main() {
	try {
		console.log("🚀 Starting lodash chunk optimization...");

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

		// Create the request payload
		const payload = {
			code: lodashChunkCode,
			config: treeShakeConfig
		};

		console.log("🌐 Posting to edge optimizer API...");

		// Make the POST request
		const response = await makePostRequest(
			"https://edge-optimizer.federation.workers.dev/",
			payload
		);

		console.log(`✅ API Response received in ${response.execution_time_ms}ms`);

		if (!response.success) {
			throw new Error(`API Error: ${response.error || "Unknown error"}`);
		}

		// Write optimized code to file
		console.log("💾 Writing optimized code to disk...");
		fs.writeFileSync(OPTIMIZED_OUTPUT_FILE, response.optimized_code, "utf8");

		// Calculate file sizes
		const optimizedSize = Buffer.byteLength(response.optimized_code, "utf8");
		const reduction = originalSize - optimizedSize;
		const reductionPercent = ((reduction / originalSize) * 100).toFixed(2);

		console.log("\n📊 Optimization Results:");
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

		console.log(`\n📁 Files created:`);
		console.log(`   ${OPTIMIZED_OUTPUT_FILE}`);

		// Sample the tree-shaking configuration
		console.log("\n🌳 Sample tree-shaking configuration:");
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
		process.exit(1);
	}
}

function makePostRequest(url, data) {
	return new Promise((resolve, reject) => {
		const postData = JSON.stringify(data);

		const options = {
			method: "POST",
			headers: {
				"Content-Type": "application/json",
				"Content-Length": Buffer.byteLength(postData)
			}
		};

		const req = https.request(url, options, res => {
			let responseData = "";

			res.on("data", chunk => {
				responseData += chunk;
			});

			res.on("end", () => {
				try {
					const parsedResponse = JSON.parse(responseData);
					resolve(parsedResponse);
				} catch (error) {
					reject(new Error(`Failed to parse response: ${error.message}`));
				}
			});
		});

		req.on("error", error => {
			reject(new Error(`Request failed: ${error.message}`));
		});

		req.write(postData);
		req.end();
	});
}

// Run the script
main();
