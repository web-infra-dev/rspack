// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = 'loose-silent';

const path = require("path");
const { describeByWalk, createHotNewIncrementalCase, createWatchNewIncrementalCase } = require("../dist");

function postfixName(name) {
  return `${name} (newIncremental)`
}

describeByWalk(__filename, (name, src, dist) => {
	createHotNewIncrementalCase(postfixName(name), src, dist, "async-node");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-node`),
	exclude: [/^css$/]
});

describeByWalk(__filename, (name, src, dist) => {
	createHotNewIncrementalCase(postfixName(name), src, dist, "web");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-web`)
});

describeByWalk(__filename, (name, src, dist) => {
	createHotNewIncrementalCase(postfixName(name), src, dist, "webworker");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-worker`),
	exclude: [/^css$/]
});

// Run tests in rspack-test-tools/tests/watchCases
describeByWalk(__filename, (name, src, dist) => {
	const tempDir = path.resolve(__dirname, `./js/new-incremental/temp`);
	createWatchNewIncrementalCase(
		postfixName(name),
		src,
		dist,
		path.join(tempDir, name),
	);
}, {
	source: path.resolve(__dirname, "./watchCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/watch`),
});

// Run tests in webpack-test/watchCases
describeByWalk(__filename, (name, src, dist) => {
	const tempDir = path.resolve(__dirname, `./js/new-incremental/temp`);
	createWatchNewIncrementalCase(
		postfixName(name),
		src,
		dist,
		path.join(tempDir, name),
	);
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/watchCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/watch`),
  exclude: [
    /module-concatenation-plugin/,
    /missing-module/,
    /caching-inner-source/,
    /production/,
    /warnings-contribute-to-hash/,
  ]
});
