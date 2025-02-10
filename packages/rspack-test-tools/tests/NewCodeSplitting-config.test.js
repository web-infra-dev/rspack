const path = require("path");
const { describeByWalk, createConfigNewCodeSplittingCase } = require("..");

// Run tests rspack-test-tools/tests/configCases
describeByWalk(
	"new-code-splitting config cases",
	(name, src, dist) => {
		createConfigNewCodeSplittingCase(name, src, dist);
	},
	{
		source: path.resolve(__dirname, "./configCases"),
		dist: path.resolve(__dirname, `./js/new-code-splitting-config`),
		exclude: [/esm-external/, /container-1/]
	}
);
