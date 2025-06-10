// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const {
	describeByWalk,
	createHotIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test-tools/tests/hotCases in target web
describeByWalk(
	v("hot web"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "web", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-web`)
	}
);

// Run tests webpack-test/hotCases in target web
describeByWalk(
	v("hot web (webpack-test)"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "web", true);
	},
	{
		source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/webpack-test/hot-web`),
		exclude: [
			// there is a self reference module in this case causing the make phase didn't found the module is removed
			/require-disposed-module-warning/
		]
	}
);
