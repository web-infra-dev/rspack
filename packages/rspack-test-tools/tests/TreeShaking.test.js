const path = require("path");
const fs = require("fs");
const { createTreeShakingCase } = require("..");

const NAME = "TreeShakingCases";
const caseDir = path.resolve(__dirname, "./treeShakingCases");
const distDir = path.resolve(__dirname, `./js/${NAME}`);

const tests = fs
	.readdirSync(caseDir)
	.filter(
		testName =>
			!testName.startsWith(".") &&
			(fs.existsSync(path.join(caseDir, testName, "test.config.json")) ||
				fs.existsSync(path.join(caseDir, testName, "test.config.js")))
	);

describe(NAME, () => {
	for (const name of tests) {
		const src = path.join(caseDir, name);
		createTreeShakingCase(name, src, path.join(distDir, name));
	}
});
