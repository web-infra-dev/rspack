const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/hot-worker`);

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotCase(name, src, dist, path.join(tempDir, name), "webworker");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/hot-worker`),
		exclude: [/^css$/]
	}
);
