const path = require("path");
const { describeByWalk, createNormalCase } = require("..");

const NAME = "TestCases";
const caseDir = path.resolve(__dirname, "./cases");
const distDir = path.resolve(__dirname, `./js/normal`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createNormalCase(name, src, dist);
});
