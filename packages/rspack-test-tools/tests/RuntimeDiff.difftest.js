const { globSync } = require("glob");
const path = require("path");
const { describeByWalk, createDiffCase } = require("..");

const caseDir = path.resolve(__dirname, "runtimeDiffCases");
const tempDir = path.resolve(__dirname, "js");

describe(`RuntimeDiffCases`, () => {
	for (let name of globSync("**/test.config.js", {
		cwd: caseDir
	})) {
		name = path.dirname(name);
		const src = path.join(caseDir, name);
		const dist = path.join(tempDir, "runtime-diff", name);
		createDiffCase(name, src, dist);
	}
});
