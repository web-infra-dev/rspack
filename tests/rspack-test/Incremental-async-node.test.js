const path = require("path");
const {
	describeByWalk,
	createHotIncrementalCase
} = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/incremental-async-node`);

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/hotCases in target async-node
describeByWalk(
	v("hot async-node"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, path.join(tempDir, name), "async-node", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-async-node`),
		exclude: [/^css$/]
	}
);

