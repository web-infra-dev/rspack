const path = require("path");
const { describeByWalk, createWatchCase, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

describeByWalk(__filename, (name, src, dist) => {
	createWatchCase(name, src, dist, path.join(tempDir, name));
});

// TODO: remove this when rspack native watcher is stable
const failedCases = [
	'reuse-deps-for-incremental-make',
	'built-modules',
	'dynamic-entries',
]
const _tempDir = path.resolve(__dirname, `./js/temp/experimental`);
describeByWalk(__filename, (name, src, dist) => {
	if (failedCases.some((caseName) => name.includes(caseName))) {
		return;
	}
	createNativeWatcher(name, src, dist, path.join(_tempDir, name), {
		nativeWatcher: true
	});
});
