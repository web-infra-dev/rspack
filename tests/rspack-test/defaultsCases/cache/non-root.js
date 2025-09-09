const path = require("path");
/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "filesystem"
		}
	}),
	cwd: path.resolve(__dirname, "../../fixtures"),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-   "cache": false,
		-   "context": "<cwd>",
		+   "cache": Object {
		+     "type": "filesystem",
		+   },
		+   "context": "<cwd>/fixtures",
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
		-     "path": "<cwd>/dist",
		+     "path": "<cwd>/fixtures/dist",
		@@ ... @@
		-     "uniqueName": "@rspack/tests",
		+     "uniqueName": "",
		@@ ... @@
		-       "<cwd>",
		+       "<cwd>/fixtures",
	`)
};
