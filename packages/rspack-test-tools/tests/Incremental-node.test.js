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

// Run tests rspack-test-tools/tests/hotCases in target async-node
describeByWalk(
	v("hot async-node"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "async-node", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-async-node`),
		exclude: [/^css$/]
	}
);

// Run tests webpack-test/hotCases in target async-node
describeByWalk(
	v("hot async-node (webpack-test)"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "async-node", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(
			__dirname,
			`./js/incremental/webpack-test/hot-async-node`
		),
		exclude: [/require-disposed-module-warning/]
	}
);

// Run tests webpack-test/hotCases in target node
describeByWalk(
	v("hot node (webpack-test)"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "node", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/webpack-test/hot-node`),
		exclude: [/require-disposed-module-warning/]
	}
);
