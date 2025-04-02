const path = require("path");
const { createHookCase, describeByWalk } = require("@rspack/test-tools");
const source = path.resolve(__dirname, "./fixtures");

describeByWalk(__filename, (name, src, dist) => {
	createHookCase(name, src, dist, source);
});
