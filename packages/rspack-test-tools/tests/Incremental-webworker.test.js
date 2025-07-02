// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const {
	describeByWalk,
	createHotIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test-tools/tests/hotCases in target webworker
describeByWalk(
	v("hot webworker"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "webworker", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-worker`),
		exclude: [/^css$/]
	}
);

// Run tests webpack-test/hotCases in target webworker
describeByWalk(
	v("hot webworker (webpack-test)"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "webworker", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(
			__dirname,
			`./js/incremental/webpack-test/hot-webworker`
		),
		exclude: [/require-disposed-module-warning/]
	}
);
