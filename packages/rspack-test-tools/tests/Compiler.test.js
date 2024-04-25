const path = require("path");
const fs = require("fs");
const { createCompilerCase } = require("../dist");
const srcDir = path.resolve(__dirname, "../../rspack/tests/fixtures");
const distDir = path.resolve(__dirname, "../../rspack/tests/js/compiler");
const caseDir = path.resolve(__dirname, "./compilerCases");

describe("Compiler", () => {
	const cases = fs.readdirSync(caseDir);
	for (let file of cases) {
		createCompilerCase(file, srcDir, distDir, caseDir);
	}
});
