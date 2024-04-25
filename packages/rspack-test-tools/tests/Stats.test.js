const path = require("path");
const fs = require("fs");
const { createStatsAPICase } = require("../dist");
const srcDir = __dirname;
const caseDir = path.resolve(__dirname, "./statsAPICases");

describe("Stats", () => {
	const cases = fs.readdirSync(caseDir);
	for (let file of cases) {
		createStatsAPICase(file, srcDir, "none", caseDir);
	}
});
