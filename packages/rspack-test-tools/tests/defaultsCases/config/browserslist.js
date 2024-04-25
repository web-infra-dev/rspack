const path = require("path");
module.exports = {
	description: "browserslist",
	options: context => ({
		context: path.resolve(context.getSource(), "./browserslist")
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "context": "<cwd>",
		+   "context": "<cwd>/tests/fixtures/browserslist",
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_test_tools",
		+     "chunkLoadingGlobal": "webpackChunkbrowserslist_test",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/test-tools",
		+     "devtoolNamespace": "browserslist-test",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_test_tools",
		+     "hotUpdateGlobal": "webpackHotUpdatebrowserslist_test",
		@@ ... @@
		-     "uniqueName": "@rspack/test-tools",
		+     "uniqueName": "browserslist-test",
		@@ ... @@
		-       "<cwd>",
		+       "<cwd>/tests/fixtures/browserslist",
	`)
};
