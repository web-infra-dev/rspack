// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";
const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

function v(name) {
	return path.join(__dirname, `native_watcher ${name}`);
}

if (process.platform === "win32" && process.env.CI) {
	describe.skip("NativeWatcher webpack parity (skipped on Windows CI)", () => {
		it("skipped due to native watcher instability on Windows CI", () => {});
	});
} else {
	describeByWalk(
		v("(webpack-test)"),
		(name, src, dist) => {
			createNativeWatcher(name, src, dist, path.join(tempDir, name));
		},
		{
			source: path.resolve(__dirname, "../webpack-test/watchCases"),
			dist: path.resolve(
				__dirname,
				`./js/native-watcher/webpack-test/watch`
			)
		}
	);
}
