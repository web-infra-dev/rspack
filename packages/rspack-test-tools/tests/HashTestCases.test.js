const path = require("path");
const fs = require("fs");
const { createHashCase } = require("..");

const NAME = "HashTestCases";
const caseDir = path.resolve(__dirname, "./hashCases");

const tests = fs
	.readdirSync(caseDir)
	.filter(
		testName =>
			fs.existsSync(path.join(caseDir, testName, "index.js")) ||
			fs.existsSync(path.join(caseDir, testName, "webpack.config.js"))
	);

describe(NAME, () => {
	for (const name of tests) {
		const src = path.join(caseDir, name);
		createHashCase(name, src, path.join(src, "dist"));
	}
});
