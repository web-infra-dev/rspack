/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "library",
	options: () => ({ output: { library: ["myLib", "awesome"] } }),
	diff: e =>
		e.toMatchInlineSnapshot(`
			- Expected
			+ Received

			@@ ... @@
			-       "force": true,
			+       "force": false,
			@@ ... @@
			-     "chunkLoadingGlobal": "rspackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "rspackChunkmyLib_awesome",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "myLib.awesome",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "rspackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "rspackHotUpdatemyLib_awesome",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "name": Array [
			+         "myLib",
			+         "awesome",
			+       ],
			+       "type": "var",
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "myLib.awesome",
		`)
};
