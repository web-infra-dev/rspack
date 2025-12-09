const path = require("path");
const { describeByWalk, createNativeWatcher } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Part 2: Test cases starting with cp-p (9 dirs, 28.1%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createNativeWatcher(name, src, dist, path.join(tempDir, name));
	},
	{
		source: path.join(__dirname, `./watchCases`),
		dist: path.resolve(__dirname, `./js/native-watcher/watch`),
		exclude: [
			// Exclude a-co
			/^[a-c][a-o]/,
			/^[ab]/,
			// Exclude r-z
			/^[r-z]/
		]
	}
);
