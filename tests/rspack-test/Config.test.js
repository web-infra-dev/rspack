const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createConfigCase(name, src, dist);
});
