// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createWatchNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests rspack-test-tools/tests/watchCases
describeByWalk(
	v("watch"),
	(name, src, dist) => {
		const tempDir = path.resolve(__dirname, `./js/new-incremental/temp`);
		createWatchNewIncrementalCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.resolve(__dirname, "./watchCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/watch`)
	}
);
