const path = require("path");
const fs = require("fs");
const { createStatsCase } = require("..");

const NAME = "StatsTestCases";
const caseDir = path.resolve(__dirname, "./statsCases");
const distDir = path.resolve(__dirname, `./js/stats`);

const tests = fs
	.readdirSync(caseDir)
	.filter(
		testName =>
			fs.existsSync(path.join(caseDir, testName, "index.js")) ||
			fs.existsSync(path.join(caseDir, testName, "webpack.config.js"))
	);

describe(NAME, () => {
	jest.setTimeout(30000);
	for (const name of tests) {
		const src = path.join(caseDir, name);
		const dist = path.join(distDir, name);
		createStatsCase(name, src, dist);
	}
});
