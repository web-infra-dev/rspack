const { createTreeShakingCase, describeByWalk } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createTreeShakingCase(name, src, dist);
	},
	{
		level: 1
	}
);
