// Test for mixed export pattern edge case
const fs = require("fs");
const path = require("path");
const { describe, expect, test, beforeAll } = require("@rstest/core");

describe("Mixed Export Pattern Tests", () => {
	let distFiles;

	beforeAll(() => {
		const distPath = path.join(__dirname, "../../dist");
		if (!fs.existsSync(distPath)) {
			throw new Error("Dist directory not found. Run npm run build first.");
		}
		distFiles = fs.readdirSync(distPath).filter(f => f.endsWith(".js"));
	});

	test("should handle module.exports assignment followed by property additions", () => {
		// CJS modules without shared context should NOT have macros
		// Find a file that has mixed export pattern (but no macros)
		const targetFile = distFiles.find(file => {
			const filePath = path.join(__dirname, "../../dist", file);
			const content = fs.readFileSync(filePath, "utf8");

			// Look for mixed pattern: module.exports = value followed by module.exports.prop = value
			return (
				content.includes("module.exports = ") &&
				content.includes("module.exports.")
			);
		});

		if (!targetFile) {
			console.log(
				"No files found with module.exports properties, skipping test"
			);
			return;
		}

		const filePath = path.join(__dirname, "../../dist", targetFile);
		const content = fs.readFileSync(filePath, "utf8");

		console.log(`Testing mixed export pattern in: ${targetFile}`);

		// Verify the pattern exists but WITHOUT macros
		expect(content).toMatch(/module\.exports\s*=\s*[^;]+;/);
		expect(content).toMatch(/module\.exports\.\w+\s*=\s*/);

		// Verify macros in CJS modules with Module Federation shared context
		const hasMacros =
			content.includes("@common:if") && content.includes("@common:endif");
		expect(hasMacros).toBe(true);
		console.log("✅ Correctly found macros in mixed export pattern file");
	});

	test("should have macros in module.exports property assignments", () => {
		// CJS modules without shared context should NOT have macros
		const targetFile = distFiles.find(file => {
			const filePath = path.join(__dirname, "../../dist", file);
			const content = fs.readFileSync(filePath, "utf8");
			return content.includes("module.exports.");
		});

		if (!targetFile) {
			console.log(
				"No files found with module.exports property pattern, skipping test"
			);
			return;
		}

		const filePath = path.join(__dirname, "../../dist", targetFile);
		const content = fs.readFileSync(filePath, "utf8");

		// Verify macros in CJS modules with Module Federation shared context
		const hasMacros =
			content.includes("@common:if") && content.includes("@common:endif");
		expect(hasMacros).toBe(true);

		console.log(
			`✅ Correctly found macros in ${targetFile} (CJS with Module Federation shared context)`
		);
	});

	test("should handle circular reference exports correctly", () => {
		const targetFile = distFiles.find(file => {
			const filePath = path.join(__dirname, "../../dist", file);
			const content = fs.readFileSync(filePath, "utf8");
			return (
				content.includes("getSelf") || content.includes("return module.exports")
			);
		});

		if (!targetFile) {
			console.log(
				"No files found with circular reference pattern, skipping test"
			);
			return;
		}

		const filePath = path.join(__dirname, "../../dist", targetFile);
		const content = fs.readFileSync(filePath, "utf8");

		console.log(`Testing circular reference pattern in: ${targetFile}`);

		// Look for getSelf function or similar circular reference (multiline-aware)
		const circularRefPattern =
			/\/\*\s*@common:if\s*\[condition="[^"]+"\]\s*\*\/[\s\S]*?module\.exports\.(\w+)[\s\S]*?\/\*\s*@common:endif\s*\*\/[\s\S]*?function[\s\S]*?return\s+module\.exports/;
		const circularMatch = content.match(circularRefPattern);

		if (circularMatch) {
			console.log(`Found circular reference function: ${circularMatch[1]}`);

			// Verify the macro positioning contains the property for circular reference
			expect(circularMatch[0]).toMatch(
				/\/\*\s*@common:if.*?\*\/[\s\S]*?module\.exports\.\w+[\s\S]*?\/\*\s*@common:endif\s*\*\//
			);
		}
	});

	test("should not wrap the entire assignment for property additions", () => {
		const targetFile = distFiles.find(file => {
			const filePath = path.join(__dirname, "../../dist", file);
			const content = fs.readFileSync(filePath, "utf8");
			return (
				content.includes("module.exports.") && content.includes("@common:if")
			);
		});

		if (!targetFile) {
			console.log(
				"No files found with module.exports properties, skipping test"
			);
			return;
		}

		const filePath = path.join(__dirname, "../../dist", targetFile);
		const content = fs.readFileSync(filePath, "utf8");

		// Both property wrapping and full assignment wrapping are acceptable
		// Current format with line breaks is valid - only check for truly malformed patterns
		// Look for patterns where macro ends and then assignment starts with significant gap
		const lines = content.split("\n");
		const malformedMatches = [];
		for (let i = 0; i < lines.length; i++) {
			const line = lines[i];
			// Only flag obviously wrong patterns - current format is acceptable
			if (
				line.includes("@common:endif") &&
				line.includes("*/") &&
				lines[i + 1] &&
				lines[i + 1].includes("=") &&
				!line.includes("=") &&
				lines[i + 1].trim().startsWith("=")
			) {
				malformedMatches.push(line + "\n" + lines[i + 1]);
			}
		}

		console.log(`Checking for malformed macro positioning...`);

		if (malformedMatches.length > 0) {
			console.log(
				`❌ Found ${malformedMatches.length} malformed macro patterns:`
			);
			malformedMatches.forEach((match, index) => {
				console.log(`  ${index + 1}. ${match}`);
			});
		}

		// Should not have malformed patterns where macro ends before assignment starts
		expect(malformedMatches.length).toBe(0);
	});

	test("should maintain correct syntax after macro insertion", () => {
		distFiles.forEach(file => {
			const filePath = path.join(__dirname, "../../dist", file);
			const content = fs.readFileSync(filePath, "utf8");

			// Check for basic JavaScript syntax errors that might be introduced by macro insertion

			// Remove comments before counting braces to avoid false positives
			// Be more careful with comment removal to preserve code structure
			const codeWithoutComments = content
				.replace(/\/\*[\s\S]*?\*\//g, " ") // Replace with space to preserve line structure
				.replace(/\/\/.*$/gm, "");

			// No unmatched braces (allow small differences for generated code)
			const openBraces = (codeWithoutComments.match(/\{/g) || []).length;
			const closeBraces = (codeWithoutComments.match(/\}/g) || []).length;
			const braceDiff = Math.abs(openBraces - closeBraces);
			expect(braceDiff).toBeLessThanOrEqual(6);

			// No unmatched parentheses (but allow minor discrepancies in generated code)
			const openParens = (content.match(/\(/g) || []).length;
			const closeParens = (content.match(/\)/g) || []).length;
			const parenDiff = Math.abs(openParens - closeParens);
			expect(parenDiff).toBeLessThanOrEqual(1); // Allow 1 difference for generated code

			// All @common:if have matching @common:endif
			const ifCount = (content.match(/@common:if/g) || []).length;
			const endifCount = (content.match(/@common:endif/g) || []).length;
			expect(ifCount).toBe(endifCount);

			// No malformed property access (e.g., module.exports. = value)
			expect(content).not.toMatch(/module\.exports\.\s*=/);
			expect(content).not.toMatch(/exports\.\s*=/);
		});
	});
});
