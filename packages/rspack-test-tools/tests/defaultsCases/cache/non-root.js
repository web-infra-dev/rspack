const path = require("path");
module.exports = {
	description: "non-root directory",
	options: () => ({
		cache: {
			type: "filesystem"
		}
	}),
	cwd: path.resolve(__dirname, "../../../../rspack/tests/fixtures"),
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
		+   "context": "<cwd>/tests/fixtures",
		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_core",
		+     "chunkLoadingGlobal": "webpackChunk",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/core",
		+     "devtoolNamespace": "",
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_core",
		+     "hotUpdateGlobal": "webpackHotUpdate",
		@@ ... @@
		-     "path": "<cwd>/dist",
		+     "path": "<cwd>/tests/fixtures/dist",
		@@ ... @@
		-     "uniqueName": "@rspack/core",
		+     "uniqueName": "",
		@@ ... @@
		-       "<cwd>",
		+       "<cwd>/tests/fixtures",
	`)
};
