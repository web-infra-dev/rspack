const { createNormalCase, describeByWalk } = require("@rspack/test-tools");

describeByWalk(__filename, (name, src, dist) => {
	createNormalCase(name, src, dist);
});
