process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const {
	describeByWalk,
	createHotIncrementalCase
} = require("@rspack/test-tools");

function v(name) {
	return path.join(__dirname, `incremental ${name}`);
}

// Run tests rspack-test/tests/hotCases in target async-node
describeByWalk(
	v("hot node"),
	(name, src, dist) => {
		createHotIncrementalCase(name, src, dist, "node", false);
	},
	{
		source: path.resolve(__dirname, "./hotCases"),
		dist: path.resolve(__dirname, `./js/incremental/hot-node`),
		exclude: [
			/^css$/,
			// FIXME: incremental failed
			/^rebuild-abnormal-module$/
		]
	}
);

