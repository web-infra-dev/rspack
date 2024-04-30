const { createNormalCase, describeByWalk } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createNormalCase(name, src, dist);
});

