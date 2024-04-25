const path = require("path");
const fs = require("fs");
const { createErrorCase } = require("../dist");
const srcDir = __dirname;
const distDir = path.join(__dirname, "./js/error");
const caseDir = path.resolve(__dirname, "./errorCases");

describe("Error", () => {
	const cases = fs.readdirSync(caseDir);
	for (let file of cases) {
		createErrorCase(file, srcDir, distDir, caseDir);
	}
});
