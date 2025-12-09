const path = require("path");
const { describeByWalk, createWatchCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Part 3: Test cases starting with r-z (11 dirs, 34.4%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createWatchCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: require("path").join(__dirname, "watchCases"),
		dist: path.resolve(__dirname, `./js/watch`),
		exclude: [
			// Exclude a-p
			/^[a-p]/
		]
	}
);
