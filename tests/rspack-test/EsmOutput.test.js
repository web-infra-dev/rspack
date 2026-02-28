const { describeByWalk, createEsmOutputCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createEsmOutputCase(name, src, dist);
});
