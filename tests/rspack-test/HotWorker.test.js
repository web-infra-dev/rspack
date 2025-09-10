const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotCase(name, src, dist, "webworker");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/hot-worker`),
		exclude: [/^css$/]
	}
);
