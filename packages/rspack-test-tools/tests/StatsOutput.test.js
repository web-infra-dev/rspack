const { createStatsOutputCase, describeByWalk } = require("..");

describeByWalk(__filename, (name, src, dist) => {
	createStatsOutputCase(name, src, dist);
}, {
	level: 1,
});

