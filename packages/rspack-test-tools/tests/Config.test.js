const { describeByWalk, createConfigCase } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createConfigCase(name, src, dist);
});
