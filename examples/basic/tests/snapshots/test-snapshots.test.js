#!/usr/bin/env node

import fs from "node:fs";
import path from "node:path";
import { expect, test, describe } from "@rstest/core";

/**
 * Rstest snapshot tests for rspack ConsumeShared chunks
 * Snapshots the actual generated chunk content for validation
 */
describe("ConsumeShared Share Chunks Snapshots", () => {
	const distPath = path.join(process.cwd(), "dist");

	test("shared utilities chunk content", () => {
		const filePath = path.join(distPath, "shared_utils_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`Shared utils chunk not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Snapshot the full chunk content
		expect(content).toMatchSnapshot();
	});

	test("shared components chunk content", () => {
		const filePath = path.join(distPath, "shared_components_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`Shared components chunk not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Snapshot the full chunk content
		expect(content).toMatchSnapshot();
	});

	test("shared API chunk content", () => {
		const filePath = path.join(distPath, "shared_api_js.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`Shared API chunk not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Snapshot the full chunk content
		expect(content).toMatchSnapshot();
	});

	test("main chunk webpack runtime", () => {
		const filePath = path.join(distPath, "main.js");

		if (!fs.existsSync(filePath)) {
			throw new Error(`Main chunk not found: ${filePath}`);
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Extract just the webpack runtime portion for more focused snapshot
		const runtimeMatch = content.match(
			/\/\*\*\*\*\*\*\/ \(\(\) => \{ \/\/ webpackBootstrap([\s\S]*?)\/\*\*\*\*\*\*\/ \}\)\(\);/
		);
		const webpackRuntime = runtimeMatch
			? runtimeMatch[1]
			: content.substring(0, 5000); // First 5KB if no match

		expect(webpackRuntime).toMatchSnapshot();
	});

	test("all dist chunk file structure", () => {
		const distFiles = fs
			.readdirSync(distPath)
			.filter(file => file.endsWith(".js"));

		const chunkSummary = {};
		for (const file of distFiles) {
			const filePath = path.join(distPath, file);
			const content = fs.readFileSync(filePath, "utf8");

			chunkSummary[file] = {
				size: content.length,
				hasMacroComments: content.includes("@common:if"),
				hasPureAnnotations: content.includes("/* #__PURE__ */"),
				hasWebpackRequire: content.includes("__webpack_require__"),
				macroCount: (content.match(/@common:if/g) || []).length,
				// Include first 200 chars for structure validation
				preview: content.substring(0, 200).replace(/\s+/g, " ").trim()
			};
		}

		expect(chunkSummary).toMatchSnapshot();
	});

	test("macro annotations extracted", () => {
		const chunkFiles = [
			"shared_utils_js.js",
			"shared_components_js.js",
			"shared_api_js.js"
		];

		const extractedMacros = {};

		for (const file of chunkFiles) {
			const filePath = path.join(distPath, file);
			if (fs.existsSync(filePath)) {
				const content = fs.readFileSync(filePath, "utf8");

				// Extract all macro comment blocks
				const macroMatches =
					content.match(
						/\/\* @common:if \[condition="[^"]+"\] \*\/[\s\S]*?\/\* @common:endif \*\//g
					) || [];

				extractedMacros[file] = macroMatches.map(match => {
					// Clean up whitespace for more stable snapshots
					return match.replace(/\s+/g, " ").trim();
				});
			}
		}

		expect(extractedMacros).toMatchSnapshot();
	});

	test("CommonJS macro positioning snapshot", () => {
		const commonJSFiles = [
			"cjs-modules_pure-cjs-helper_js.js",
			"cjs-modules_legacy-utils_js.js",
			"cjs-modules_data-processor_js.js"
		];

		const macroPositioningSummary = {};

		for (const file of commonJSFiles) {
			const filePath = path.join(distPath, file);
			if (fs.existsSync(filePath)) {
				const content = fs.readFileSync(filePath, "utf8");

				// Extract macro positioning patterns
				const lines = content.split("\n");
				const macroLines = [];
				const positioningIssues = [];

				lines.forEach((line, index) => {
					if (line.includes("@common:if")) {
						macroLines.push({
							lineNumber: index + 1,
							content: line.trim(),
							hasEndif: line.includes("@common:endif"),
							hasEquals: line.includes("="),
							potentialIssue:
								line.includes("@common:endif") &&
								line.includes("=") &&
								line.indexOf("@common:endif") < line.indexOf("=")
						});

						// Check for positioning issues
						if (
							line.includes("@common:endif") &&
							line.includes("=") &&
							line.indexOf("@common:endif") < line.indexOf("=")
						) {
							positioningIssues.push({
								line: index + 1,
								issue: "macro_ends_before_assignment",
								pattern: line.trim()
							});
						}
					}
				});

				macroPositioningSummary[file] = {
					totalMacroLines: macroLines.length,
					positioningIssues: positioningIssues,
					macroPatterns: macroLines
				};
			}
		}

		// Snapshot the positioning summary
		expect(macroPositioningSummary).toMatchSnapshot();
	});

	test("export pattern analysis snapshot", () => {
		const filePath = path.join(distPath, "cjs-modules_pure-cjs-helper_js.js");

		if (!fs.existsSync(filePath)) {
			return;
		}

		const content = fs.readFileSync(filePath, "utf8");

		// Extract all export patterns with their context
		const exportPatterns = {
			moduleExports: [],
			exports: [],
			mixedPatterns: false
		};

		const lines = content.split("\n");
		lines.forEach((line, index) => {
			if (line.includes("module.exports.")) {
				exportPatterns.moduleExports.push({
					line: index + 1,
					content: line.trim(),
					hasMacro: line.includes("@common:if")
				});
			}

			if (line.includes("exports.") && !line.includes("module.exports.")) {
				exportPatterns.exports.push({
					line: index + 1,
					content: line.trim(),
					hasMacro: line.includes("@common:if")
				});
			}
		});

		exportPatterns.mixedPatterns =
			exportPatterns.moduleExports.length > 0 &&
			exportPatterns.exports.length > 0;

		// Snapshot the export pattern analysis
		expect(exportPatterns).toMatchSnapshot();
	});
});
