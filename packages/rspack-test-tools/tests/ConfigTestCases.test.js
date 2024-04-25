const path = require("path");
const { describeByWalk, createConfigCase } = require("../dist");

const NAME = "ConfigTestCases";
const caseDir = path.resolve(__dirname, "./configCases");
const distDir = path.resolve(__dirname, `./js/${NAME}`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createConfigCase(name, src, dist);
});
