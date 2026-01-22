/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
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
			-       "force": true,
			+       "force": false,
			@@ ... @@
			-     "chunkLoadingGlobal": "rspackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "rspackChunkmyLib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "myLib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdatemyLib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "name": Array [
			+         "myLib",
			+         "[name]",
			+       ],
			+       "type": "var",
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "myLib",
		`)
};
