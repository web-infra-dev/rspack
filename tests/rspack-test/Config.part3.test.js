const path = require("path");
const { describeByWalk, createConfigCase } = require("@rspack/test-tools");

// Part 3: Test cases starting with p-z and others (44 dirs, 32.4%)
describeByWalk(
	__filename,
	(name, src, dist) => {
		createConfigCase(name, src, dist);
	},
	{
		source: require("path").join(__dirname, "configCases"),
		dist: path.resolve(__dirname, `./js/config`),
		exclude: [
			// Exclude a-o
			/^[a-o]/
		]
	}
);
