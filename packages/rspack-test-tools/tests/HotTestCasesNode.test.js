const path = require("path");
const { describeByWalk, createHotCase } = require("..");

const NAME = "HotTestCases";
const caseDir = path.resolve(__dirname, "../../rspack/tests/hotCases");
const distDir = path.resolve(__dirname, `../../rspack/tests/js/${NAME}`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createHotCase(name, src, dist, "async-node");
});
