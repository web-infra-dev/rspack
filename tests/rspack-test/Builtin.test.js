const { describeByWalk, createBuiltinCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createBuiltinCase(name, src, dist);
});
