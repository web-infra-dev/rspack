const path = require("path");
const { describeByWalk, createHotCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/hot-node`);

describeByWalk(
	__filename,
	(name, src, dist) => {
		createHotCase(name, src, dist, path.join(tempDir, name), "async-node");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/hot-node`),
		exclude: [/^css$/]
	}
);
