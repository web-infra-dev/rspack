// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";
const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `native_watcher ${name}`);
}

describeByWalk(
	v("(webpack-test)"),
	(name, src, dist) => {
		const tempDir = path.resolve(
			__dirname,
			`./js/incremental/webpack-test/temp`
		);
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/watchCases"),
		dist: path.resolve(__dirname, `./js/native-watcher/webpack-test/watch`)
	}
);
