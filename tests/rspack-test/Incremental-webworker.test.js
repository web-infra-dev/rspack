const path = require("path");
const {
	describeByWalk,
	createHotIncrementalCase
} = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp/incremental-webworker`);

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/hotCases in target webworker
describeByWalk(
	v("hot webworker"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, path.join(tempDir, name), "webworker", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-worker`),
		exclude: [/^css$/]
	}
);


