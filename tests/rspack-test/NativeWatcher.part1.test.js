const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Part 1: Test cases starting with a-co (12 dirs, 37.5%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(__dirname, `./watchCases`),
		dist: path.resolve(__dirname, `./js/native-watcher/watch`),
		exclude: [
			// Exclude cp-z
			/^c[p-z]/,
			/^[d-z]/
		]
	}
);
