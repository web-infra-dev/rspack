const { createTreeShakingCase, describeByWalk } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createTreeShakingCase(name, src, dist);
}, {
	level: 1,
});


