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

// Run tests rspack-test-tools/tests/hotCases in target async-node
describeByWalk(
	v("hot async-node"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "async-node", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/hot-async-node`),
		exclude: [
			/^css$/,
			/move-between-runtime/,
			/require-disposed-module-warning/
		]
	}
);

// Run tests webpack-test/hotCases in target async-node
describeByWalk(
	v("hot async-node (webpack-test)"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "async-node", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(
			__dirname,
			`./js/new-incremental/webpack-test/hot-async-node`
		),
		exclude: [/move-between-runtime/, /require-disposed-module-warning/]
	}
);

// Run tests webpack-test/hotCases in target node
describeByWalk(
	v("hot node (webpack-test)"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "node", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-node`),
		exclude: [/move-between-runtime/, /require-disposed-module-warning/]
	}
);
