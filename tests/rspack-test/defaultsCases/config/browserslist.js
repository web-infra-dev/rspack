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
			-     "chunkLoadingGlobal": "rspackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "rspackChunk",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "",
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdate",
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "",
			@@ ... @@
			-       "<cwd>",
			+       "<TEST_TOOLS_ROOT>/tests/fixtures/browserslist",
		`)
};
