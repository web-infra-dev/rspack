/** @type {import('@rspack/test-tools').TDefaultsCaseConfig} */
module.exports = {
	description: "library.name.root contains [name] placeholder",
	options: () => ({
		output: {
			library: {
				name: {
					root: ["[name]", "myLib"]
				},
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
			+     "chunkLoadingGlobal": "webpackChunkmyLib",
			@@ ... @@
			-     "devtoolNamespace": "@rspack/tests",
			+     "devtoolNamespace": "myLib",
			@@ ... @@
			-     "enabledLibraryTypes": Array [],
			+     "enabledLibraryTypes": Array [
			+       "var",
			+     ],
			@@ ... @@
			-     "hotUpdateGlobal": "webpackHotUpdate_rspack_tests",
			+     "hotUpdateGlobal": "webpackHotUpdatemyLib",
			@@ ... @@
			-     "library": undefined,
			+     "library": Object {
			+       "amdContainer": undefined,
			+       "auxiliaryComment": undefined,
			+       "export": undefined,
			+       "name": Object {
			+         "root": Array [
			+           "[name]",
			+           "myLib",
			+         ],
			+       },
			+       "type": "var",
			+       "umdNamedDefine": undefined,
			+     },
			@@ ... @@
			-     "uniqueName": "@rspack/tests",
			+     "uniqueName": "myLib",
		`)
};
