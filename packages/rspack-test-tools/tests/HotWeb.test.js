const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotCase(name, src, dist, "web");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/hot-web`)
	}
);
