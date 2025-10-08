const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

if (process.platform === "win32" && process.env.CI) {
	describe.skip("NativeWatcher (skipped on Windows CI)", () => {
		it("skipped due to native watcher instability on Windows CI", () => {});
	});
} else {
	describeByWalk(
		__filename,
		(name, src, dist) => {
			createNativeWatcher(name, src, dist, path.join(tempDir, name));
		},
		{
			source: path.join(__dirname, `./watchCases`),
			dist: path.resolve(__dirname, `./js/native-watcher/watch`)
		}
	);
}
