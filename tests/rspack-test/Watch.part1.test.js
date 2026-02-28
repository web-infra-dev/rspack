const path = require("path");
const { describeByWalk, createWatchCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Part 1: Test cases starting with a-co (12 dirs, 37.5%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createWatchCase(name, src, dist, path.join(tempDir, name));
	},
	{
		source: require("path").join(__dirname, "watchCases"),
		dist: path.resolve(__dirname, `./js/watch`),
		exclude: [
			// Exclude cp-z
			/^c[p-z]/,
			/^[d-z]/
		]
	}
);
