const { createStatsOutputCase, describeByWalk } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createStatsOutputCase(name, src, dist);
	},
	{
		level: 1
	}
);
