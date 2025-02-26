// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createHotNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests rspack-test-tools/tests/hotCases in target async-node
describeByWalk(
	v("hot async-node"),
	(name, src, dist) => {
		createHotNewIncrementalCase(name, src, dist, "async-node", "jsdom");
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/hot-async-node`),
		exclude: [/^css$/]
	}
);
