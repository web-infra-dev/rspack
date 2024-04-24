const path = require("path");
const { describeByWalk, createHotCase } = require("..");

const NAME = "HotTestCases";
const caseDir = path.resolve(__dirname, "./hotCases");
const distDir = path.resolve(__dirname, `./js/hot-cases-web`);

describeByWalk(NAME, caseDir, distDir, (name, src, dist) => {
	createHotCase(name, src, dist, "web");
});
