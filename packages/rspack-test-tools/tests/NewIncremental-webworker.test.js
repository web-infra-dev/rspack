// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createHotNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests rspack-test-tools/tests/hotCases in target webworker
describeByWalk(
	v("hot webworker"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "webworker", "jsdom");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/hot-worker`),
		exclude: [/^css$/]
	}
);

// Run tests webpack-test/hotCases in target webworker
describeByWalk(
	v("hot webworker (webpack-test)"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "webworker", "fake");
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(
			__dirname,
			`./js/new-incremental/webpack-test/hot-webworker`
		)
	}
);
