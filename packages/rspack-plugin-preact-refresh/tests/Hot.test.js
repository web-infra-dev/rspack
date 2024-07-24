const { describeByWalk, createHotCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createHotCase(name, src, dist, "web");
});
