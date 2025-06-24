// Test to ensure there are no double commas: /* @common:endif */,
const fs = require("node:fs");
const path = require("node:path");

export const testNoDoubleCommas = () => {
	const targetFile = path.join(__dirname, "../../dist/cjs-modules_module-exports-pattern_js.js");
	const content = fs.readFileSync(targetFile, "utf8");
	
	console.log("🔍 Checking for double comma issue...");
	
	// Look for the problematic pattern: /* @common:endif */,
	const problematicPattern = /\/\*\s*@common:endif\s*\*\/\s*,/g;
	const problematicMatches = content.match(problematicPattern);
	
	console.log(`❌ Found ${problematicMatches ? problematicMatches.length : 0} problematic patterns with comma after endif`);
	
	if (problematicMatches) {
		console.log("❌ Examples of problematic patterns:");
		problematicMatches.slice(0, 3).forEach((match, i) => {
			console.log(`  ${i + 1}. "${match}"`);
		});
		throw new Error("Found commas outside macro blocks - this creates double comma issue");
	}
	
	// Verify the correct pattern exists: property, /* @common:endif */
	const correctPattern = /\w+,\s*\/\*\s*@common:endif\s*\*\/(?!\s*,)/g;
	const correctMatches = content.match(correctPattern);
	
	console.log(`✅ Found ${correctMatches ? correctMatches.length : 0} correct patterns with comma inside macro only`);
	
	if (correctMatches) {
		console.log("✅ Example correct pattern:", correctMatches[0]);
	}
	
	if (!correctMatches || correctMatches.length === 0) {
		throw new Error("No correct comma patterns found");
	}
	
	console.log("🎉 No double comma issues found!");
};