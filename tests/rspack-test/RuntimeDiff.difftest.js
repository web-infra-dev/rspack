const { createDiffCase, describeByWalk } = require("@rspack/test-tools");
const path = require("path");

describeByWalk(__filename, (name, src, dist) => {
	createDiffCase(name, src, dist);
}, {
	dist: path.resolve(__dirname, `./js/runtime-diff`)
});
