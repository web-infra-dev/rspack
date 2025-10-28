process.env.RSPACK_INCREMENTAL_WATCH_TEST = true;

const path = require("path");
const {
	describeByWalk,
	createWatchIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/watchCases
describeByWalk(
	v("watch"),
	(name, src, dist) => {
		const tempDir = path.resolve(__dirname, `./js/incremental/temp`);
		createWatchIncrementalCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.resolve(__dirname, "./watchCases"),
		dist: path.resolve(__dirname, `./js/incremental/watch`)
	}
);
