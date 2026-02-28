const path = require("path");
const { describeByWalk, createCacheCase } = require("@rspack/test-tools");
const tempDir = path.resolve(__dirname, `./js/temp`);

// Run tests rspack-test/tests/cacheCases in target async-node
describeByWalk(
	__filename,
	(name, src, dist) => {
		createCacheCase(name, src, dist, "async-node", path.join(tempDir, name));
	},
	{
		source: path.resolve(__dirname, "./cacheCases"),
		dist: path.resolve(__dirname, `./js/cache/async-node`),
		exclude: [/^css$/]
	}
);
