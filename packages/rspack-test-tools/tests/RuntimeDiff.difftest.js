const { createDiffCase, describeByWalk } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createDiffCase(name, src, dist);
});
