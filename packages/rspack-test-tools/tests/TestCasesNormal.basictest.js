const path = require("path");
const { describeByWalk, createNormalCase } = require("..");

const NAME = "TestCases";
const caseDir = path.resolve(__dirname, "../../rspack/tests/cases");
const distDir = path.resolve(__dirname, `../../rspack/tests/js/normal`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createNormalCase(name, src, dist, path.resolve(__dirname, "../../rspack"));
});
