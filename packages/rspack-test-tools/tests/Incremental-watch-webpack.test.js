// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const {
	describeByWalk,
	createWatchIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests webpack-test/watchCases
describeByWalk(
	v("watch (webpack-test)"),
	(name, src, dist) => {
		const tempDir = path.resolve(
			__dirname,
			`./js/incremental/webpack-test/temp`
		);
		createWatchIncrementalCase(name, src, dist, path.join(tempDir, name), {
			ignoreNotFriendlyForIncrementalWarnings: true
		});
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/watchCases"),
		dist: path.resolve(__dirname, `./js/incremental/webpack-test/watch`)
	}
);
