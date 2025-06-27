// Unit test for correct comma positioning in CommonJS object literals
const fs = require("node:fs");
const path = require("node:path");
const { describe, expect, test } = require("@rstest/core");

describe("Comma positioning in CommonJS object literals", () => {
	const distPath = path.join(__dirname, "../../dist");

	test("should include commas inside macro blocks", () => {
		const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
		let foundObjectExports = false;
		let allCorrect = true;
		const issues = [];

		for (const file of distFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Look for module.exports = { patterns
			const moduleExportsPattern = /module\.exports\s*=\s*\{[^}]*\}/gs;
			const matches = content.match(moduleExportsPattern);

			if (matches) {
				foundObjectExports = true;
				
				for (const match of matches) {
					// Check for incorrect patterns: comma outside macro block
					const incorrectPattern = /\/\*\s*@common:if[^*]*\*\/[^,]*\/\*\s*@common:endif\s*\*\/\s*,/g;
					const incorrectMatches = match.match(incorrectPattern);

					if (incorrectMatches) {
						allCorrect = false;
						issues.push(`${file}: Found ${incorrectMatches.length} commas outside macro blocks`);
					}

					// Check for correct patterns: comma inside macro block
					const correctPattern = /\/\*\s*@common:if[^*]*\*\/[^*]*,\s*\/\*\s*@common:endif\s*\*\//g;
					const correctMatches = match.match(correctPattern);

					if (correctMatches) {
						console.log(`✅ ${file}: Found ${correctMatches.length} correctly positioned commas`);
					}
				}
			}
		}

		expect(foundObjectExports).toBe(true);
		expect(allCorrect).toBe(true);
		
		if (issues.length > 0) {
			console.log("Issues found:");
			for (const issue of issues) {
				console.log(`  - ${issue}`);
			}
		}
	});

	test("should not have orphaned commas when macros are removed", () => {
		const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
		
		for (const file of distFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Simulate macro removal by removing @common:if blocks
			const withoutMacros = content.replace(
				/\/\*\s*@common:if[^*]*\*\/.*?\/\*\s*@common:endif\s*\*\//gs,
				""
			);

			// Check for syntax issues like double commas
			const doubleCommas = /,,/g;
			const doubleCommaMatches = withoutMacros.match(doubleCommas);

			// Check for problematic trailing commas (but allow valid ones in object literals)
			// Only check for trailing commas followed by a closing brace on the same line
			const problematicTrailingCommas = /,\s*}\s*(?![,\s]*$)/g;
			const problematicMatches = withoutMacros.match(problematicTrailingCommas);

			if (doubleCommaMatches) {
				console.log(`❌ ${file}: Found ${doubleCommaMatches.length} double commas after macro removal`);
			}

			if (problematicMatches) {
				console.log(`❌ ${file}: Found ${problematicMatches.length} problematic trailing commas after macro removal`);
			}

			expect(doubleCommaMatches).toBeNull();
			expect(problematicMatches).toBeNull();
		}
	});

	test("should have consistent comma formatting in object literals", () => {
		const distFiles = fs.readdirSync(distPath).filter(file => file.endsWith(".js"));
		
		for (const file of distFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			// Look for module.exports = { patterns
			const moduleExportsPattern = /module\.exports\s*=\s*\{([^}]*)\}/gs;
			const matches = [...content.matchAll(moduleExportsPattern)];

			for (const match of matches) {
				const objectContent = match[1];
				
				// Find all macro blocks in the object
				const macroBlocks = [...objectContent.matchAll(
					/\/\*\s*@common:if[^*]*\*\/(.*?)\/\*\s*@common:endif\s*\*\//gs
				)];

				for (const [fullMatch, innerContent] of macroBlocks) {
					// Each macro block should contain at most one comma (some may have none for last properties)
					const commaCount = (innerContent.match(/,/g) || []).length;
					expect(commaCount).toBeLessThanOrEqual(1);
					
					// If it has a comma, it should be at the end of the property name
					if (commaCount === 1) {
						expect(innerContent.trim()).toMatch(/\w+,\s*$/);
					}
				}
			}
		}
	});
});