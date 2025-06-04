// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const {
	describeByWalk,
	createHotNewIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests rspack-test-tools/tests/hotCases in target webworker
describeByWalk(
	v("hot webworker"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "webworker", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/hot-worker`),
		exclude: [
			/^css$/,
			/move-between-runtime/,
			/require-disposed-module-warning/
		]
	}
);

// Run tests webpack-test/hotCases in target webworker
describeByWalk(
	v("hot webworker (webpack-test)"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "webworker", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(
			__dirname,
			`./js/new-incremental/webpack-test/hot-webworker`
		),
		exclude: [/move-between-runtime/, /require-disposed-module-warning/]
	}
);
