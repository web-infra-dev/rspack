const { describeByWalk, createSerialCase } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createSerialCase(name, src, dist);
});
