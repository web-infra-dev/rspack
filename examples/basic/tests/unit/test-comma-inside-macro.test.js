// Simple test to verify comma is inside macro blocks
const fs = require("node:fs");
const path = require("node:path");

export const testCommaPositioning = () => {
		const targetFile = path.join(__dirname, "dist/cjs-modules_module-exports-pattern_js.js");
		const content = fs.readFileSync(targetFile, "utf8");
		
		console.log("🔍 Checking comma positioning in generated file...");
		
		// Look for the specific correct pattern: /* @common:if [...] */ property, /* @common:endif */
		const correctPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*\w+,\s*\/\*\s*@common:endif\s*\*\//g;
		const correctMatches = content.match(correctPattern);
		
		// Look for the incorrect pattern: /* @common:if [...] */ property /* @common:endif */,
		const incorrectPattern = /\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/\s*\w+\s*\/\*\s*@common:endif\s*\*\/\s*,/g;
		const incorrectMatches = content.match(incorrectPattern);
		
		console.log(`✅ Found ${correctMatches ? correctMatches.length : 0} correctly positioned commas`);
		console.log(`❌ Found ${incorrectMatches ? incorrectMatches.length : 0} incorrectly positioned commas`);
		
		if (correctMatches) {
			console.log("✅ Example correct format:", correctMatches[0]);
		}
		
		if (incorrectMatches) {
			console.log("❌ Example incorrect format:", incorrectMatches[0]);
			throw new Error("Found commas outside macro blocks");
		}
		
		if (!correctMatches || correctMatches.length === 0) {
			throw new Error("No correctly positioned commas found");
		}
};