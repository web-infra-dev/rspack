const { describeByWalk, createHotStepCase } = require("@rspack/test-tools");
const path = require("path");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotStepCase(name, src, dist, "web");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/hot-snapshot`)
	}
);
