const { describeByWalk, createSerialCase } = require("../dist");

describeByWalk(__filename, (name, src, dist) => {
	createSerialCase(name, src, dist);
});
