const { createHashCase, describeByWalk } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHashCase(name, src, dist);
	},
	{
		level: 1,
		absoluteDist: false
	}
);
