const path = require("path");
const { createErrorCase, describeByWalk } = require("@rspack/test-tools");
const caseDir = path.resolve(__dirname, "./errorCases");

describeByWalk(
	__filename,
	(name, testConfig, dist) => {
		createErrorCase(name, caseDir, dist, testConfig);
	},
	{
		level: 1,
		type: "file"
	}
);
