const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

// Part 1: Test cases starting with a-d (49 dirs, 36.0%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist);
	},
	{
		source: require("path").join(__dirname, "configCases"),
		dist: path.resolve(__dirname, `./js/config`),
		exclude: [
			// Exclude e-z and non-ascii
			/^[e-z]/,
			/^[^a-d]/
		]
	}
);
