const { describeByWalk, createEsmOutputCase } = require("../dist");

describeByWalk(__filename, (name, src, dist) => {
	createEsmOutputCase(name, src, dist);
});
