const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Part 3: Test cases starting with r-z (11 dirs, 34.4%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(__dirname, `./watchCases`),
		dist: path.resolve(__dirname, `./js/native-watcher/watch`),
		exclude: [
			// Exclude a-p
			/^[a-p]/,
			/skip-building-chunk-graph/
		]
	}
);
