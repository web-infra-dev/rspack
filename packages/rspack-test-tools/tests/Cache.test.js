// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = "loose-silent";

const path = require("path");
const { describeByWalk, createCacheCase } = require("../dist");

// Run tests rspack-test-tools/tests/cacheCases in target async-node
describeByWalk(
	"cache",
	(name, src, dist) => {
		createCacheCase(name, src, dist, "async-node");
	},
	{
		source: path.resolve(__dirname, "./cacheCases"),
		dist: path.resolve(__dirname, `./js/cache/async-node`),
		exclude: [/^css$/]
	}
);
