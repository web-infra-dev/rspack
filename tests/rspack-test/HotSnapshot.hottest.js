const path = require("path");
const { describeByWalk, createHotStepCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/hot-snapshot`);

describeByWalk(__filename, (name, src, dist) => {
	createHotStepCase(name, src, dist, path.join(tempDir, name), "web");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/hot-snapshot`),
	exclude: [/remove-add-worker/]
});
