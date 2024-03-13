const path = require("path");
const fs = require("fs");
const { createTreeShakingCase } = require("..");

const NAME = "TreeShakingCases";
const caseDir = path.resolve(
	__dirname,
	"../../../crates/rspack/tests/tree-shaking"
);
const distDir = path.resolve(__dirname, `../../rspack/tests/js/${NAME}`);

const tests = fs
	.readdirSync(caseDir)
	.filter(
		testName =>
			!testName.startsWith(".") &&
			fs.existsSync(path.join(caseDir, testName, "test.config.json"))
	);

describe(NAME, () => {
	jest.setTimeout(30000);
	for (const name of tests) {
		const src = path.join(caseDir, name);
		createTreeShakingCase(name, src, path.join(distDir, name));
	}
});
