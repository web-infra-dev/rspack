/** @type {import('../../..').TDefaultsCaseConfig} */
module.exports = {
	description: "library contains [name] placeholder",
	options: () => ({
		output: {
			library: ["myLib", "[name]"]
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
		- Expected
		+ Received

		@@ ... @@
		-     "chunkLoadingGlobal": "webpackChunk_rspack_test_tools",
		+     "chunkLoadingGlobal": "webpackChunkmyLib",
		@@ ... @@
		-     "devtoolNamespace": "@rspack/test-tools",
		+     "devtoolNamespace": "myLib",
		@@ ... @@
		-     "enabledLibraryTypes": Array [],
		+     "enabledLibraryTypes": Array [
		+       "var",
		+     ],
		@@ ... @@
		-     "hotUpdateGlobal": "webpackHotUpdate_rspack_test_tools",
		+     "hotUpdateGlobal": "webpackHotUpdatemyLib",
		@@ ... @@
		-     "library": undefined,
		+     "library": Object {
		+       "amdContainer": undefined,
		+       "auxiliaryComment": undefined,
		+       "export": undefined,
		+       "name": Array [
		+         "myLib",
		+         "[name]",
		+       ],
		+       "type": "var",
		+       "umdNamedDefine": undefined,
		+     },
		@@ ... @@
		-     "uniqueName": "@rspack/test-tools",
		+     "uniqueName": "myLib",
	`)
};
