// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createCacheCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `cache ${name}`);
}

// Run tests rspack-test-tools/tests/cacheCases in target async-node
describeByWalk(
	v("hot async-node"),
	(name, src, dist) => {
		createCacheCase(name, src, dist, "async-node");
	},
	{
		source: path.resolve(__dirname, "./cacheCases"),
		dist: path.resolve(__dirname, `./js/cache/async-node`),
		exclude: [/^css$/]
	}
);
