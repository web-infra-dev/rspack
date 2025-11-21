const path = require("path");
/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
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
		+   "context": "<TEST_TOOLS_ROOT>/tests/fixtures/browserslist",
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_tests",
		+     "chunkLoadingGlobal": "webpackChunk",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/tests",
		+     "devtoolNamespace": "",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_tests",
		+     "hotUpdateGlobal": "webpackHotUpdate",
		@@ ... @@
		-     "uniqueName": "@rspack/tests",
		+     "uniqueName": "",
		@@ ... @@
		-       "<cwd>",
		+       "<TEST_TOOLS_ROOT>/tests/fixtures/browserslist",
	`)
};
