const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

// Part 2: Test cases starting with e-o (43 dirs, 31.6%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist);
	},
	{
		source: require("path").join(__dirname, "configCases"),
		dist: path.resolve(__dirname, `./js/config`),
		exclude: [
			// Exclude a-d
			/^[a-d]/,
			// Exclude p-z and non-ascii
			/^[p-z]/,
			/^[^a-o]/
		]
	}
);
