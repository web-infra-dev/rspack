// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createHotNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests webpack-test/hotCases in target node
describeByWalk(
	v("hot node (webpack-test)"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "node", "fake");
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-node`),
		exclude: [/move-between-runtime/]
	}
);
