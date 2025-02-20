// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createWatchNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`);
}

// Run tests webpack-test/watchCases
describeByWalk(
	v("watch (webpack-test)"),
	(name, src, dist) => {
		const tempDir = path.resolve(
			__dirname,
			`./js/new-incremental/webpack-test/temp`
		);
		createWatchNewIncrementalCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/watchCases"),
		dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/watch`)
	}
);
