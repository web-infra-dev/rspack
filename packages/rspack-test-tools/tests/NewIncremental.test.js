// Need to run some webpack-test
process.env.RSPACK_CONFIG_VALIDATE = 'loose-silent';

const path = require("path");
const { describeByWalk, createHotNewIncrementalCase, createWatchNewIncrementalCase } = require("../dist");

function v(name) {
	return path.join(__dirname, `new-incremental ${name}`)
}

// Run tests rspack-test-tools/tests/hotCases in target async-node
describeByWalk(v("hot async-node"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "async-node", "jsdom");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-async-node`),
	exclude: [/^css$/]
});

// Run tests rspack-test-tools/tests/hotCases in target web
describeByWalk(v("hot web"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "web", "jsdom");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-web`)
});

// Run tests rspack-test-tools/tests/hotCases in target webworker
describeByWalk(v("hot webworker"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "webworker", "jsdom");
}, {
	source: path.resolve(__dirname, "./hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/hot-worker`),
	exclude: [/^css$/]
});

// Run tests rspack-test-tools/tests/watchCases
describeByWalk(v("watch"), (name, src, dist) => {
	const tempDir = path.resolve(__dirname, `./js/new-incremental/temp`);
	createWatchNewIncrementalCase(
		name,
		src,
		dist,
		path.join(tempDir, name),
	);
}, {
	source: path.resolve(__dirname, "./watchCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/watch`),
});

// Run tests webpack-test/hotCases in target async-node
describeByWalk(v("hot async-node (webpack-test)"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "async-node", "fake");
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-async-node`),
	exclude: [
		/move-between-runtime/,
	]
});

// Run tests webpack-test/hotCases in target node
describeByWalk(v("hot node (webpack-test)"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "node", "fake");
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-node`),
	exclude: [
		/move-between-runtime/,
	]
});

// Run tests webpack-test/hotCases in target web
describeByWalk(v("hot web (webpack-test)"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "web", "fake");
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-web`),
	exclude: [
		/move-between-runtime/,
	]
});

// Run tests webpack-test/hotCases in target webworker
describeByWalk(v("hot webworker (webpack-test)"), (name, src, dist) => {
	createHotNewIncrementalCase(name, src, dist, "webworker", "fake");
}, {
	source: path.resolve(__dirname, "../../../tests/webpack-test/hotCases"),
	dist: path.resolve(__dirname, `./js/new-incremental/webpack-test/hot-webworker`),
	exclude: [
		/move-between-runtime/,
	]
});

// Run tests webpack-test/watchCases
describeByWalk(v("watch (webpack-test)"), (name, src, dist) => {
	const tempDir = path.resolve(__dirname, `./js/new-incremental/webpack-test/temp`);
	createWatchNewIncrementalCase(
		name,
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
