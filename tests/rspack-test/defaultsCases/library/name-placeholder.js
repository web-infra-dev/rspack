/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "library.name contains [name] placeholder",
	options: () => ({
		output: {
			library: {
				name: ["my[name]Lib", "[name]", "lib"],
				type: "var"
			}
		}
	}),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-       "force": true,
			+       "force": false,
			@@ ... @@
			-     "chunkLoadingGlobal": "webpackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "webpackChunkmyLib_lib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "myLib.lib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdatemyLib_lib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "name": Array [
			+         "my[name]Lib",
			+         "[name]",
			+         "lib",
			+       ],
			+       "type": "var",
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "myLib.lib",
		`)
};
