const path = require("path");
const { createCompilerCase, describeByWalk } = require("@rspack/test-tools");
const srcDir = path.resolve(__dirname, "./fixtures");

describeByWalk(
	__filename,
	(name, testConfig, dist) => {
		createCompilerCase(name, srcDir, dist, testConfig);
	},
	{
		level: 1,
		type: "file"
	}
);
