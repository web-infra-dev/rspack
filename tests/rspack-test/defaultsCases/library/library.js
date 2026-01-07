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
			-     "chunkLoadingGlobal": "webpackChunk_rspack_tests",
			+     "chunkLoadingGlobal": "webpackChunkmyLib_awesome",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "myLib.awesome",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "webpackHotUpdatemyLib_awesome",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Array [
			+         "myLib",
			+         "awesome",
			+       ],
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "myLib.awesome",
		`)
};
